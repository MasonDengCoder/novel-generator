// commands/constraint.rs
// s01-project-bootstrap：constraint 命令的参数解析与输出选择。
//
// 职责（CLAUDE.md §6）：参数解析、基础校验（参数缺失/枚举非法返回退出码 2）、
// 调用 service 层用例、选择渲染器输出。不实现业务逻辑。

use clap::{Arg, ArgMatches, Command};

use crate::domain::{ConstraintScope, ConstraintSeverity, ConstraintType, ConstraintView};
use crate::errors::CliError;
use crate::renderers::constraint_renderer;
use crate::services::constraint_service;

/// 构建 `novel constraint` 子命令。
pub fn build() -> Command {
    Command::new("constraint")
        .about("管理创作约束规则")
        .subcommand(
            Command::new("define")
                .about("定义新约束规则")
                .arg(
                    Arg::new("type")
                        .long("type")
                        .help("约束类型（plot, character, world, style, custom）")
                        .required(true),
                )
                .arg(
                    Arg::new("scope")
                        .long("scope")
                        .help("作用域（如 world, volume:v1, chapter:c1）")
                        .required(true),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .help("约束名称")
                        .required(true),
                )
                .arg(
                    Arg::new("content")
                        .long("content")
                        .help("约束内容（纯文本）")
                        .required(true),
                )
                .arg(
                    Arg::new("severity")
                        .long("severity")
                        .help("严重等级（info/warning/hard）")
                        .required(true),
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
                .about("查询约束规则（单条精确查询或按 view/scope 批量渲染）")
                .arg(
                    Arg::new("type")
                        .long("type")
                        .help("约束类型（单条精确查询形态）"),
                )
                .arg(
                    Arg::new("scope")
                        .long("scope")
                        .help("作用域：单条形态为自由字符串（如 world/volume:v1/chapter:c1）；受控形态为 world|volume|chapter 枚举（与 --view 同现时为受控形态）"),
                )
                .arg(
                    Arg::new("name")
                        .long("name")
                        .help("约束名称（单条精确查询形态）"),
                )
                .arg(
                    Arg::new("view")
                        .long("view")
                        .help("受控视图名（受控批量形态）：llm|human。出现 --view 即走受控形态分支，与 --type/--name 互斥"),
                )
                .arg(
                    Arg::new("chapter-id")
                        .long("chapter-id")
                        .help("章节 ID（受控形态 --scope chapter 必填；--scope volume 可用于取章节所属卷）"),
                )
                .arg(
                    Arg::new("volume-id")
                        .long("volume-id")
                        .help("卷 ID 整数（受控形态 --scope volume 定位卷）")
                        .value_parser(clap::value_parser!(i64)),
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
                .about("列出约束规则")
                .arg(
                    Arg::new("scope")
                        .long("scope")
                        .help("按 scope 前缀过滤（如 volume: 匹配所有卷级约束）")
                        .required(false),
                )
                .arg(
                    Arg::new("severity")
                        .long("severity")
                        .help("按 severity 过滤（info/warning/hard）")
                        .required(false),
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
            Command::new("check")
                .about("规则检查结果管理")
                .arg(
                    Arg::new("chapter")
                        .long("chapter")
                        .help("章节 ID")
                        .required(true),
                )
                .arg(
                    Arg::new("input")
                        .long("input")
                        .value_name("FILE")
                        .help("输入 JSON 文件（批量写入检查结果）")
                        .conflicts_with("list"),
                )
                .arg(
                    Arg::new("list")
                        .long("list")
                        .help("查询检查结果")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verify-hash")
                        .long("verify-hash")
                        .help("强制验证检查结果的 content_hash 与当前章节一致")
                        .action(clap::ArgAction::SetTrue)
                        .requires("list"),
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .help("输出 JSON 格式")
                        .action(clap::ArgAction::SetTrue)
                        .requires("list"),
                )
                .arg(
                    Arg::new("path")
                        .long("path")
                        .value_name("DIR")
                        .help("世界所在目录，默认当前工作目录"),
                ),
        )
}

/// 分发 constraint 子命令。
pub fn run(matches: &ArgMatches) -> Result<(), CliError> {
    match matches.subcommand() {
        Some(("define", sub)) => run_define(sub),
        Some(("render", sub)) => run_render(sub),
        Some(("list", sub)) => run_list(sub),
        Some(("check", sub)) => run_check(sub),
        _ => Err(CliError::usage(
            "请指定子命令：define, render, list, check。运行 `novel constraint --help` 查看用法。",
        )),
    }
}

/// `novel constraint define --type <type> --scope <scope> --name <name> --content <content> --severity <severity>`
fn run_define(matches: &ArgMatches) -> Result<(), CliError> {
    let type_str = matches
        .get_one::<String>("type")
        .ok_or_else(|| CliError::usage("缺少参数: --type"))?;
    let scope = matches
        .get_one::<String>("scope")
        .ok_or_else(|| CliError::usage("缺少参数: --scope"))?;
    let name = matches
        .get_one::<String>("name")
        .ok_or_else(|| CliError::usage("缺少参数: --name"))?;
    let content = matches
        .get_one::<String>("content")
        .ok_or_else(|| CliError::usage("缺少参数: --content"))?;
    let severity_str = matches
        .get_one::<String>("severity")
        .ok_or_else(|| CliError::usage("缺少参数: --severity"))?;
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 基础校验：解析枚举
    let constraint_type = type_str
        .parse::<ConstraintType>()
        .map_err(|e| CliError::usage(e))?;
    let severity = severity_str
        .parse::<ConstraintSeverity>()
        .map_err(|e| CliError::usage(e))?;

    let target = constraint_service::resolve_target(path)?;

    // 调用 service 层
    constraint_service::define_constraint(&target, constraint_type, scope, name, content, severity)?;

    // 成功无输出（静默成功）
    Ok(())
}

/// `novel constraint render`：双形态路由。
///
/// - 受控批量形态：出现 `--view` → 需 `--view <llm|human>` + `--scope <world|volume|chapter>`，
///   与 `--type`/`--name` 互斥；`--scope chapter` 需 `--chapter-id`，`--scope volume` 需
///   `--volume-id` 或 `--chapter-id` 之一。
/// - 单条精确形态（s01 行为不变）：无 `--view` → 需 `--type`/`--scope`/`--name` 三者。
fn run_render(matches: &ArgMatches) -> Result<(), CliError> {
    let view_str = matches.get_one::<String>("view");
    let type_str = matches.get_one::<String>("type");
    let scope_str = matches.get_one::<String>("scope");
    let name = matches.get_one::<String>("name");
    let chapter_id = matches.get_one::<String>("chapter-id").cloned();
    let volume_id = matches.get_one::<i64>("volume-id").copied();
    let json_mode = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    if view_str.is_some() {
        // ===== 受控批量形态分支 =====
        // 互斥校验：--view 形态不可与 --type/--name 同现
        if type_str.is_some() || name.is_some() {
            return Err(CliError::usage(
                "--view 受控形态与 --type/--name 单条形态不可同时使用。",
            ));
        }
        // 受控形态必须同时有 --view 与 --scope
        let view = view_str
            .unwrap()
            .parse::<ConstraintView>()
            .map_err(|e| CliError::usage(e))?;
        let scope = scope_str
            .ok_or_else(|| {
                CliError::usage("受控形态必须同时提供 --view 与 --scope。")
            })?
            .parse::<ConstraintScope>()
            .map_err(|e| CliError::usage(e))?;

        // scope 定位校验
        match scope {
            ConstraintScope::Chapter => {
                if chapter_id.is_none() {
                    return Err(CliError::usage(
                        "--scope chapter 必须配合 --chapter-id。",
                    ));
                }
            }
            ConstraintScope::Volume => {
                if volume_id.is_none() && chapter_id.is_none() {
                    return Err(CliError::usage(
                        "--scope volume 必须配合 --volume-id 或 --chapter-id。",
                    ));
                }
            }
            ConstraintScope::World => {}
        }

        let target = constraint_service::resolve_target(path)?;
        let view_target = constraint_service::ConstraintViewTarget {
            view,
            scope,
            chapter_id,
            volume_id,
        };
        let result = constraint_service::render_view(&target, view_target)?;

        if json_mode {
            constraint_renderer::render_constraint_view_json(&result);
        } else {
            constraint_renderer::render_constraint_view(&result);
        }
        Ok(())
    } else {
        // ===== 单条精确形态分支（s01 行为不变）=====
        let type_str = type_str.ok_or_else(|| CliError::usage("缺少参数: --type"))?;
        let scope = scope_str.ok_or_else(|| CliError::usage("缺少参数: --scope"))?;
        let name = name.ok_or_else(|| CliError::usage("缺少参数: --name"))?;

        let constraint_type = type_str
            .parse::<ConstraintType>()
            .map_err(|e| CliError::usage(e))?;

        let target = constraint_service::resolve_target(path)?;
        let (constraint, schema_version) =
            constraint_service::render_constraint(&target, constraint_type, &scope, &name)?;

        if json_mode {
            constraint_renderer::render_constraint_json(&constraint, &schema_version);
        } else {
            constraint_renderer::render_constraint(&constraint);
        }
        Ok(())
    }
}

/// `novel constraint list [--scope <prefix>] [--severity <severity>] [--json]`
fn run_list(matches: &ArgMatches) -> Result<(), CliError> {
    let scope_filter = matches.get_one::<String>("scope").map(|s| s.as_str());
    let severity_str = matches.get_one::<String>("severity");
    let json_mode = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    // 基础校验：解析 severity 枚举（如果提供）
    let severity_filter = if let Some(s) = severity_str {
        Some(s.parse::<ConstraintSeverity>().map_err(|e| CliError::usage(e))?)
    } else {
        None
    };

    let target = constraint_service::resolve_target(path)?;

    // 调用 service 层
    let (constraints, schema_version) =
        constraint_service::list_constraints_filtered(&target, scope_filter, severity_filter)?;

    // 选择渲染器
    if json_mode {
        constraint_renderer::render_constraint_list_json(&constraints, &schema_version);
    } else {
        constraint_renderer::render_constraint_list(&constraints);
    }

    Ok(())
}

/// `novel constraint check --chapter <id> [--input <file> | --list [--verify-hash] [--json]]`
fn run_check(matches: &ArgMatches) -> Result<(), CliError> {
    let chapter_id = matches
        .get_one::<String>("chapter")
        .ok_or_else(|| CliError::usage("缺少参数: --chapter"))?;
    let input_file = matches.get_one::<String>("input");
    let list_mode = matches.get_flag("list");
    let verify_hash = matches.get_flag("verify-hash");
    let json_mode = matches.get_flag("json");
    let path = matches.get_one::<String>("path").map(String::as_str);

    let target = constraint_service::resolve_target(path)?;

    if list_mode {
        // 查询检查结果
        let (checks, current_hash, schema_version) =
            constraint_service::list_checks(&target, chapter_id, verify_hash)?;

        // 选择渲染器
        constraint_renderer::render_check_results(&checks, &current_hash, &schema_version, json_mode);
    } else if let Some(input_path_str) = input_file {
        // 批量写入检查结果
        let input_path = std::path::Path::new(input_path_str);
        if !input_path.exists() {
            return Err(CliError::invalid_input(
                format!("输入文件不存在: {}", input_path.display()),
                Some("检查 --input 参数路径是否正确。".to_string()),
            ));
        }

        constraint_service::batch_write_checks(&target, chapter_id, input_path)?;

        // 成功时输出（JSON 模式下输出结构化成功对象，人读模式静默）
        if json_mode {
            constraint_renderer::render_check_success(true);
        }
    } else {
        return Err(CliError::usage(
            "请指定 --input <file> 写入检查结果，或使用 --list 查询检查结果。",
        ));
    }

    Ok(())
}
