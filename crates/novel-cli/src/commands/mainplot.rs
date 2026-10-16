// `novel mainplot` 命令：主线节点管理命令。

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::errors::CliError;
use crate::renderers::mainplot_renderer;
use crate::services::mainplot_service;

pub fn build() -> Command {
    Command::new("mainplot")
        .about("主线节点管理命令")
        .subcommand_required(true)
        .subcommand(
            Command::new("node")
                .about("主线节点操作")
                .subcommand_required(true)
                .subcommand(
                    Command::new("create")
                        .about("创建主线节点")
                        .arg(
                            Arg::new("volume-id")
                                .required(true)
                                .value_name("VOLUME_ID")
                                .value_parser(clap::value_parser!(i64))
                                .help("卷 ID"),
                        )
                        .arg(
                            Arg::new("title")
                                .long("title")
                                .required(true)
                                .value_name("TEXT")
                                .help("主线节点标题"),
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .value_name("TEXT")
                                .help("主线节点描述"),
                        )
                        .arg(
                            Arg::new("target-chapters")
                                .long("target-chapters")
                                .value_name("IDS")
                                .help("目标章节 ID 列表，逗号分隔（如：ch-001,ch-002）"),
                        )
                        .arg(
                            Arg::new("json")
                                .long("json")
                                .action(ArgAction::SetTrue)
                                .help("输出 JSON 格式"),
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
                        .about("列举卷内所有主线节点")
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
                                .help("输出 JSON 格式"),
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
                        .about("获取单个主线节点详情")
                        .arg(
                            Arg::new("node-id")
                                .required(true)
                                .value_name("NODE_ID")
                                .help("主线节点 ID"),
                        )
                        .arg(
                            Arg::new("json")
                                .long("json")
                                .action(ArgAction::SetTrue)
                                .help("输出 JSON 格式"),
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
                        .about("更新主线节点达成状态")
                        .arg(
                            Arg::new("node-id")
                                .required(true)
                                .value_name("NODE_ID")
                                .help("主线节点 ID"),
                        )
                        .arg(
                            Arg::new("achieved")
                                .long("achieved")
                                .required(true)
                                .value_name("0|1")
                                .value_parser(["0", "1"])
                                .help("达成状态：0=未达成，1=已达成"),
                        )
                        .arg(
                            Arg::new("path")
                                .long("path")
                                .value_name("DIR")
                                .help("世界所在目录，默认当前工作目录"),
                        ),
                ),
        )
}

pub fn handle(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("node", node_matches)) => handle_node(node_matches),
        _ => Err(CliError::usage(
            "未知 mainplot 子命令。可用：node。运行 `novel mainplot --help` 查看用法。",
        )),
    }
}

fn handle_node(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => handle_node_create(sub_matches),
        Some(("list", sub_matches)) => handle_node_list(sub_matches),
        Some(("get", sub_matches)) => handle_node_get(sub_matches),
        Some(("update", sub_matches)) => handle_node_update(sub_matches),
        _ => Err(CliError::usage(
            "未知 mainplot node 子命令。可用：create、list、get、update。运行 `novel mainplot node --help` 查看用法。",
        )),
    }
}

fn handle_node_create(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let title = matches.get_one::<String>("title").unwrap();
    let description = matches.get_one::<String>("description").map(|s| s.as_str());
    let target_chapters = matches
        .get_one::<String>("target-chapters")
        .map(|s| s.split(',').map(|id| id.trim().to_string()).collect());
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let node = mainplot_service::create_node(target, volume_id, title, description, target_chapters)?;

    if json {
        mainplot_renderer::render_node_json(&node)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        mainplot_renderer::render_node(&node);
    }

    Ok(())
}

fn handle_node_list(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let nodes = mainplot_service::list_nodes(target, volume_id)?;

    if json {
        mainplot_renderer::render_node_list_json(&nodes)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        mainplot_renderer::render_node_list(&nodes);
    }

    Ok(())
}

fn handle_node_get(matches: &ArgMatches) -> Result<(), CliError> {
    let node_id = matches.get_one::<String>("node-id").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let node = mainplot_service::get_node(target, node_id)?;

    if json {
        mainplot_renderer::render_node_json(&node)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        mainplot_renderer::render_node(&node);
    }

    Ok(())
}

fn handle_node_update(matches: &ArgMatches) -> Result<(), CliError> {
    let node_id = matches.get_one::<String>("node-id").unwrap();
    // value_parser 已限定 achieved ∈ {"0","1"}，转 i64 传入 service（保持 service/repository 签名稳定）。
    let achieved_str = matches.get_one::<String>("achieved").map(String::as_str).unwrap_or("0");
    let achieved: i64 = if achieved_str == "1" { 1 } else { 0 };
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    mainplot_service::update_node_achieved(target, node_id, achieved)?;

    mainplot_renderer::render_node_update_success(node_id);

    Ok(())
}
