// `novel thread` 命令：线索管理命令。

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::Path;

use crate::errors::CliError;
use crate::renderers::thread_renderer;
use crate::services::thread_service;

pub fn build() -> Command {
    Command::new("thread")
        .about("线索管理命令")
        .subcommand_required(true)
        .subcommand(
            Command::new("create")
                .about("创建线索")
                .arg(
                    Arg::new("volume-id")
                        .required(true)
                        .value_name("VOLUME_ID")
                        .value_parser(clap::value_parser!(i64))
                        .help("卷 ID"),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .required(true)
                        .value_name("NAME")
                        .help("线索名称"),
                )
                .arg(
                    Arg::new("description")
                        .long("description")
                        .value_name("TEXT")
                        .help("线索描述"),
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
            Command::new("bind")
                .about("绑定线索到章节")
                .arg(
                    Arg::new("thread-id")
                        .required(true)
                        .value_name("THREAD_ID")
                        .help("线索 ID"),
                )
                .arg(
                    Arg::new("chapter")
                        .long("chapter")
                        .required(true)
                        .value_name("CHAPTER_ID")
                        .help("章节 ID"),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .value_name("TYPE")
                        .help("绑定类型：mention、develop、resolve（默认：mention）"),
                )
                .arg(
                    Arg::new("note")
                        .long("note")
                        .value_name("TEXT")
                        .help("绑定备注"),
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
                .about("列举卷内所有线索")
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
            Command::new("query")
                .about("查询章节的所有线索绑定")
                .arg(
                    Arg::new("chapter")
                        .long("chapter")
                        .required(true)
                        .value_name("CHAPTER_ID")
                        .help("章节 ID"),
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
                .about("更新线索状态")
                .arg(
                    Arg::new("thread-id")
                        .required(true)
                        .value_name("THREAD_ID")
                        .help("线索 ID"),
                )
                .arg(
                    Arg::new("status")
                        .long("status")
                        .required(true)
                        .value_name("STATUS")
                        .help("线索状态：active、paused、resolved"),
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
        Some(("bind", sub_matches)) => handle_bind(sub_matches),
        Some(("list", sub_matches)) => handle_list(sub_matches),
        Some(("query", sub_matches)) => handle_query(sub_matches),
        Some(("update", sub_matches)) => handle_update(sub_matches),
        _ => Err(CliError::usage(
            "未知 thread 子命令。可用：create、bind、list、query、update。运行 `novel thread --help` 查看用法。",
        )),
    }
}

fn handle_create(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let name = matches.get_one::<String>("name").unwrap();
    let description = matches.get_one::<String>("description").map(|s| s.as_str());
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let thread = thread_service::create_thread(target, volume_id, name, description)?;

    if json {
        thread_renderer::render_thread_json(&thread)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        thread_renderer::render_thread(&thread);
    }

    Ok(())
}

fn handle_bind(matches: &ArgMatches) -> Result<(), CliError> {
    let thread_id = matches.get_one::<String>("thread-id").unwrap();
    let chapter_id = matches.get_one::<String>("chapter").unwrap();
    let binding_type = matches.get_one::<String>("type").map(|s| s.as_str());
    let note = matches.get_one::<String>("note").map(|s| s.as_str());
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    thread_service::bind_chapter(target, thread_id, chapter_id, binding_type, note)?;

    thread_renderer::render_bind_success(thread_id, chapter_id);

    Ok(())
}

fn handle_list(matches: &ArgMatches) -> Result<(), CliError> {
    let volume_id = *matches.get_one::<i64>("volume-id").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let threads = thread_service::list_threads(target, volume_id)?;

    if json {
        thread_renderer::render_thread_list_json(&threads)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        thread_renderer::render_thread_list(&threads);
    }

    Ok(())
}

fn handle_query(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches.get_one::<String>("chapter").unwrap();
    let json = matches.get_flag("json");
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    let bindings = thread_service::query_by_chapter(target, chapter_id)?;

    if json {
        thread_renderer::render_binding_list_json(&bindings)
            .map_err(|e| CliError::storage(format!("JSON 序列化失败：{}", e)))?;
    } else {
        thread_renderer::render_binding_list(&bindings);
    }

    Ok(())
}

fn handle_update(matches: &ArgMatches) -> Result<(), CliError> {
    let thread_id = matches.get_one::<String>("thread-id").unwrap();
    let status = matches.get_one::<String>("status").unwrap();
    let path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = Path::new(path);

    thread_service::update_status(target, thread_id, status)?;

    thread_renderer::render_update_success(thread_id, status);

    Ok(())
}
