// `novel chapter` 命令：参数解析、基础校验、调用 service、选择输出格式。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §4）：command 层只做参数解析与基础校验，
// 不实现业务规则，不直接访问 storage。写命令只调用对应 service/use-case。

use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::domain::ChapterContextView;
use crate::errors::CliError;
use crate::renderers::{chapter_context_renderer, chapter_renderer};
use crate::services::chapter_context_service;
use crate::services::chapter_service;
use crate::services::chapter_state_service;
use std::path::Path;

pub fn build() -> Command {
    Command::new("chapter")
        .about("章节管理命令")
        .subcommand(
            Command::new("create")
                .about("创建新章节")
                .arg(
                    Arg::new("title")
                        .required(true)
                        .value_name("TITLE")
                        .help("章节标题"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出稳定结构化 JSON"),
                )
                .arg(
                    Arg::new("volume")
                        .long("volume")
                        .value_name("VOLUME_ID")
                        .value_parser(clap::value_parser!(i64))
                        .help("归属卷 ID（可选）；传入时校验卷存在且状态为 active"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("更新章节正文")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("章节 ID（如 chapter-001）"),
                )
                .arg(
                    Arg::new("content")
                        .long("content")
                        .required(true)
                        .value_name("FILE")
                        .help("正文文件路径"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出稳定结构化 JSON"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("get")
                .about("读取章节元数据和正文")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("章节 ID（如 chapter-001）"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出稳定结构化 JSON"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("列出所有章节")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出稳定结构化 JSON"),
                )
                .arg(
                    Arg::new("verify-hash")
                        .long("verify-hash")
                        .action(ArgAction::SetTrue)
                        .help("验证章节正文 hash 一致性"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("reimport")
                .about("重新导入外部修改的正文")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("章节 ID（如 chapter-001）"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("推进章节状态")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("章节 ID（如 chapter-001）"),
                )
                .arg(
                    Arg::new("target-status")
                        .required(true)
                        .value_name("TARGET-STATUS")
                        .help("目标状态（outline/drafting/review/approved）")
                        .value_parser(["outline", "drafting", "review", "approved"]),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出稳定结构化 JSON"),
                ),
        )
        .subcommand(
            Command::new("context")
                .about("渲染章节上下文包（Chapter Context Packet，只读）")
                .arg(
                    Arg::new("id")
                        .required(true)
                        .value_name("ID")
                        .help("章节 ID（如 chapter-001）"),
                )
                .arg(
                    Arg::new("view")
                        .long("view")
                        .required(true)
                        .value_name("VIEW")
                        .help("受控视图名：writer|verifier|repair（views-scopes.md §A 组 4）"),
                )
                .arg(
                    Arg::new("budget")
                        .long("budget")
                        .required(true)
                        .value_name("N")
                        .help("上下文预算（正整数，token 估算）"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("输出稳定结构化 JSON"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
}

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("create", sub)) => run_create(sub),
        Some(("update", sub)) => run_update(sub),
        Some(("get", sub)) => run_get(sub),
        Some(("list", sub)) => run_list(sub),
        Some(("reimport", sub)) => run_reimport(sub),
        Some(("status", sub)) => run_status(sub),
        Some(("context", sub)) => run_context(sub),
        _ => Err(CliError::usage(
            "请指定 chapter 子命令。可用：create, update, get, list, reimport, status, context。运行 `novel chapter --help` 查看用法。",
        )),
    }
}

/// 解析世界目标目录：`--path` 缺省→当前工作目录；提供→绝对化；目录不存在→退出码 2。
///
/// chapter 组所有子命令共用此解析逻辑，与 commands.md `[--path <dir>]` 契约一致。
fn resolve_target_dir(path: Option<&str>) -> Result<std::path::PathBuf, CliError> {
    let target_dir = match path {
        Some(p) => std::path::absolute(p)
            .map_err(|e| CliError::usage(format!("解析 --path 失败: {e}")))?,
        None => std::env::current_dir()
            .map_err(|e| CliError::storage(format!("获取当前目录失败：{}", e)))?,
    };
    if !target_dir.is_dir() {
        return Err(CliError::usage(format!(
            "`--path` 指向的目录不存在: {}",
            target_dir.display()
        )));
    }
    Ok(target_dir)
}

fn run_create(matches: &ArgMatches) -> Result<(), CliError> {
    let title = matches
        .get_one::<String>("title")
        .map(String::as_str)
        .unwrap_or("");
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);
    let volume_id = matches.get_one::<i64>("volume").copied();

    let target_dir = resolve_target_dir(path)?;

    // 调用 service 层创建章节（--volume 传入时校验卷存在且 active 并写入 volume_id）
    let chapter_id = chapter_service::create_chapter(&target_dir, title, volume_id)?;

    // 获取创建时间与归属卷（重新读取章节以获取完整信息）
    let chapter = chapter_service::get_chapter(&target_dir, &chapter_id)?;

    // 渲染输出
    if json {
        let output = chapter_renderer::render_chapter_create_json(
            &chapter_id,
            title,
            &chapter.created_at,
            chapter.volume_id,
        );
        println!("{}", output);
    } else {
        let output = chapter_renderer::render_chapter_create_human(&chapter_id, title);
        println!("{}", output);
    }

    Ok(())
}

fn run_update(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("id")
        .map(String::as_str)
        .unwrap_or("");
    let content_file = matches
        .get_one::<String>("content")
        .map(String::as_str)
        .unwrap_or("");
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target_dir = resolve_target_dir(path)?;

    let chapter =
        chapter_service::update_chapter_content(&target_dir, chapter_id, Path::new(content_file))?;

    if json {
        println!("{}", chapter_renderer::render_chapter_update_json(&chapter));
    }

    Ok(())
}

fn run_get(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("id")
        .map(String::as_str)
        .unwrap_or("");
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target_dir = resolve_target_dir(path)?;

    // 调用 service 层读取章节
    let chapter = chapter_service::get_chapter(&target_dir, chapter_id)?;

    // 渲染输出
    if json {
        let output = chapter_renderer::render_chapter_json(&chapter);
        println!("{}", output);
    } else {
        let output = chapter_renderer::render_chapter_human(&chapter);
        println!("{}", output);
    }

    Ok(())
}

fn run_list(matches: &ArgMatches) -> Result<(), CliError> {
    let json = matches.get_flag("json");
    let verify_hash = matches.get_flag("verify-hash");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target_dir = resolve_target_dir(path)?;

    // 调用 service 层列出章节
    let chapters = chapter_service::list_chapters(&target_dir, verify_hash)?;

    // 渲染输出
    if json {
        let output = chapter_renderer::render_chapter_list_json(&chapters, verify_hash);
        println!("{}", output);
    } else {
        let output = chapter_renderer::render_chapter_list_human(&chapters);
        println!("{}", output);
    }

    Ok(())
}

fn run_reimport(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("id")
        .map(String::as_str)
        .unwrap_or("");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target_dir = resolve_target_dir(path)?;

    // 调用 service 层重新导入章节（静默成功，退出码 0；见 commands.md chapter reimport 行为语义）。
    chapter_service::reimport_chapter(&target_dir, chapter_id)?;

    Ok(())
}

fn run_status(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("id")
        .map(String::as_str)
        .unwrap_or("");
    let target_status = matches
        .get_one::<String>("target-status")
        .map(String::as_str)
        .unwrap_or("");
    let path = matches.get_one::<String>("path").map(String::as_str);
    let json = matches.get_flag("json");

    let target_dir = resolve_target_dir(path)?;

    // 调用 service 层推进状态
    let result = chapter_state_service::transition_status(&target_dir, chapter_id, target_status)?;

    // 渲染输出
    if json {
        let output = chapter_renderer::render_chapter_status_json(
            &result.chapter_id,
            &result.to,
            &result.updated_at,
        );
        println!("{}", output);
    } else {
        println!("✓ 章节状态已更新：{} → {}", result.from, result.to);
    }

    Ok(())
}

fn run_context(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("id")
        .map(String::as_str)
        .unwrap_or("");
    let view_str = matches
        .get_one::<String>("view")
        .map(String::as_str)
        .unwrap_or("");
    let budget_str = matches
        .get_one::<String>("budget")
        .map(String::as_str)
        .unwrap_or("");
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 参数解析：受控 view 名校验（退出码 2 + usage）。
    let view: ChapterContextView = view_str.parse().map_err(|e: String| {
        CliError::usage(format!(
            "{e} 受控 --view 名引用 contracts/cli/views-scopes.md §A 组 4 冻结枚举：writer、verifier、repair。"
        ))
    })?;

    // 参数解析：--budget 正整数校验（退出码 2 + usage）。
    let budget: i64 = budget_str.parse().map_err(|_| {
        CliError::usage(format!("--budget 必须为正整数，得到 '{}'.", budget_str))
    })?;
    if budget <= 0 {
        return Err(CliError::usage(format!(
            "--budget 必须为正整数，得到 {}.", budget
        )));
    }

    let target_dir = resolve_target_dir(path)?;

    // 调用 service 层渲染上下文包（只读）。预算超限返回 ContextBudgetExceeded，
    // 由 lib::run() 的 render_error 统一输出错误对象（含 budget/failures/partialPacket）。
    let packet = chapter_context_service::render_context(&target_dir, chapter_id, view, budget)?;

    // 渲染输出（renderer 不感知 view 差异）。
    if json {
        let output = chapter_context_renderer::render_context_json(&packet);
        println!("{}", output);
    } else {
        let output = chapter_context_renderer::render_context_human(&packet);
        println!("{}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_chapter_command() {
        let cmd = build();
        assert_eq!(cmd.get_name(), "chapter");
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "create"));
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "update"));
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "get"));
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "list"));
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "reimport"));
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "status"));
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "context"));
    }
}
