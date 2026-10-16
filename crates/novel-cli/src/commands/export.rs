// `novel export` 命令：导出世界状态快照（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6）：参数解析 + 输出选择，调用 export_service，渲染 JSON。
// 导出为只读操作。export chapter hash 失配返回 21；export full hash 失配列入 warnings（退出码 0）。

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::errors::CliError;
use crate::renderers::export_renderer;
use crate::services::export_service;

pub fn build() -> Command {
    Command::new("export")
        .about("导出世界状态快照")
        .subcommand_required(true)
        .subcommand(
            Command::new("volume")
                .about("导出单个卷及其下属实体")
                .arg(
                    Arg::new("volume-id")
                        .required(true)
                        .value_name("VOLUME_ID")
                        .value_parser(clap::value_parser!(i64))
                        .help("卷 ID"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出 JSON 格式（export 默认即为 JSON，此开关用于契约兼容性）"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("chapter")
                .about("导出单个章节及其裁决与规则检查结果")
                .arg(
                    Arg::new("chapter-id")
                        .required(true)
                        .value_name("CHAPTER_ID")
                        .help("章节 ID"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出 JSON 格式（export 默认即为 JSON，此开关用于契约兼容性）"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("full")
                .about("导出整个世界的全量快照")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出 JSON 格式（export 默认即为 JSON，此开关用于契约兼容性）"),
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
        Some(("volume", sub_matches)) => handle_volume(sub_matches),
        Some(("chapter", sub_matches)) => handle_chapter(sub_matches),
        Some(("full", sub_matches)) => handle_full(sub_matches),
        _ => Err(CliError::usage(
            "未知 export 子命令。可用：volume、chapter、full。运行 `novel export --help` 查看用法。",
        )),
    }
}

fn handle_volume(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");

    let target = Path::new(path);
    let (schema_version, data) = export_service::export_volume(target, volume_id)?;
    let json = export_renderer::render_export_volume_json(&schema_version, &data)
        .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    println!("{}", json);
    Ok(())
}

fn handle_chapter(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches.get_one::<String>("chapter-id").unwrap();
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");

    let target = Path::new(path);
    let (schema_version, data) = export_service::export_chapter(target, chapter_id)?;
    let json = export_renderer::render_export_chapter_json(&schema_version, &data)
        .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    println!("{}", json);
    Ok(())
}

fn handle_full(matches: &ArgMatches) -> Result<(), CliError> {
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");

    let target = Path::new(path);
    let (schema_version, data) = export_service::export_full(target)?;
    let json = export_renderer::render_export_full_json(&schema_version, &data)
        .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    println!("{}", json);
    Ok(())
}
