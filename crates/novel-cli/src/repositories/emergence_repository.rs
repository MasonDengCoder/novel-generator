// emergence_repository 模块：封装涌现 CRUD 操作与状态转换。
//
// 职责（CLAUDE.md §6）：封装 SQL 查询，不做业务判断。

use crate::domain::{Decision, Emergence, EmergenceStatus};
use crate::errors::CliError;
use rusqlite::{Connection, params};

/// 创建涌现。
pub fn create(
    conn: &Connection,
    volume_id: i64,
    chapter_id: Option<&str>,
    title: &str,
    description: &str,
) -> Result<Emergence, CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 获取下一个序号
    let max_seq: Option<i64> = conn
        .query_row(
            "SELECT MAX(CAST(SUBSTR(id, LENGTH('em-' || ?1 || '-') + 1) AS INTEGER)) FROM emergences WHERE volume_id = ?1",
            params![volume_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询最大序号失败：{}", e)))?;

    let sequence_number = max_seq.unwrap_or(0) + 1;
    let id = format!("em-{}-{}", volume_id, sequence_number);

    conn.execute(
        "INSERT INTO emergences (id, volume_id, chapter_id, title, description, discovered_at, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8)",
        params![id, volume_id, chapter_id, title, description, timestamp, timestamp, timestamp],
    ).map_err(|e| CliError::storage(format!("创建涌现失败：{}", e)))?;

    get(conn, &id)
}

/// 根据 ID 获取涌现。
pub fn get(conn: &Connection, emergence_id: &str) -> Result<Emergence, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, volume_id, chapter_id, title, description, discovered_at, status, rating, rating_reason, rated_at, decision, decision_reason, decided_at, execution_target, execution_note, executed_at, archived_at, created_at, updated_at FROM emergences WHERE id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let emergence = stmt
        .query_row(params![emergence_id], |row| {
            let status_str: String = row.get(6)?;
            let status = EmergenceStatus::from_str(&status_str).unwrap_or(EmergenceStatus::Pending);

            let decision_str: Option<String> = row.get(10)?;
            let decision = decision_str.and_then(|s| Decision::from_str(&s));

            Ok(Emergence {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                chapter_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                discovered_at: row.get(5)?,
                status,
                rating: row.get(7)?,
                rating_reason: row.get(8)?,
                rated_at: row.get(9)?,
                decision,
                decision_reason: row.get(11)?,
                decided_at: row.get(12)?,
                execution_target: row.get(13)?,
                execution_note: row.get(14)?,
                executed_at: row.get(15)?,
                archived_at: row.get(16)?,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::emergence_not_found(format!("涌现 '{}' 不存在", emergence_id))
            }
            _ => CliError::storage(format!("查询涌现失败：{}", e)),
        })?;

    Ok(emergence)
}

/// 列出卷的所有涌现（按发现时间倒序）。
pub fn list(
    conn: &Connection,
    volume_id: i64,
    status_filter: Option<EmergenceStatus>,
) -> Result<Vec<Emergence>, CliError> {
    let (query, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(status) = status_filter {
        (
            "SELECT id, volume_id, chapter_id, title, description, discovered_at, status, rating, rating_reason, rated_at, decision, decision_reason, decided_at, execution_target, execution_note, executed_at, archived_at, created_at, updated_at FROM emergences WHERE volume_id = ?1 AND status = ?2 ORDER BY discovered_at DESC".to_string(),
            vec![Box::new(volume_id), Box::new(status.as_str().to_string())],
        )
    } else {
        (
            "SELECT id, volume_id, chapter_id, title, description, discovered_at, status, rating, rating_reason, rated_at, decision, decision_reason, decided_at, execution_target, execution_note, executed_at, archived_at, created_at, updated_at FROM emergences WHERE volume_id = ?1 ORDER BY discovered_at DESC".to_string(),
            vec![Box::new(volume_id)],
        )
    };

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let emergences = stmt
        .query_map(params_refs.as_slice(), |row| {
            let status_str: String = row.get(6)?;
            let status = EmergenceStatus::from_str(&status_str).unwrap_or(EmergenceStatus::Pending);

            let decision_str: Option<String> = row.get(10)?;
            let decision = decision_str.and_then(|s| Decision::from_str(&s));

            Ok(Emergence {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                chapter_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                discovered_at: row.get(5)?,
                status,
                rating: row.get(7)?,
                rating_reason: row.get(8)?,
                rated_at: row.get(9)?,
                decision,
                decision_reason: row.get(11)?,
                decided_at: row.get(12)?,
                execution_target: row.get(13)?,
                execution_note: row.get(14)?,
                executed_at: row.get(15)?,
                archived_at: row.get(16)?,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询涌现列表失败：{}", e)))?;

    let mut result = Vec::new();
    for emergence in emergences {
        result.push(emergence.map_err(|e| CliError::storage(format!("解析涌现失败：{}", e)))?);
    }

    Ok(result)
}

/// 评分涌现（pending → rated）。
pub fn rate(
    conn: &Connection,
    emergence_id: &str,
    rating: i64,
    reason: &str,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows_affected = conn
        .execute(
            "UPDATE emergences SET status = 'rated', rating = ?1, rating_reason = ?2, rated_at = ?3, updated_at = ?4 WHERE id = ?5",
            params![rating, reason, timestamp, timestamp, emergence_id],
        )
        .map_err(|e| CliError::storage(format!("评分涌现失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::emergence_not_found(format!(
            "涌现 '{}' 不存在",
            emergence_id
        )));
    }

    Ok(())
}

/// 决策涌现（rated → decided）。
pub fn decide(
    conn: &Connection,
    emergence_id: &str,
    decision: Decision,
    reason: &str,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows_affected = conn
        .execute(
            "UPDATE emergences SET status = 'decided', decision = ?1, decision_reason = ?2, decided_at = ?3, updated_at = ?4 WHERE id = ?5",
            params![decision.as_str(), reason, timestamp, timestamp, emergence_id],
        )
        .map_err(|e| CliError::storage(format!("决策涌现失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::emergence_not_found(format!(
            "涌现 '{}' 不存在",
            emergence_id
        )));
    }

    Ok(())
}

/// 执行涌现（decided → executed）。
pub fn execute(
    conn: &Connection,
    emergence_id: &str,
    target: &str,
    note: Option<&str>,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows_affected = conn
        .execute(
            "UPDATE emergences SET status = 'executed', execution_target = ?1, execution_note = ?2, executed_at = ?3, updated_at = ?4 WHERE id = ?5",
            params![target, note, timestamp, timestamp, emergence_id],
        )
        .map_err(|e| CliError::storage(format!("执行涌现失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::emergence_not_found(format!(
            "涌现 '{}' 不存在",
            emergence_id
        )));
    }

    Ok(())
}

/// 归档涌现（executed → archived）。
pub fn archive(conn: &Connection, emergence_id: &str) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows_affected = conn
        .execute(
            "UPDATE emergences SET status = 'archived', archived_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![timestamp, timestamp, emergence_id],
        )
        .map_err(|e| CliError::storage(format!("归档涌现失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::emergence_not_found(format!(
            "涌现 '{}' 不存在",
            emergence_id
        )));
    }

    Ok(())
}
