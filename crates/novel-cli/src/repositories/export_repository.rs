// export_repository 模块：导出所需的只读组合查询（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6）：封装导出所需的 SQL 组合查询，不做业务判断。
// 导出为只读操作：不修改 SQLite、不改正文、不更新 updated_at。

use crate::errors::CliError;
use crate::repositories::volume_repository::Volume;
use rusqlite::{Connection, params};
use serde::Serialize;

/// 导出章节体（含 contentHash，正文由 service 层读文件嵌入）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportChapterRow {
    pub id: String,
    pub title: String,
    pub content_path: String,
    pub content_hash: String,
    pub word_count: i64,
    pub status: String,
    pub volume_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// 卷下所有章节（按 created_at 升序）。
pub fn list_chapters_by_volume(conn: &Connection, volume_id: i64) -> Result<Vec<ExportChapterRow>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at \
             FROM chapters WHERE volume_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| CliError::storage(format!("准备查询章节失败：{}", e)))?;

    let rows = stmt
        .query_map(params![volume_id], |row| {
            Ok(ExportChapterRow {
                id: row.get(0)?,
                title: row.get(1)?,
                content_path: row.get(2)?,
                content_hash: row.get(3)?,
                word_count: row.get(4)?,
                status: row.get(5)?,
                volume_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询章节失败：{}", e)))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| CliError::storage(format!("解析章节数据失败：{}", e)))?);
    }
    Ok(result)
}

/// 所有卷（按 volume_order 升序），用于全量导出。
pub fn list_all_volumes(conn: &Connection) -> Result<Vec<Volume>, CliError> {
    crate::repositories::volume_repository::list_volumes(conn)
}

/// 所有章节（按 created_at 升序），用于全量 hash 校验。
pub fn list_all_chapters(conn: &Connection) -> Result<Vec<ExportChapterRow>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at \
             FROM chapters ORDER BY created_at ASC",
        )
        .map_err(|e| CliError::storage(format!("准备查询全部章节失败：{}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ExportChapterRow {
                id: row.get(0)?,
                title: row.get(1)?,
                content_path: row.get(2)?,
                content_hash: row.get(3)?,
                word_count: row.get(4)?,
                status: row.get(5)?,
                volume_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询全部章节失败：{}", e)))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| CliError::storage(format!("解析章节数据失败：{}", e)))?);
    }
    Ok(result)
}

/// 单章节行（按 ID）。
pub fn get_chapter_row(conn: &Connection, chapter_id: &str) -> Result<ExportChapterRow, CliError> {
    conn.query_row(
        "SELECT id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at \
         FROM chapters WHERE id = ?1",
        params![chapter_id],
        |row| {
            Ok(ExportChapterRow {
                id: row.get(0)?,
                title: row.get(1)?,
                content_path: row.get(2)?,
                content_hash: row.get(3)?,
                word_count: row.get(4)?,
                status: row.get(5)?,
                volume_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            CliError::chapter_not_found(format!("章节 '{}' 不存在", chapter_id), None)
        }
        _ => CliError::storage(format!("查询章节失败：{}", e)),
    })
}
