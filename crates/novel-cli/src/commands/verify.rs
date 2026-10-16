// `novel verify` 命令：完整性验证（s07-archive-and-export / s04-error-render-unification）。
//
// 职责（CLAUDE.md §6、design D5）：参数解析 + 输出选择，调用 verify_service，渲染分层报告。
// hard 违规经 `lib::run()`→`renderers::render_error` 统一输出结构化错误对象（其 `details`
// 承载完整报告失败项），不再 `std::process::exit` 直出绕过错误对象承载。验证为只读操作。

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::errors::CliError;
use crate::renderers::verify_renderer;
use crate::services::verify_service;

pub fn build() -> Command {
    Command::new("verify")
        .about("完整性验证")
        .subcommand_required(true)
        .subcommand(
            Command::new("integrity")
                .about("对世界状态做完整性验证（hash、引用、状态一致性）")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出 JSON 分层报告（默认即 JSON）"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("integrity", sub_matches)) => handle_integrity(sub_matches),
        _ => Err(CliError::usage(
            "未知 verify 子命令。可用：integrity。运行 `novel verify --help` 查看用法。",
        )),
    }
}

fn handle_integrity(matches: &ArgMatches) -> Result<(), CliError> {
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");

    let target = Path::new(path);
    // 世界已初始化的前置校验下沉至 verify_service::verify_integrity（其内部 open_existing_at
    // 在缺 .novel/novel.db 时返回 world_not_initialized，退出码 3）——command 层不再 open。
    let report = verify_service::verify_integrity(target)?;

    let (exit_code, code) = verify_service::verify_exit_mapping(&report);

    if exit_code == crate::errors::ExitCode::Success {
        // 成功路径（design D5 步骤 3）：全通过，输出报告 JSON / 人读摘要，返回 Ok(())。
        if json {
            let output = verify_renderer::render_verify_integrity_json(&report)
                .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
            println!("{}", output);
        } else {
            let output = verify_renderer::render_verify_integrity_human(&report);
            print!("{}", output);
        }
        return Ok(());
    }

    // hard 违规（design D5 步骤 4）：不打印报告，构造 `VerifyIntegrity` 错误返回 Err，
    // 由 `lib::run()`→`render_error` 统一输出结构化错误对象（--json stdout 错误对象，
    // 非 JSON stderr 人读错误）。错误对象 `details` 承载完整报告（同成功路径结构，含
    // schemaVersion/三分项），让 Spec 经错误对象获取失败项，不依赖裸退出码、不二次输出。
    // details 复用 renderer 的成功路径 JSON 结构（VerifyReportJson），保证与成功路径同构。
    let report_json = verify_renderer::render_verify_integrity_json(&report)
        .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    let details: serde_json::Value = serde_json::from_str(&report_json)
        .map_err(|e| CliError::storage(format!("报告 JSON 解析失败：{}", e)))?;

    // 失败项计数（人读概览 message）。退出码语义不变，message/suggestion 文本契约测试不固化。
    let failed_count = report.hash_integrity.mismatches.len()
        + report.hash_integrity.missing_files.len()
        + report.reference_integrity.dangling_references.len()
        + report.state_consistency.inconsistencies.len();
    let message = format!("完整性验证未通过：{} 项失败", failed_count);
    let suggestion = Some(
        "运行 `novel chapter reimport <id>` 重新导入被外部修改的正文，或检查世界数据库的引用与状态一致性。".to_string(),
    );

    // 05 治理默认化（V2 默认）：退出码由错误类别派生（`verify_exit_code_v2`），
    // 与 `lib.rs` 的类别派生进程退出码一致。V1 per-entity 退出码由 `verify_exit_mapping`
    // 保留为弃用兼容路径的单一映射源（`code` 仍取自该映射，决定目录错误 ID 与类别）。
    let v2_exit_code = verify_service::verify_exit_code_v2(&report);
    let exit_code = exit_code_v2_for(v2_exit_code).unwrap_or(exit_code);
    debug_assert_eq!(i32::from(exit_code), v2_exit_code, "V2 退出码应为类别派生值");

    Err(verify_service::make_verify_integrity_error(
        exit_code,
        code,
        details,
        message,
        suggestion,
    ))
}

/// 把类别派生的退出码数值映射回 `ExitCode` 变体（仅用于承载在 `VerifyIntegrity` 错误上）。
///
/// 类别码 `{0,2,3,4,5,6,70}` 均是 `ExitCode` 既有变体；未知数值返回 `None`（调用方回退 V1 码）。
fn exit_code_v2_for(code: i32) -> Option<crate::errors::ExitCode> {
    use crate::errors::ExitCode;
    match code {
        0 => Some(ExitCode::Success),
        2 => Some(ExitCode::Usage),
        3 => Some(ExitCode::PreconditionBlocked),
        4 => Some(ExitCode::ResourceConflict),
        5 => Some(ExitCode::ExternalModification),
        6 => Some(ExitCode::NotFound),
        70 => Some(ExitCode::Storage),
        _ => None,
    }
}
