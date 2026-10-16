// `novel init` 命令：参数解析、基础校验、调用 service、选择输出格式。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §4）：command 层只做参数解析与基础校验，
// 不实现业务规则，不直接访问 storage。写命令只调用对应 service/use-case。

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::errors::CliError;
use crate::renderers;
use crate::services;

pub fn build() -> Command {
    Command::new("init")
        .about("在目标目录创建 .novel/ 世界结构与 SQLite 元数据")
        .arg(
            Arg::new("name")
                .required(true)
                .value_name("NAME")
                .help("项目/世界名称，不得为空且不含路径分隔符"),
        )
        .arg(
            Arg::new("path")
                .long("path")
                .value_name("DIR")
                .help("世界落点目录，默认当前工作目录"),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .help("输出稳定结构化 JSON"),
        )
}

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    let name = matches.get_one::<String>("name").map(String::as_str).unwrap_or("");
    let path = matches.get_one::<String>("path").map(String::as_str);
    let json = matches.get_flag("json");

    // 基础校验委托 validators；深层冲突检测在 service 内（含事务一致性）。
    let target = services::init::resolve_target(path)?;
    let output = services::init::create_world(name, &target)?;

    renderers::init::render_success(&output, json);
    Ok(())
}
