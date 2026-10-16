// `novel verdict` 命令：主线落地裁决的参数解析、基础校验、调用 service、选择输出格式。
//
// 职责（CLAUDE.md §6）：command 层只做参数解析与基础校验，不实现业务规则，不直接访问 storage。
// `--achieved` 取值由 value_parser 限定为 0/1（非法值退出码 2）。

use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::errors::CliError;
use crate::renderers::verdict_renderer;
use crate::services::verdict_service;

/// 构建 `novel verdict` 子命令。
pub fn build() -> Command {
    Command::new("verdict")
        .about("主线落地裁决管理命令")
        .subcommand(
            Command::new("set-mainplot-landing")
                .about("记录主线落地裁决")
                .arg(
                    Arg::new("chapter-id")
                        .required(true)
                        .value_name("CHAPTER-ID")
                        .help("章节 ID（如 chapter-001）"),
                )
                .arg(
                    Arg::new("node-id")
                        .long("node-id")
                        .required(true)
                        .value_name("ID")
                        .help("主线节点 ID"),
                )
                .arg(
                    Arg::new("achieved")
                        .long("achieved")
                        .required(true)
                        .value_name("0|1")
                        .help("0=未达成，1=已达成")
                        .value_parser(["0", "1"]),
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
}

/// 分发 verdict 子命令。
pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("set-mainplot-landing", sub)) => run_set_mainplot_landing(sub),
        _ => Err(CliError::usage(
            "请指定 verdict 子命令：set-mainplot-landing。运行 `novel verdict --help` 查看用法。",
        )),
    }
}

/// `novel verdict set-mainplot-landing <chapter-id> --node-id <id> --achieved <0|1>`
fn run_set_mainplot_landing(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("chapter-id")
        .map(String::as_str)
        .unwrap_or("");
    let node_id = matches
        .get_one::<String>("node-id")
        .ok_or_else(|| CliError::usage("缺少参数: --node-id"))?;
    let achieved_str = matches
        .get_one::<String>("achieved")
        .map(String::as_str)
        .unwrap_or("");
    let path = matches.get_one::<String>("path").map(String::as_str);
    let json = matches.get_flag("json");

    // value_parser 已限定 achieved ∈ {0,1}，此处解析为 bool。
    let achieved = achieved_str == "1";

    // 解析目标目录
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

    // 调用 service 层写入裁决
    let verdict = verdict_service::set_mainplot_landing(&target_dir, chapter_id, node_id, achieved)?;

    // 渲染输出
    if json {
        let output = verdict_renderer::render_verdict_json(&verdict);
        println!("{}", output);
    } else {
        let output = verdict_renderer::render_verdict_human(&verdict);
        println!("{}", output);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_verdict_command() {
        let cmd = build();
        assert_eq!(cmd.get_name(), "verdict");
        assert!(cmd.get_subcommands().any(|s| s.get_name() == "set-mainplot-landing"));
    }
}
