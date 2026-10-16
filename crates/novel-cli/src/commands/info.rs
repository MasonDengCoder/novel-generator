// `novel info` 命令：参数解析，委托 service，选择输出格式。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §4）：command 层只做参数解析与输出选择，
// 不实现业务规则，不直接访问 storage。

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::errors::CliError;
use crate::renderers;
use crate::services;

pub fn build() -> Command {
    Command::new("info")
        .about("输出当前世界的基本信息与数据概况")
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
        )
}

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let path = matches.get_one::<String>("path").map(String::as_str);
    let json = matches.get_flag("json");

    let target = services::info::resolve_target(path)?;
    let info = services::info::read_info(&target)?;

    renderers::info::render(&info, json);
    Ok(())
}
