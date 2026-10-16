// services/constraint_service.rs
// s01-project-bootstrap + s03-rule-checking-engine：constraint 相关业务用例。
//
// 职责（CLAUDE.md §6、§10）：执行状态机、权限矩阵、前置条件校验（如世界已初始化）；
// 编排 storage 层调用；hard 阻断返回非零退出码，不静默降级。

use std::path::{Path, PathBuf};
use time::OffsetDateTime;

use crate::domain::{Constraint, ConstraintScope, ConstraintSeverity, ConstraintType, ConstraintView};
use crate::errors::CliError;
use crate::repositories::rule_check_repository::{self, CheckResultInput, RuleCheck};
use crate::repositories::volume_repository;
use crate::storage::chapters::get_chapter_metadata;
use crate::storage::connection::{open_existing_at, read_schema_version};
use crate::storage::constraints::{get_constraint, insert_constraint, list_constraints, list_constraints_by_scopes};

/// 解析 constraint 相关命令的目标目录。
pub fn resolve_target(path: Option<&str>) -> Result<PathBuf, CliError> {
    let target = match path {
        Some(p) => std::path::absolute(p)
            .map_err(|e| CliError::usage(format!("解析 --path 失败: {e}")))?,
        None => std::env::current_dir()
            .map_err(|e| CliError::usage(format!("获取当前目录失败: {e}")))?,
    };
    if !target.is_dir() {
        return Err(CliError::usage(format!(
            "`--path` 指向的目录不存在: {}",
            target.display()
        )));
    }
    Ok(target)
}

/// define_constraint 用例：定义新约束规则。
///
/// 前置条件：世界已初始化。
/// 如果复合主键 (type, scope, name) 冲突则返回 ResourceConflict（退出码 4）。
pub fn define_constraint(
    target: &Path,
    constraint_type: ConstraintType,
    scope: &str,
    name: &str,
    content: &str,
    severity: ConstraintSeverity,
) -> Result<(), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;

    // 2. 生成时间戳
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间失败: {}", e)))?
        .to_string();

    // 3. 插入约束（storage 层会检查复合主键冲突）
    insert_constraint(&conn, constraint_type, scope, name, content, severity, &created_at)
        .map_err(|err| match err {
            CliError::ResourceConflict { .. } => CliError::constraint_already_exists(
                format!("约束 (type='{}', scope='{}', name='{}') 已存在，拒绝重复定义。", constraint_type, scope, name),
                Some("若需修改约束内容，请先删除旧约束（功能待实现）或使用不同的 name。".to_string()),
            ),
            other => other,
        })?;

    Ok(())
}

/// render_constraint 用例：查询单个约束规则。
///
/// 前置条件：世界已初始化。
/// 不存在则返回 ConstraintNotFound 错误（退出码 6）。
pub fn render_constraint(
    target: &Path,
    constraint_type: ConstraintType,
    scope: &str,
    name: &str,
) -> Result<(Constraint, String), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 查询 constraint
    let constraint = get_constraint(&conn, constraint_type, scope, name)?;

    // 3. 不存在则返回 NotFound 错误
    let constraint = constraint.ok_or_else(|| {
        CliError::constraint_not_found(
            format!(
                "约束 (type='{}', scope='{}', name='{}') 不存在。",
                constraint_type, scope, name
            ),
            Some("使用 `novel constraint list` 查看所有已定义的约束，或检查 type/scope/name 参数是否正确。".to_string()),
        )
    })?;

    Ok((constraint, schema_version))
}

/// list_constraints_filtered 用例：列出约束规则，支持过滤。
///
/// 前置条件：世界已初始化。
pub fn list_constraints_filtered(
    target: &Path,
    scope_filter: Option<&str>,
    severity_filter: Option<ConstraintSeverity>,
) -> Result<(Vec<Constraint>, String), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 查询 constraints（带过滤）
    let constraints = list_constraints(&conn, scope_filter, severity_filter)?;
    Ok((constraints, schema_version))
}

