// export_service 模块：导出世界状态快照（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6、design D4）：组合卷与下属实体为只读 JSON 快照，正文以字段嵌入。
// 导出为只读操作：不修改 SQLite、不改正文、不更新 updated_at，不推进状态机。
// hash 失配：export chapter 返回 21（content_modified_externally）；export full 列入 warnings（退出码 0）。

use crate::errors::CliError;
use crate::repositories::{emergence_repository, export_repository, mainplot_repository, rule_check_repository, thread_repository, volume_repository};
use crate::storage::connection;
use crate::validators::chapter_validator;
use rusqlite::Connection;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 导出章节体（JSON 输出，正文以字段嵌入）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportChapterBody {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(rename = "contentHash")]
    pub content_hash: String,
    #[serde(rename = "wordCount")]
    pub word_count: i64,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// 读取正文文件并校验 hash（导出章节用，hash 失配返回 21）。
fn read_and_verify_content(novel_dir: &Path, row: &export_repository::ExportChapterRow) -> Result<(String, String), CliError> {
    let content_path = novel_dir.join(&row.content_path);
    let content = std::fs::read_to_string(&content_path).map_err(|e| {
        CliError::file_not_found(format!("正文文件不存在：{} - {}", content_path.display(), e))
    })?;
    let actual_hash = chapter_validator::calculate_content_hash(&content);
    if actual_hash != row.content_hash {
        return Err(CliError::content_modified_externally(format!(
            "章节 {} 的正文已被外部修改，hash 失配。",
            row.id
        )));
    }
    Ok((content, actual_hash))
}

/// 导出单个卷（含下属实体，正文嵌入）。
/// 返回 `(schema_version, VolumeExportData)`：schemaVersion 来自 `PRAGMA user_version`（design D2：renderer 不再接收 `&Connection`）。
pub fn export_volume(target_dir: &Path, volume_id: i64) -> Result<(String, VolumeExportData), CliError> {
    let conn = connection::open_existing_at(target_dir)?;
    let schema_version = connection::read_schema_version(&conn)?;
    let volume = volume_repository::get_volume_by_id(&conn, volume_id)?;
    let novel_dir = target_dir.join(".novel");

    let chapter_rows = export_repository::list_chapters_by_volume(&conn, volume_id)?;
    let mut chapters = Vec::new();
    let mut warnings = Vec::new();
    if volume.status != "archived" {
        warnings.push("卷未归档，导出为非冻结快照".to_string());
    }
    for row in &chapter_rows {
        // 导出卷：hash 失配列入 warnings 而非阻断（与 export chapter 不同）。
        match read_and_verify_content(&novel_dir, row) {
            Ok((content, hash)) => chapters.push(ExportChapterBody {
                id: row.id.clone(),
                title: row.title.clone(),
                content,
                content_hash: hash,
                word_count: row.word_count,
                status: row.status.clone(),
                created_at: row.created_at.clone(),
                updated_at: row.updated_at.clone(),
            }),
            Err(e) => {
                // hash 失配或文件缺失：列入 warning，正文置空。
                warnings.push(format!("章节 {} hash 校验失败：{}", row.id, e.message()));
                chapters.push(ExportChapterBody {
                    id: row.id.clone(),
                    title: row.title.clone(),
                    content: String::new(),
                    content_hash: row.content_hash.clone(),
                    word_count: row.word_count,
                    status: row.status.clone(),
                    created_at: row.created_at.clone(),
                    updated_at: row.updated_at.clone(),
                });
            }
        }
    }

    let emergences = emergence_repository::list(&conn, volume_id, None)?;
    let mainplot_nodes = mainplot_repository::list_nodes(&conn, volume_id)?;
    let threads = thread_repository::list_threads(&conn, volume_id)?;
    let goals = volume_repository::get_goals_by_volume(&conn, volume_id)?;

    // verdicts 与 ruleChecks：按章节遍历收集。
    let mut verdicts = Vec::new();
    let mut rule_checks = Vec::new();
    for row in &chapter_rows {
        // verdicts：查询该章节所有 mainplot-landing 裁决（通过 list_check_results 模式不可用，遍历 verdicts 表）。
        let chapter_verdicts = list_verdicts_for_chapter(&conn, &row.id)?;
        verdicts.extend(chapter_verdicts);
        let checks = rule_check_repository::list_check_results(&conn, &row.id)?;
        rule_checks.extend(checks);
    }

    Ok((schema_version, VolumeExportData {
        volume,
        chapters,
        emergences,
        mainplot_nodes,
        threads,
        goals,
        verdicts,
        rule_checks,
        warnings,
    }))
}

