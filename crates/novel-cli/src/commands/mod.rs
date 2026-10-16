// commands 层：参数解析、基础参数校验与输出选择。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §4）：命令名称/参数/选项与契约一致；参数缺失、
// 枚举非法、路径参数非法等基础错误返回退出码 2；写命令只调用对应 service/use-case，
// 不直接修改数据；查询类默认人读输出，需机器消费时支持 `--json`；JSON 模式不输出 ANSI/
// 旁白；command 层不实现状态机和权限矩阵，不绕过 service 直接调 repository/storage。

pub mod chapter;
pub mod constraint;
pub mod emergence;
pub mod export;
pub mod info;
pub mod init;
pub mod mainplot;
pub mod setting;
pub mod thread;
pub mod verify;
pub mod verdict;
pub mod volume;

use clap::{ArgMatches, Command};

/// 构建 CLI parser。命令语法对应 contracts/cli/commands.md。
pub fn build_cli() -> Command {
    Command::new("novel")
        .bin_name("novel")
        .version("0.1.0")
        .about("novel CLI: 世界状态日志的硬层内核与唯一入口")
        .subcommand(init::build())
        .subcommand(info::build())
        .subcommand(setting::build())
        .subcommand(constraint::build())
        .subcommand(chapter::build())
        .subcommand(verdict::build())
        .subcommand(volume::build())
        .subcommand(mainplot::build())
        .subcommand(thread::build())
        .subcommand(emergence::build())
        .subcommand(export::build())
        .subcommand(verify::build())
}

/// 分发到具体命令，返回领域结果。
pub fn dispatch(matches: &ArgMatches) -> Result<(), crate::errors::CliError> {
    match matches.subcommand() {
        Some(("init", sub)) => init::run(sub),
        Some(("info", sub)) => info::run(sub),
        Some(("setting", sub)) => setting::run(sub),
        Some(("constraint", sub)) => constraint::run(sub),
        Some(("chapter", sub)) => chapter::run(sub),
        Some(("verdict", sub)) => verdict::run(sub),
        Some(("volume", sub)) => volume::run(sub),
        Some(("mainplot", sub)) => mainplot::handle(sub),
        Some(("thread", sub)) => thread::handle(sub),
        Some(("emergence", sub)) => emergence::handle(sub),
        Some(("export", sub)) => export::handle(sub),
        Some(("verify", sub)) => verify::handle(sub),
        _ => {
            // 未提供子命令：视为用法错误。
            Err(crate::errors::CliError::usage(
                "请指定子命令。可用：init, info, setting, constraint, chapter, verdict, volume, mainplot, thread, emergence, export, verify。运行 `novel --help` 查看用法。",
            ))
        }
    }
}