/// 受控形态 render 视图的范围定位输入。
#[derive(Debug, Clone)]
pub struct ConstraintViewTarget {
    /// 受控 view 名（llm|human）。
    pub view: ConstraintView,
    /// 受控 scope 名（world|volume|chapter）。
    pub scope: ConstraintScope,
    /// `--chapter-id`（scope=chapter 必填；scope=volume 可替代 --volume-id 定位卷）。
    pub chapter_id: Option<String>,
    /// `--volume-id`（scope=volume 可选定位卷）。
    pub volume_id: Option<i64>,
}

/// 受控形态 render 视图的结果。
#[derive(Debug, Clone)]
pub struct ConstraintViewResult {
    pub view: ConstraintView,
    pub scope: ConstraintScope,
    /// scope=chapter 时的章节 ID。
    pub chapter_id: Option<String>,
    /// scope=volume 时的卷 ID。
    pub volume_id: Option<i64>,
    pub constraints: Vec<Constraint>,
    pub schema_version: String,
}

/// render_view 用例：按受控 view/scope 批量渲染约束注入视图（只读）。
///
/// 引用 contracts/cli/views-scopes.md §A 组 2/组 3 冻结枚举。只读：不写 SQLite、
/// 不改正文文件、不更新 updated_at、不推进状态机。
///
/// 范围解析（design D2），按 scope 字符串精确匹配（约定见 storage::
/// list_constraints_by_scopes 文档）：
/// - `world`：scope = "world"。
/// - `chapter`：必须有 `chapter_id`；查 chapter 所属 volume_id，叠加
///   "world" + "volume:<vid>" + "chapter:<chapter_id>" 三层。
/// - `volume`：必须有 `volume_id` 或 `chapter_id`（取所属卷）之一；
///   叠加 "world" + "volume:<vid>" 两层。
///
/// 前置条件：世界已初始化。
/// 失败场景：
/// - scope=chapter 缺 chapter_id：返回参数错误（退出码 2，命令层已校验，service 兜底）。
/// - scope=volume 缺 volume_id 与 chapter_id：返回参数错误（退出码 2）。
/// - chapter_id 不存在：返回 ChapterNotFound（退出码 6）。
/// - volume_id 不存在：返回 VolumeNotFound（退出码 6）。
pub fn render_view(target: &Path, t: ConstraintViewTarget) -> Result<ConstraintViewResult, CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 解析范围 → 一组精确 scope 字符串
    let (scopes, resolved_chapter_id, resolved_volume_id) = resolve_scopes(&conn, &t)?;

    // 3. 按精确 scope 集合查询约束
    let constraints = list_constraints_by_scopes(&conn, &scopes)?;

    Ok(ConstraintViewResult {
        view: t.view,
        scope: t.scope,
        chapter_id: resolved_chapter_id,
        volume_id: resolved_volume_id,
        constraints,
        schema_version,
    })
}

