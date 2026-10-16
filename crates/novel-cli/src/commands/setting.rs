// commands/setting.rs
// s01-project-bootstrap：setting 命令的参数解析与输出选择。
//
// 职责（CLAUDE.md §6）：参数解析、基础校验（参数缺失/非法返回退出码 2）、
// 调用 service 层用例、选择渲染器输出。不实现业务逻辑。

use clap::{Arg, ArgMatches, Command};

use crate::domain::SettingView;
use crate::errors::CliError;
use crate::renderers::setting_renderer;
use crate::services::setting_service;

/// 构建 `novel setting` 子命令。
pub fn build() -> Command {
    Command::new("setting")
        .about("管理项目配置项")
        .subcommand(
            Command::new("set")
                .about("设置配置项（UPSERT 语义）")
                .arg(
                    Arg::new("name")
                        .help("配置项名称")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("value")
                        .help("配置项值")
                        .required(true)
                        .index(2),
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
                .about("查询单个配置项")
                .arg(
                    Arg::new("name")
                        .help("配置项名称")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .help("输出 JSON 格式")
                        .action(clap::ArgAction::SetTrue),
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
                .about("列出所有配置项")
                .arg(
                    Arg::new("json")
                        .long("json")
                        .help("输出 JSON 格式")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("render")
                .about("按受控 view 渲染设定注入视图（只读）")
                .arg(
                    Arg::new("view")
                        .long("view")
                        .help("受控视图名：intro|four-poles|world|style|summary|full|llm")
                        .required(true),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .help("输出 JSON 格式")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
}

/// 分发 setting 子命令。
pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("set", sub)) => run_set(sub),
        Some(("get", sub)) => run_get(sub),
        Some(("list", sub)) => run_list(sub),
        Some(("render", sub)) => run_render(sub),
        _ => Err(CliError::usage(
            "请指定子命令：set, get, list, render。运行 `novel setting --help` 查看用法。",
        )),
    }
}

/// `novel setting set <name> <value>`
fn run_set(matches: &ArgMatches) -> Result<(), CliError> {
    let name = matches
        .get_one::<String>("name")
        .ok_or_else(|| CliError::usage("缺少参数: <name>"))?;
    let value = matches
        .get_one::<String>("value")
        .ok_or_else(|| CliError::usage("缺少参数: <value>"))?;
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 基础校验：name 不能为空
    if name.trim().is_empty() {
        return Err(CliError::usage("配置项名称不能为空"));
    }

    let target = setting_service::resolve_target(path)?;

    // 调用 service 层
    setting_service::set_setting(&target, name, value)?;

    // 成功无输出（静默成功）
    Ok(())
}

/// `novel setting get <name> [--json]`
fn run_get(matches: &ArgMatches) -> Result<(), CliError> {
    let name = matches
        .get_one::<String>("name")
        .ok_or_else(|| CliError::usage("缺少参数: <name>"))?;
    let json_mode = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = setting_service::resolve_target(path)?;

    // 调用 service 层
    let (setting, schema_version) = setting_service::get_setting(&target, name)?;

    // 选择渲染器
    if json_mode {
        setting_renderer::render_setting_json(&setting, &schema_version);
    } else {
        setting_renderer::render_setting(&setting);
    }

    Ok(())
}

/// `novel setting list [--json]`
fn run_list(matches: &ArgMatches) -> Result<(), CliError> {
    let json_mode = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = setting_service::resolve_target(path)?;

    // 调用 service 层
    let (settings, schema_version) = setting_service::list_settings(&target)?;

    // 选择渲染器
    if json_mode {
        setting_renderer::render_setting_list_json(&settings, &schema_version);
    } else {
        setting_renderer::render_setting_list(&settings);
    }

    Ok(())
}

/// `novel setting render --view <受控名> [--json]`（只读）
fn run_render(matches: &ArgMatches) -> Result<(), CliError> {
    let view_str = matches
        .get_one::<String>("view")
        .ok_or_else(|| CliError::usage("缺少参数: --view"))?;
    let json_mode = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 基础校验：受控 view 名枚举
    let view = view_str
        .parse::<SettingView>()
        .map_err(|e| CliError::usage(e))?;

    let target = setting_service::resolve_target(path)?;

    // 调用 service 层（只读）
    let (settings, schema_version) = setting_service::render_view(&target, view)?;

    // 选择渲染器
    if json_mode {
        setting_renderer::render_setting_view_json(&settings, view, &schema_version);
    } else {
        setting_renderer::render_setting_view(&settings, view);
    }

    Ok(())
}
