// mainplot_repository 模块：封装主线节点 CRUD 操作。
//
// 职责（CLAUDE.md §6）：封装 SQL 查询，不做业务判断。

use crate::domain::MainplotNode;
use crate::errors::CliError;
use rusqlite::{Connection, params};

/// 创建主线节点。
pub fn create_node(
    conn: &Connection,
    volume_id: i64,
    title: &str,
    description: Option<&str>,
    target_chapters: Option<&str>, // JSON 字符串
) -> Result<MainplotNode, CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 获取下一个序号
    let max_seq: Option<i64> = conn
        .query_row(
            "SELECT MAX(sequence_number) FROM mainplot_nodes WHERE volume_id = ?1",
            params![volume_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询最大序号失败：{}", e)))?;

    let sequence_number = max_seq.unwrap_or(0) + 1;
    // 与 chapter-001 对齐：序号 3 位零填充，保证排序稳定且格式一致。
    let id = format!("mp-{}-{:03}", volume_id, sequence_number);

    conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, description, target_chapters, achieved, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        params![id, volume_id, sequence_number, title, description, target_chapters, timestamp, timestamp],
    ).map_err(|e| CliError::storage(format!("创建主线节点失败：{}", e)))?;

    get_node(conn, &id)
}

/// 根据 ID 获取主线节点。
pub fn get_node(conn: &Connection, node_id: &str) -> Result<MainplotNode, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, volume_id, sequence_number, title, description, target_chapters, achieved, created_at, updated_at FROM mainplot_nodes WHERE id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let node = stmt
        .query_row(params![node_id], |row| {
            let target_chapters_json: Option<String> = row.get(5)?;
            let target_chapters = target_chapters_json.and_then(|json| {
                serde_json::from_str::<Vec<String>>(&json).ok()
            });

            Ok(MainplotNode {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                sequence_number: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                target_chapters,
                achieved: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::mainplot_node_not_found(format!("主线节点 '{}' 不存在", node_id))
            }
            _ => CliError::storage(format!("查询主线节点失败：{}", e)),
        })?;

    Ok(node)
}

/// 列出卷的所有主线节点（按序号排序）。
pub fn list_nodes(conn: &Connection, volume_id: i64) -> Result<Vec<MainplotNode>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, volume_id, sequence_number, title, description, target_chapters, achieved, created_at, updated_at FROM mainplot_nodes WHERE volume_id = ?1 ORDER BY sequence_number ASC")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let nodes = stmt
        .query_map(params![volume_id], |row| {
            let target_chapters_json: Option<String> = row.get(5)?;
            let target_chapters = target_chapters_json.and_then(|json| {
                serde_json::from_str::<Vec<String>>(&json).ok()
            });

            Ok(MainplotNode {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                sequence_number: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                target_chapters,
                achieved: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询主线节点列表失败：{}", e)))?;

    let mut result = Vec::new();
    for node in nodes {
        result.push(node.map_err(|e| CliError::storage(format!("解析主线节点失败：{}", e)))?);
    }

    Ok(result)
}

/// 更新主线节点的达成状态。
pub fn update_node_achieved(
    conn: &Connection,
    node_id: &str,
    achieved: i64,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows_affected = conn
        .execute(
            "UPDATE mainplot_nodes SET achieved = ?1, updated_at = ?2 WHERE id = ?3",
            params![achieved, timestamp, node_id],
        )
        .map_err(|e| CliError::storage(format!("更新主线节点失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::mainplot_node_not_found(format!(
            "主线节点 '{}' 不存在",
            node_id
        )));
    }

    Ok(())
}
