// verify_service 模块：完整性验证（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6、§10、design D2/D4/D5）：对世界状态做 hash 校验、引用完整性、状态一致性三层
// 只读验证，输出可机读分项报告，hard 违规非零退出（取最严重的一项为进程退出码）。
// 验证为只读操作：不执行任何 INSERT/UPDATE/DELETE，不更新 updated_at，不推进状态机。

use crate::errors::{CliError, ErrorCatalog, ErrorCode, ExitCode};
use crate::repositories::export_repository;
use crate::storage::connection;
use crate::validators::chapter_validator;
use rusqlite::Connection;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。IntegrityReport 的
// schema_version 字段 #[serde(skip)] 序列化时跳过，schemars 默认也 skip 该字段 schema
// 生成（该字段不进 JSON，不出现在生成 schema 是正确行为）。
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 验证报告（renderer 消费）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityReport {
    #[serde(skip)]
    pub schema_version: String,
    pub verified_at: String,
    pub hash_integrity: HashIntegrity,
    pub reference_integrity: ReferenceIntegrity,
    pub state_consistency: StateConsistency,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HashIntegrity {
    pub passed: bool,
    pub mismatches: Vec<HashMismatch>,
    pub missing_files: Vec<MissingFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HashMismatch {
    #[serde(rename = "chapterId")]
    pub chapter_id: String,
    #[serde(rename = "storedHash")]
    pub stored_hash: String,
    #[serde(rename = "actualHash")]
    pub actual_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MissingFile {
    #[serde(rename = "chapterId")]
    pub chapter_id: String,
    #[serde(rename = "expectedPath")]
    pub expected_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceIntegrity {
    pub passed: bool,
    pub dangling_references: Vec<DanglingReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DanglingReference {
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "entityId")]
    pub entity_id: String,
    #[serde(rename = "referencedEntityType")]
    pub referenced_entity_type: String,
    #[serde(rename = "referencedId")]
    pub referenced_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StateConsistency {
    pub passed: bool,
    pub inconsistencies: Vec<Inconsistency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Inconsistency {
    #[serde(rename = "type")]
    pub inconsistency_type: String,
    #[serde(rename = "chapterId", skip_serializing_if = "Option::is_none")]
    pub chapter_id: Option<String>,
    #[serde(rename = "volumeId", skip_serializing_if = "Option::is_none")]
    pub volume_id: Option<i64>,
    #[serde(rename = "boundHash", skip_serializing_if = "Option::is_none")]
    pub bound_hash: Option<String>,
    #[serde(rename = "currentHash", skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    #[serde(rename = "chapterStatus", skip_serializing_if = "Option::is_none")]
    pub chapter_status: Option<String>,
}

/// 验证退出码与错误码映射（取最严重的一项，design D2）。
///
/// 单一映射源：退出码与错误码的对应关系是 verify 独有契约（同一失败项的退出码与码串的组合
/// 不出现在其他命令），收敛到此函数，command 层只消费映射结果构造 `VerifyIntegrity` 错误。
/// 退出码优先级（保持现状，不改 SSOT 取值）：
/// - hash 失配 21；有 missing 文件 11（11 优先于 21，因文件缺失更基础）；
/// - 悬空引用 70（code `not_found`，退出码复用 `Storage` 类别）；
/// - 状态不一致 5（code `state_conflict`，退出码复用 `ExternalModification` 类别）；
/// - 全通过 0（成功路径，不走错误对象）。
///
/// 映射表（与 `error-codes.md` `verify integrity` 行一致）：
/// | 失败类别 | ExitCode | ErrorCode |
/// | hash 失配 | ContentModifiedExternally(21) | ContentModifiedExternally |
/// | 文件缺失 | FileNotFound(11) | FileNotFound |
/// | 状态不一致 | ExternalModification(5) | StateConflict |
/// | 悬空引用 | Storage(70) | NotFound |
/// | 全通过 | Success(0) | （成功路径不消费 code） |
pub fn verify_exit_mapping(report: &IntegrityReport) -> (ExitCode, ErrorCode) {
    if !report.hash_integrity.passed {
        // hash 失配 21；有 missing 文件 11（11 优先于 21，因文件缺失更基础）。
        if !report.hash_integrity.missing_files.is_empty() {
            return (ExitCode::FileNotFound, ErrorCode::FileNotFound);
        }
        return (ExitCode::ContentModifiedExternally, ErrorCode::ContentModifiedExternally);
    }
    if !report.reference_integrity.passed {
        return (ExitCode::Storage, ErrorCode::NotFound); // 悬空引用 70，码 `not_found`
    }
    if !report.state_consistency.passed {
        return (ExitCode::ExternalModification, ErrorCode::StateConflict); // 状态不一致 5，码 `state_conflict`
    }
    (ExitCode::Success, ErrorCode::Usage) // 全通过：code 不被消费
}

/// 验证退出码（i32，取最严重的一项）。
///
/// 保留既有 i32 接口以兼容调用方；内部委托 `verify_exit_mapping`（V1 per-entity 语义）。
pub fn verify_exit_code(report: &IntegrityReport) -> i32 {
    verify_exit_mapping(report).0.into()
}

/// 验证失败项的默认（V2）退出码——由错误类别派生，收敛 02 task 3.1（VerifyIntegrity 目录纯派生）。
///
/// V2 默认路径的退出码只表达自动化处置类别，不再按实体细分：hash 失配 / 文件缺失 / 状态不一致
/// 同属 `integrity` 类别 → `5`；悬空引用的 `not_found` 属 `not_found` 类别 → `6`。错误 ID 与
/// 类别的绑定仍由 `verify_exit_mapping` 决定（其返回的 `ErrorCode` 即目录 ID），退出码由该 ID
/// 经 `ErrorCatalog` 类别派生——保证「退出码是类别的纯函数」不变量（errors 层 `exit_code()` 仍唯一权威）。
///
/// 全通过时返回 `Success(0)`。未登记错误 ID（不应发生）回退该 ID 的 V1 per-entity 码。
pub fn verify_exit_code_v2(report: &IntegrityReport) -> i32 {
    let (v1_exit, code) = verify_exit_mapping(report);
    if v1_exit == ExitCode::Success {
        return ExitCode::Success.into();
    }
    ErrorCatalog::exit_code(code.as_str()).unwrap_or_else(|| i32::from(v1_exit))
}

/// 构造 verify hard 违规的 `CliError::VerifyIntegrity`（design D2/D4/D5）。
///
/// command 层据 `verify_exit_mapping` 的结果与完整报告 JSON 构造错误对象。`details` 须为
/// 与成功路径报告**同构**的 JSON（含 `schemaVersion`/`verifiedAt`/三分项，结构同
/// `verify-integrity.schema.json`，不重排/删除/重压缩）——故 command 层应复用 renderer 的
/// `render_verify_integrity_json` 产出（同成功路径结构），而非直接序列化 `IntegrityReport`
/// （其 `schema_version` 字段为 `#[serde(skip)]`，直接序列化会丢 `schemaVersion`）。
/// `message` 为人读概览，`suggestion` 为修复指引（具体文本由 command 层定，契约测试不固化）。
pub fn make_verify_integrity_error(
    exit_code: ExitCode,
    code: ErrorCode,
    details: serde_json::Value,
    message: impl Into<String>,
    suggestion: Option<String>,
) -> CliError {
    CliError::VerifyIntegrity {
        message: message.into(),
        suggestion,
        exit_code,
        code,
        details,
    }
}

/// 执行完整性验证，返回分项报告。
pub fn verify_integrity(target_dir: &Path) -> Result<IntegrityReport, CliError> {
    let conn = connection::open_existing_at(target_dir)?;
    let novel_dir = target_dir.join(".novel");
    let verified_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    let chapters = export_repository::list_all_chapters(&conn)?;

    // 1. hash 校验分项
    let mut mismatches = Vec::new();
    let mut missing_files = Vec::new();
    for row in &chapters {
        let content_path = novel_dir.join(&row.content_path);
        match std::fs::read_to_string(&content_path) {
            Ok(content) => {
                let actual_hash = chapter_validator::calculate_content_hash(&content);
                if actual_hash != row.content_hash {
                    mismatches.push(HashMismatch {
                        chapter_id: row.id.clone(),
                        stored_hash: row.content_hash.clone(),
                        actual_hash,
                    });
                }
            }
            Err(_) => {
                missing_files.push(MissingFile {
                    chapter_id: row.id.clone(),
                    expected_path: row.content_path.clone(),
                });
            }
        }
    }
    let hash_integrity = HashIntegrity {
        passed: mismatches.is_empty() && missing_files.is_empty(),
        mismatches,
        missing_files,
    };

    // 2. 引用完整性分项
    let dangling = check_reference_integrity(&conn)?;
    let reference_integrity = ReferenceIntegrity {
        passed: dangling.is_empty(),
        dangling_references: dangling,
    };

    // 3. 状态一致性分项
    let inconsistencies = check_state_consistency(&conn, &chapters)?;
    let state_consistency = StateConsistency {
        passed: inconsistencies.is_empty(),
        inconsistencies,
    };

    let schema_version = connection::read_schema_version(&conn).unwrap_or_default();

    Ok(IntegrityReport {
        schema_version,
        verified_at,
        hash_integrity,
        reference_integrity,
        state_consistency,
    })
}

/// 引用完整性：校验各实体外键引用有效。
fn check_reference_integrity(conn: &Connection) -> Result<Vec<DanglingReference>, CliError> {
    let mut dangling = Vec::new();

    // chapter → volume
    let rows: Vec<(String, Option<i64>)> = conn
        .prepare("SELECT id, volume_id FROM chapters")
        .map_err(|e| CliError::storage(format!("查询章节引用失败：{}", e)))?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CliError::storage(format!("查询章节引用失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析章节引用失败：{}", e)))?;
    for (chapter_id, volume_id) in rows {
        if let Some(vid) = volume_id {
            let exists: bool = conn
                .query_row("SELECT EXISTS(SELECT 1 FROM volumes WHERE id = ?1)", [vid], |row| row.get(0))
                .unwrap_or(false);
            if !exists {
                dangling.push(DanglingReference {
                    entity_type: "chapter".to_string(),
                    entity_id: chapter_id,
                    referenced_entity_type: "volume".to_string(),
                    referenced_id: vid.to_string(),
                });
            }
        }
    }

    // emergence → chapter / volume
    let em_rows: Vec<(String, Option<String>, i64)> = conn
        .prepare("SELECT id, chapter_id, volume_id FROM emergences")
        .map_err(|e| CliError::storage(format!("查询涌现引用失败：{}", e)))?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| CliError::storage(format!("查询涌现引用失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析涌现引用失败：{}", e)))?;
    for (em_id, chapter_id, volume_id) in em_rows {
        let vexists: bool = conn
            .query_row("SELECT EXISTS(SELECT 1 FROM volumes WHERE id = ?1)", [volume_id], |row| row.get(0))
            .unwrap_or(false);
        if !vexists {
            dangling.push(DanglingReference {
                entity_type: "emergence".to_string(),
                entity_id: em_id.clone(),
                referenced_entity_type: "volume".to_string(),
                referenced_id: volume_id.to_string(),
            });
        }
        if let Some(ch_id) = chapter_id {
            let cexists: bool = conn
                .query_row("SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)", [ch_id.as_str()], |row| row.get(0))
                .unwrap_or(false);
            if !cexists {
                dangling.push(DanglingReference {
                    entity_type: "emergence".to_string(),
                    entity_id: em_id.clone(),
                    referenced_entity_type: "chapter".to_string(),
                    referenced_id: ch_id.clone(),
                });
            }
        }
    }

    // thread_binding → chapter / thread
    let tb_rows: Vec<(String, String)> = conn
        .prepare("SELECT thread_id, chapter_id FROM thread_bindings")
        .map_err(|e| CliError::storage(format!("查询线索绑定引用失败：{}", e)))?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CliError::storage(format!("查询线索绑定引用失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析线索绑定引用失败：{}", e)))?;
    for (thread_id, chapter_id) in tb_rows {
        let texists: bool = conn
            .query_row("SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?1)", [thread_id.as_str()], |row| row.get(0))
            .unwrap_or(false);
        if !texists {
            dangling.push(DanglingReference {
                entity_type: "thread_binding".to_string(),
                entity_id: thread_id.clone(),
                referenced_entity_type: "thread".to_string(),
                referenced_id: String::new(),
            });
        }
        let cexists: bool = conn
            .query_row("SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)", [chapter_id.as_str()], |row| row.get(0))
            .unwrap_or(false);
        if !cexists {
            dangling.push(DanglingReference {
                entity_type: "thread_binding".to_string(),
                entity_id: thread_id.clone(),
                referenced_entity_type: "chapter".to_string(),
                referenced_id: chapter_id.clone(),
            });
        }
    }

    // verdict → chapter
    let v_rows: Vec<(i64, String)> = conn
        .prepare("SELECT id, chapter_id FROM verdicts")
        .map_err(|e| CliError::storage(format!("查询裁决引用失败：{}", e)))?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CliError::storage(format!("查询裁决引用失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析裁决引用失败：{}", e)))?;
    for (verdict_id, chapter_id) in v_rows {
        let exists: bool = conn
            .query_row("SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)", [chapter_id.as_str()], |row| row.get(0))
            .unwrap_or(false);
        if !exists {
            dangling.push(DanglingReference {
                entity_type: "verdict".to_string(),
                entity_id: verdict_id.to_string(),
                referenced_entity_type: "chapter".to_string(),
                referenced_id: chapter_id,
            });
        }
    }

    Ok(dangling)
}

/// 状态一致性：approved 章节裁决未 stale、archived 卷内章节均 approved、archived 涌现终态。
fn check_state_consistency(
    conn: &Connection,
    chapters: &[export_repository::ExportChapterRow],
) -> Result<Vec<Inconsistency>, CliError> {
    let mut inconsistencies = Vec::new();

    // approved 章节裁决未 stale（裁决 content_hash 与当前 chapters.content_hash 一致）。
    for row in chapters {
        if row.status == "approved" {
            let stale: Vec<(i64, String)> = conn
                .prepare("SELECT id, content_hash FROM verdicts WHERE chapter_id = ?1 AND content_hash != ?2")
                .map_err(|e| CliError::storage(format!("查询 stale 裁决失败：{}", e)))?
                .query_map([&row.id, &row.content_hash], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| CliError::storage(format!("查询 stale 裁决失败：{}", e)))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CliError::storage(format!("解析 stale 裁决失败：{}", e)))?;
            for (vid, bound_hash) in stale {
                inconsistencies.push(Inconsistency {
                    inconsistency_type: "stale_verdict".to_string(),
                    chapter_id: Some(row.id.clone()),
                    volume_id: row.volume_id,
                    bound_hash: Some(bound_hash),
                    current_hash: Some(row.content_hash.clone()),
                    chapter_status: None,
                });
                let _ = vid;
            }
        }
    }

    // archived 卷内章节均 approved。
    let archived_volumes: Vec<(i64,)> = conn
        .prepare("SELECT id FROM volumes WHERE status = 'archived'")
        .map_err(|e| CliError::storage(format!("查询归档卷失败：{}", e)))?
        .query_map([], |row| Ok((row.get(0)?,)))
        .map_err(|e| CliError::storage(format!("查询归档卷失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析归档卷失败：{}", e)))?;
    for (volume_id,) in archived_volumes {
        let non_approved: Vec<(String, String)> = conn
            .prepare("SELECT id, status FROM chapters WHERE volume_id = ?1 AND status != 'approved'")
            .map_err(|e| CliError::storage(format!("查询归档卷内非 approved 章节失败：{}", e)))?
            .query_map([volume_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| CliError::storage(format!("查询归档卷内非 approved 章节失败：{}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::storage(format!("解析归档卷内非 approved 章节失败：{}", e)))?;
        for (chapter_id, status) in non_approved {
            inconsistencies.push(Inconsistency {
                inconsistency_type: "archived_volume_has_non_approved_chapter".to_string(),
                chapter_id: Some(chapter_id),
                volume_id: Some(volume_id),
                bound_hash: None,
                current_hash: None,
                chapter_status: Some(status),
            });
        }
    }

    Ok(inconsistencies)
}