/// 根据受控 scope 与定位参数解析为一组精确 scope 字符串与定位信息。
///
/// 返回 (scope 字符串集合, chapter_id if scope=chapter, volume_id if scope=volume)。
fn resolve_scopes(
    conn: &rusqlite::Connection,
    t: &ConstraintViewTarget,
) -> Result<(Vec<String>, Option<String>, Option<i64>), CliError> {
    match t.scope {
        ConstraintScope::World => {
            // 全局不变式：仅 scope = "world"
            Ok((vec!["world".to_string()], None, None))
        }
        ConstraintScope::Chapter => {
            // 必须有 chapter_id（命令层已校验，service 兜底返回参数错误）
            let chapter_id = t.chapter_id.as_ref().ok_or_else(|| {
                CliError::usage("--scope chapter 必须配合 --chapter-id")
            })?;
            // 验证章节存在并取 volume_id
            let chapter_meta = get_chapter_metadata(conn, chapter_id)?.ok_or_else(|| {
                CliError::chapter_not_found(
                    format!("章节 '{}' 不存在。", chapter_id),
                    Some("使用 `novel chapter list` 查看所有章节。".to_string()),
                )
            })?;
            // 三层叠加：world + volume:<vid> + chapter:<cid>
            let mut scopes = vec!["world".to_string()];
            if let Some(vid) = chapter_meta.volume_id {
                scopes.push(format!("volume:{}", vid));
            }
            scopes.push(format!("chapter:{}", chapter_id));
            Ok((scopes, Some(chapter_id.clone()), chapter_meta.volume_id))
        }
        ConstraintScope::Volume => {
            // 必须有 volume_id 或 chapter_id 之一
            let volume_id = match (t.volume_id, t.chapter_id.as_ref()) {
                (Some(vid), _) => {
                    // 验证卷存在
                    verify_volume_exists(conn, vid)?;
                    Some(vid)
                }
                (None, Some(cid)) => {
                    // 取章节所属卷
                    let chapter_meta = get_chapter_metadata(conn, cid)?.ok_or_else(|| {
                        CliError::chapter_not_found(
                            format!("章节 '{}' 不存在。", cid),
                            Some("使用 `novel chapter list` 查看所有章节。".to_string()),
                        )
                    })?;
                    let vid = chapter_meta.volume_id.ok_or_else(|| {
                        CliError::usage(format!(
                            "章节 '{}' 未关联卷，无法用 --scope volume 定位。",
                            cid
                        ))
                    })?;
                    verify_volume_exists(conn, vid)?;
                    Some(vid)
                }
                (None, None) => {
                    return Err(CliError::usage(
                        "--scope volume 必须配合 --volume-id 或 --chapter-id",
                    ));
                }
            };
            let vid = volume_id.unwrap();
            // 两层叠加：world + volume:<vid>
            Ok((vec!["world".to_string(), format!("volume:{}", vid)], None, Some(vid)))
        }
    }
}

/// 验证卷存在，不存在返回 VolumeNotFound（退出码 6）。
fn verify_volume_exists(conn: &rusqlite::Connection, volume_id: i64) -> Result<(), CliError> {
    volume_repository::get_volume_by_id(conn, volume_id).map(|_| ()).map_err(|err| match err {
        CliError::NotFound { message, .. } => CliError::volume_not_found(message),
        other => other,
    })
}


/// batch_write_checks 用例：批量写入规则检查结果。
///
/// 前置条件：
/// - 世界已初始化
/// - 章节存在
/// - input JSON 文件存在且格式正确
/// - contentHash 与章节当前正文 hash 一致
///
/// 失败场景：
/// - 章节不存在：返回 ChapterNotFound 错误（退出码 6）
/// - contentHash 不匹配：返回 ContentHashMismatch 错误（退出码 5）
/// - JSON 格式错误：返回 InvalidInput 错误（退出码 2）
/// - 检查结果 severity 取值非法：返回 InvalidInput 错误（退出码 2）
pub fn batch_write_checks(
    target: &Path,
    chapter_id: &str,
    input_json_path: &Path,
) -> Result<(), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;

    // 2. 验证章节存在并读取当前 content_hash
    let chapter_meta = get_chapter_metadata(&conn, chapter_id)?
        .ok_or_else(|| CliError::chapter_not_found(
            format!("章节 '{}' 不存在，无法写入检查结果。", chapter_id),
            Some("使用 `novel chapter list` 查看所有章节。".to_string()),
        ))?;

    // 3. 读取并解析输入 JSON 文件
    let input_json = std::fs::read_to_string(input_json_path)
        .map_err(|e| CliError::invalid_input(
            format!("读取输入文件 {} 失败: {}", input_json_path.display(), e),
            Some("检查文件路径是否正确且文件可读。".to_string()),
        ))?;

    let input: CheckInput = serde_json::from_str(&input_json)
        .map_err(|e| CliError::invalid_input(
            format!("解析输入 JSON 失败: {}", e),
            Some("检查 JSON 格式是否正确，必须包含 contentHash 和 checks 字段。".to_string()),
        ))?;

    // 4. 验证 contentHash 一致性
    if input.content_hash != chapter_meta.content_hash {
        return Err(CliError::content_hash_mismatch(
            format!(
                "章节 '{}' 的 contentHash 不匹配：输入 JSON 中为 '{}'，当前章节为 '{}'。",
                chapter_id, input.content_hash, chapter_meta.content_hash
            ),
            Some("正文已被修改，请重新执行规则检查并生成新的检查结果。".to_string()),
        ));
    }

    // 4.5. 校验每个检查结果的 severity 取值（必须为 info/warning/hard）
    // 在入库前显式校验，使非法取值返回退出码 2（invalid_input）而非依赖
    // SQLite CHECK 兜底返回退出码 70（storage 错误）。
    for (idx, check) in input.checks.iter().enumerate() {
        if check.severity.parse::<ConstraintSeverity>().is_err() {
            return Err(CliError::invalid_input(
                format!(
                    "第 {} 个检查结果的 severity '{}' 非法：必须为 info、warning 或 hard。",
                    idx + 1,
                    check.severity
                ),
                Some("检查结果的 severity 字段必须为 info/warning/hard。".to_string()),
            ));
        }
    }

    // 5. 生成时间戳
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间失败: {}", e)))?
        .to_string();

    // 6. 批量插入检查结果
    rule_check_repository::insert_check_results(
        &conn,
        chapter_id,
        &chapter_meta.content_hash,
        &input.checks,
        &now,
    )?;

    Ok(())
}

