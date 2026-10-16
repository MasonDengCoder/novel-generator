// `novel volume` 命令：卷级管理命令。

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::errors::CliError;
use crate::renderers::volume_renderer;
use crate::services::volume_service;

pub fn build() -> Command {
    Command::new("volume")
        .about("卷级管理命令")
        .subcommand_required(true)
        .subcommand(
            Command::new("create")
                .about("创建新卷")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .value_name("NAME")
                        .help("卷名称"),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .value_name("TEXT")
                        .help("卷描述"),
                )
                .arg(
                    Arg::new("order")
                        .long("order")
                        .value_name("N")
                        .value_parser(clap::value_parser!(i64))
                        .default_value("1")
                        .help("卷序号（用于排序）"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("查询单个卷信息")
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
            Command::new("list")
                .about("列举所有卷")
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
            Command::new("seal")
                .about("封卷（执行前置条件检查）")
                .arg(
                    Arg::new("volume-id")
                        .required(true)
                        .value_name("VOLUME_ID")
                        .value_parser(clap::value_parser!(i64))
                        .help("卷 ID"),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .action(ArgAction::SetTrue)
                        .help("干运行：只检查前置条件，不修改卷状态"),
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
            Command::new("archive")
                .about("归档卷（sealing/sealed 状态推进到 archived，生成卷末监测报告）")
                .arg(
                    Arg::new("volume-id")
                        .required(true)
                        .value_name("VOLUME_ID")
                        .value_parser(clap::value_parser!(i64))
                        .help("卷 ID"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
        .subcommand(
            Command::new("goal")
                .about("卷目标管理")
                .subcommand_required(true)
                .subcommand(
                    Command::new("set")
                        .about("设置卷目标")
                        .arg(
                            Arg::new("volume-id")
                                .required(true)
                                .value_name("VOLUME_ID")
                                .value_parser(clap::value_parser!(i64))
                                .help("卷 ID"),
                        )
                        .arg(
                            Arg::new("mainplot-node-id")
                                .required(true)
                                .value_name("NODE_ID")
                                .help("主线节点 ID"),
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .value_name("TEXT")
                                .help("目标描述"),
                        )
                        .arg(
                            Arg::new("optional")
                                .long("optional")
                                .action(ArgAction::SetTrue)
                                .help("标记为可选目标（默认为必达）"),
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
                        .about("查询卷的所有目标")
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
                    Command::new("remove")
                        .about("删除卷目标")
                        .arg(
                            Arg::new("volume-id")
                                .required(true)
                                .value_name("VOLUME_ID")
                                .value_parser(clap::value_parser!(i64))
                                .help("卷 ID"),
                        )
                        .arg(
                            Arg::new("goal-id")
                                .required(true)
                                .value_name("GOAL_ID")
                                .value_parser(clap::value_parser!(i64))
                                .help("目标 ID"),
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

pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => run_create(sub_matches),
        Some(("info", sub_matches)) => run_info(sub_matches),
        Some(("list", sub_matches)) => run_list(sub_matches),
        Some(("seal", sub_matches)) => run_seal(sub_matches),
        Some(("archive", sub_matches)) => run_archive(sub_matches),
        Some(("goal", sub_matches)) => run_goal(sub_matches),
        _ => Err(CliError::usage("未知子命令".to_string())),
    }
}

fn run_goal(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("set", sub_matches)) => run_goal_set(sub_matches),
        Some(("get", sub_matches)) => run_goal_get(sub_matches),
        Some(("remove", sub_matches)) => run_goal_remove(sub_matches),
        _ => Err(CliError::usage("未知 goal 子命令".to_string())),
    }
}

fn run_create(matches: &ArgMatches) -> Result<(), CliError> {
    let name = matches.get_one::<String>("name").map(String::as_str).unwrap_or("");
    let description = matches.get_one::<String>("description").map(String::as_str);
    let order = matches.get_one::<i64>("order").copied().unwrap_or(1);
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 解析世界目录
    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    // 调用 service 创建卷
    let volume_id = volume_service::create_volume(target, name, description, order)?;

    volume_renderer::render_create_success(volume_id);
    Ok(())
}

fn run_info(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = matches.get_one::<i64>("volume-id").copied().unwrap_or(0);
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 解析世界目录
    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    // 查询卷信息
    let volume = volume_service::get_volume(target, volume_id)?;

    // 渲染输出
    if json {
        let output = volume_renderer::render_volume_info_json(&volume)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
        println!("{}", output);
    } else {
        let output = volume_renderer::render_volume_info(&volume);
        print!("{}", output);
    }

    Ok(())
}

fn run_list(matches: &ArgMatches) -> Result<(), CliError> {
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 解析世界目录
    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    // 查询卷列表
    let volumes = volume_service::list_volumes(target)?;

    // 渲染输出
    if json {
        let output = volume_renderer::render_volume_list_json(&volumes)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
        println!("{}", output);
    } else {
        let output = volume_renderer::render_volume_list(&volumes);
        print!("{}", output);
    }

    Ok(())
}

fn run_seal(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = matches.get_one::<i64>("volume-id").copied().unwrap_or(0);
    let dry_run = matches.get_flag("dry-run");
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    let check_results = volume_service::seal_volume(target, volume_id, dry_run)?;

    if json {
        let output = volume_renderer::render_seal_check_results_json(volume_id, dry_run, &check_results)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
        println!("{}", output);
    } else {
        let output = volume_renderer::render_seal_check_results(volume_id, dry_run, &check_results);
        print!("{}", output);
    }

    Ok(())
}

fn run_archive(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = matches.get_one::<i64>("volume-id").copied().unwrap_or(0);
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    volume_service::archive_volume(target, volume_id)?;

    // 人读成功消息（commands.md volume archive 行为语义；未声明 --json）。
    volume_renderer::render_archive_success(volume_id);
    Ok(())
}

fn run_goal_set(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = matches.get_one::<i64>("volume-id").copied().unwrap_or(0);
    let mainplot_node_id = matches.get_one::<String>("mainplot-node-id").map(String::as_str).unwrap_or("");
    let description = matches.get_one::<String>("description").map(String::as_str);
    let is_mandatory = !matches.get_flag("optional");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    let goal_id = volume_service::set_volume_goal(target, volume_id, mainplot_node_id, description, is_mandatory)?;

    volume_renderer::render_goal_set_success(goal_id);
    Ok(())
}

fn run_goal_get(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = matches.get_one::<i64>("volume-id").copied().unwrap_or(0);
    let json = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    let goals = volume_service::get_volume_goals(target, volume_id)?;

    if json {
        let output = volume_renderer::render_volume_goals_json(volume_id, &goals)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
        println!("{}", output);
    } else {
        let output = volume_renderer::render_volume_goals(&goals);
        print!("{}", output);
    }

    Ok(())
}

fn run_goal_remove(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = matches.get_one::<i64>("volume-id").copied().unwrap_or(0);
    let goal_id = matches.get_one::<i64>("goal-id").copied().unwrap_or(0);
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = path.map(Path::new).unwrap_or_else(|| Path::new("."));

    volume_service::remove_volume_goal(target, volume_id, goal_id)?;

    volume_renderer::render_goal_remove_success(goal_id);
    Ok(())
}