/// 导出单个章节（含裁决与规则检查结果，hash 失配返回 21）。
/// 返回 `(schema_version, ChapterExportData)`（design D2）。
pub fn export_chapter(target_dir: &Path, chapter_id: &str) -> Result<(String, ChapterExportData), CliError> {
    let conn = connection::open_existing_at(target_dir)?;
    let schema_version = connection::read_schema_version(&conn)?;
    let row = export_repository::get_chapter_row(&conn, chapter_id)?;
    let novel_dir = target_dir.join(".novel");
    let (content, content_hash) = read_and_verify_content(&novel_dir, &row)?;

    let verdicts = list_verdicts_for_chapter(&conn, chapter_id)?;
    let rule_checks = rule_check_repository::list_check_results(&conn, chapter_id)?;

    Ok((schema_version, ChapterExportData {
        chapter: ExportChapterBody {
            id: row.id,
            title: row.title,
            content,
            content_hash,
            word_count: row.word_count,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        },
        verdicts,
        rule_checks,
    }))
}

/// 全量导出（遍历所有卷及其下属实体，逐实体校验 hash，失配列入 warnings）。
/// 返回 `(schema_version, FullExportData)`（design D2）。
pub fn export_full(target_dir: &Path) -> Result<(String, FullExportData), CliError> {
    let conn = connection::open_existing_at(target_dir)?;
    let schema_version = connection::read_schema_version(&conn)?;
    let novel_dir = target_dir.join(".novel");
    let volumes = export_repository::list_all_volumes(&conn)?;

    let mut volume_exports = Vec::new();
    let mut warnings = Vec::new();

    for volume in &volumes {
        let chapter_rows = export_repository::list_chapters_by_volume(&conn, volume.id)?;
        let mut chapters = Vec::new();
        for row in &chapter_rows {
            match read_and_verify_content(&novel_dir, row) {
                Ok((content, hash)) => chapters.push(ExportChapterBody {
                    id: row.id.clone(),
                    title: row.title.clone(),
                    content,
                    content_hash: hash,
                    word_count: row.word_count,
                    status: row.status.clone(),
                    created_at: row.created_at.clone(),
                    updated_at: row.updated_at.clone(),
                }),
                Err(e) => {
                    // hash 失配：列入 warnings（含 storedHash/actualHash）。
                    if let crate::errors::CliError::ContentModifiedExternally { .. } = e {
                        warnings.push(HashWarning {
                            chapter_id: row.id.clone(),
                            stored_hash: row.content_hash.clone(),
                            actual_hash: String::new(),
                        });
                    }
                    chapters.push(ExportChapterBody {
                        id: row.id.clone(),
                        title: row.title.clone(),
                        content: String::new(),
                        content_hash: row.content_hash.clone(),
                        word_count: row.word_count,
                        status: row.status.clone(),
                        created_at: row.created_at.clone(),
                        updated_at: row.updated_at.clone(),
                    });
                }
            }
        }
        volume_exports.push(VolumeExportSummary {
            volume: volume.clone(),
            chapters,
        });
    }

    Ok((schema_version, FullExportData {
        volumes: volume_exports,
        warnings,
    }))
}

/// 列出章节的所有裁决（用于导出）。
fn list_verdicts_for_chapter(conn: &Connection, chapter_id: &str) -> Result<Vec<crate::repositories::verdict_repository::Verdict>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, chapter_id, verdict_type, node_id, achieved, content_hash, created_at FROM verdicts WHERE chapter_id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询裁决失败：{}", e)))?;
    let rows = stmt
        .query_map([chapter_id], |row| {
            Ok(crate::repositories::verdict_repository::Verdict {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                verdict_type: row.get(2)?,
                node_id: row.get(3)?,
                achieved: row.get::<_, i32>(4)? != 0,
                content_hash: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询裁决失败：{}", e)))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| CliError::storage(format!("解析裁决失败：{}", e)))?);
    }
    Ok(result)
}

/// 卷导出体（renderer 消费）。
pub struct VolumeExportData {
    pub volume: crate::repositories::volume_repository::Volume,
    pub chapters: Vec<ExportChapterBody>,
    pub emergences: Vec<crate::domain::Emergence>,
    pub mainplot_nodes: Vec<crate::domain::MainplotNode>,
    pub threads: Vec<crate::domain::Thread>,
    pub goals: Vec<crate::repositories::volume_repository::VolumeGoal>,
    pub verdicts: Vec<crate::repositories::verdict_repository::Verdict>,
    pub rule_checks: Vec<crate::repositories::rule_check_repository::RuleCheck>,
    pub warnings: Vec<String>,
}

/// 章节导出体（renderer 消费）。
pub struct ChapterExportData {
    pub chapter: ExportChapterBody,
    pub verdicts: Vec<crate::repositories::verdict_repository::Verdict>,
    pub rule_checks: Vec<crate::repositories::rule_check_repository::RuleCheck>,
}

/// 全量导出体（renderer 消费）。
pub struct FullExportData {
    pub volumes: Vec<VolumeExportSummary>,
    pub warnings: Vec<HashWarning>,
}

/// 全量导出中的卷摘要（volume + chapters，其他实体省略以控制体积）。
pub struct VolumeExportSummary {
    pub volume: crate::repositories::volume_repository::Volume,
    pub chapters: Vec<ExportChapterBody>,
}

/// hash 失配警告（全量导出）。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HashWarning {
    #[serde(rename = "chapterId")]
    pub chapter_id: String,
    #[serde(rename = "storedHash")]
    pub stored_hash: String,
    #[serde(rename = "actualHash")]
    pub actual_hash: String,
}