/// list_checks 用例：查询章节的所有检查结果。
///
/// 前置条件：
/// - 世界已初始化
/// - 章节存在
///
/// 参数：
/// - target: 项目目录
/// - chapter_id: 章节 ID
/// - verify_hash: 是否强制验证所有检查结果的 content_hash 与当前章节 hash 一致
///
/// 返回：(检查结果数组, 当前章节 content_hash, schemaVersion)
///
/// 失败场景：
/// - 章节不存在：返回 ChapterNotFound 错误（退出码 6）
/// - verify_hash=true 且存在 stale 结果：返回 StaleRuleChecks 错误（退出码 5）
pub fn list_checks(
    target: &Path,
    chapter_id: &str,
    verify_hash: bool,
) -> Result<(Vec<RuleCheck>, String, String), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 验证章节存在并读取当前 content_hash
    let chapter_meta = get_chapter_metadata(&conn, chapter_id)?
        .ok_or_else(|| CliError::chapter_not_found(
            format!("章节 '{}' 不存在，无法查询检查结果。", chapter_id),
            Some("使用 `novel chapter list` 查看所有章节。".to_string()),
        ))?;

    // 3. 查询所有检查结果
    let checks = rule_check_repository::list_check_results(&conn, chapter_id)?;

    // 4. 如果 verify_hash=true，检查是否存在 stale 结果
    if verify_hash {
        let has_stale = checks.iter().any(|c| c.content_hash != chapter_meta.content_hash);
        if has_stale {
            return Err(CliError::stale_rule_checks(
                format!(
                    "章节 '{}' 的检查结果已过期（正文已修改）。",
                    chapter_id
                ),
                Some("重新执行 `novel constraint check` 写入新的检查结果。".to_string()),
            ));
        }
    }

    Ok((checks, chapter_meta.content_hash, schema_version))
}

/// 输入 JSON 结构：用于 batch_write_checks 用例。
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckInput {
    content_hash: String,
    checks: Vec<CheckResultInput>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn constraint_service_blocks_when_world_not_initialized() {
        let dir = tempdir().unwrap();

        let err = define_constraint(
            dir.path(),
            ConstraintType::Plot,
            "world",
            "时间线一致性",
            "所有章节时间线必须一致",
            ConstraintSeverity::Hard,
        )
        .unwrap_err();
        assert!(matches!(err, CliError::PreconditionBlocked { .. }));
        assert_eq!(i32::from(err.exit_code()), 3);
    }

    #[test]
    fn severity_from_str_rejects_invalid_value() {
        let err = "invalid".parse::<ConstraintSeverity>().unwrap_err();
        assert!(err.contains("Must be one of: info, warning, hard"));
    }
}
