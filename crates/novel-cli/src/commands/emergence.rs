// `novel emergence` 命令：涌现管理命令。

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::errors::CliError;
use crate::renderers::emergence_renderer;
use crate::services::emergence_service;

pub fn build() -> Command {
    Command::new("emergence")
        .about("涌现管理命令")
        .subcommand_required(true)
        .subcommand(
            Command::new("create")
                .about("创建涌现")
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
                        .help("涌现标题"),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .required(true)
                        .value_name("TEXT")
                        .help("涌现描述"),
                )
                .arg(
                    Arg::new("chapter")
                        .long("chapter")
                        .value_name("CHAPTER_ID")
                        .help("发现涌现的章节 ID（可选）"),
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
            Command::new("rate")
                .about("评分涌现（pending → rated）")
                .arg(
                    Arg::new("emergence-id")
                        .required(true)
                        .value_name("EMERGENCE_ID")
                        .help("涌现 ID"),
                )
                .arg(
                    Arg::new("rating")
                        .long("rating")
                        .required(true)
                        .value_name("0-10")
                        .value_parser(clap::value_parser!(i64))
                        .help("评分（0-10）"),
                )
                .arg(
                    Arg::new("reason")
                        .long("reason")
                        .required(true)
                        .value_name("TEXT")
                        .help("评分理由"),
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
            Command::new("decide")
                .about("决策涌现（rated → decided）")
                .arg(
                    Arg::new("emergence-id")
                        .required(true)
                        .value_name("EMERGENCE_ID")
                        .help("涌现 ID"),
                )
                .arg(
                    Arg::new("decision")
                        .long("decision")
                        .required(true)
                        .value_name("accept|defer|reject")
                        .help("决策：accept、defer、reject"),
                )
                .arg(
                    Arg::new("reason")
                        .long("reason")
                        .required(true)
                        .value_name("TEXT")
                        .help("决策理由"),
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
            Command::new("execute")
                .about("执行涌现（decided → executed）")
                .arg(
                    Arg::new("emergence-id")
                        .required(true)
                        .value_name("EMERGENCE_ID")
                        .help("涌现 ID"),
                )
                .arg(
                    Arg::new("target")
                        .long("target")
                        .required(true)
                        .value_name("CHAPTER_ID")
                        .help("执行目标章节 ID"),
                )
                .arg(
                    Arg::new("note")
                        .long("note")
                        .value_name("TEXT")
                        .help("执行备注"),
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
                .about("归档涌现（executed → archived）")
                .arg(
                    Arg::new("emergence-id")
                        .required(true)
                        .value_name("EMERGENCE_ID")
                        .help("涌现 ID"),
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
                .about("列举卷内涌现")
                .arg(
                    Arg::new("volume-id")
                        .required(true)
                        .value_name("VOLUME_ID")
                        .value_parser(clap::value_parser!(i64))
                        .help("卷 ID"),
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .value_name("STATUS")
                        .help("状态过滤：pending、rated、decided、executed、archived"),
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
                .about("获取单个涌现详情")
                .arg(
                    Arg::new("emergence-id")
                        .required(true)
                        .value_name("EMERGENCE_ID")
                        .help("涌现 ID"),
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
}

pub fn handle(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => handle_create(sub_matches),
        Some(("rate", sub_matches)) => handle_rate(sub_matches),
        Some(("decide", sub_matches)) => handle_decide(sub_matches),
        Some(("execute", sub_matches)) => handle_execute(sub_matches),
        Some(("archive", sub_matches)) => handle_archive(sub_matches),
        Some(("list", sub_matches)) => handle_list(sub_matches),
        Some(("get", sub_matches)) => handle_get(sub_matches),
        _ => Err(CliError::usage(
            "未知 emergence 子命令。可用：create、rate、decide、execute、archive、list、get。运行 `novel emergence --help` 查看用法。",
        )),
    }
}

fn handle_create(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let title = matches.get_one::<String>("title").unwrap();
    let description = matches.get_one::<String>("description").unwrap();
    let chapter_id = matches.get_one::<String>("chapter").map(|s| s.as_str());
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergence = emergence_service::create(target, volume_id, title, description, chapter_id)?;

    if json {
        emergence_renderer::render_emergence_json(&emergence)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence(&emergence);
    }

    Ok(())
}

fn handle_rate(matches: &ArgMatches) -> Result<(), CliError> {
    let emergence_id = matches.get_one::<String>("emergence-id").unwrap();
    let rating = *matches.get_one::<i64>("rating").unwrap();
    let reason = matches.get_one::<String>("reason").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergence = emergence_service::rate(target, emergence_id, rating, reason)?;

    if json {
        emergence_renderer::render_emergence_json(&emergence)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence(&emergence);
    }

    Ok(())
}

fn handle_decide(matches: &ArgMatches) -> Result<(), CliError> {
    let emergence_id = matches.get_one::<String>("emergence-id").unwrap();
    let decision = matches.get_one::<String>("decision").unwrap();
    let reason = matches.get_one::<String>("reason").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergence = emergence_service::decide(target, emergence_id, decision, reason)?;

    if json {
        emergence_renderer::render_emergence_json(&emergence)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence(&emergence);
    }

    Ok(())
}

fn handle_execute(matches: &ArgMatches) -> Result<(), CliError> {
    let emergence_id = matches.get_one::<String>("emergence-id").unwrap();
    let target_chapter = matches.get_one::<String>("target").unwrap();
    let note = matches.get_one::<String>("note").map(|s| s.as_str());
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergence = emergence_service::execute(target, emergence_id, target_chapter, note)?;

    if json {
        emergence_renderer::render_emergence_json(&emergence)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence(&emergence);
    }

    Ok(())
}

fn handle_archive(matches: &ArgMatches) -> Result<(), CliError> {
    let emergence_id = matches.get_one::<String>("emergence-id").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergence = emergence_service::archive(target, emergence_id)?;

    if json {
        emergence_renderer::render_emergence_json(&emergence)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence(&emergence);
    }

    Ok(())
}

fn handle_list(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let status_filter = matches.get_one::<String>("status").map(|s| s.as_str());
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergences = emergence_service::list(target, volume_id, status_filter)?;

    if json {
        emergence_renderer::render_emergence_list_json(&emergences)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence_list(&emergences);
    }

    Ok(())
}

fn handle_get(matches: &ArgMatches) -> Result<(), CliError> {
    let emergence_id = matches.get_one::<String>("emergence-id").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let emergence = emergence_service::get(target, emergence_id)?;

    if json {
        emergence_renderer::render_emergence_json(&emergence)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        emergence_renderer::render_emergence(&emergence);
    }

    Ok(())
}
