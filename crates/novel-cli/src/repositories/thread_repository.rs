// thread_repository 模块：封装线索与线索绑定 CRUD 操作。
//
// 职责（CLAUDE.md §6）：封装 SQL 查询，不做业务判断。

use crate::domain::{BindingType, Thread, ThreadBinding, ThreadStatus};
use crate::errors::CliError;
use rusqlite::{Connection, params};

/// 创建线索。
pub fn create_thread(
    conn: &Connection,
    volume_id: i64,
    name: &str,
    description: Option<&str>,
) -> Result<Thread, CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 获取下一个序号
    let max_seq: Option<i64> = conn
        .query_row(
            "SELECT MAX(CAST(SUBSTR(id, LENGTH('th-' || ?1 || '-') + 1) AS INTEGER)) FROM threads WHERE volume_id = ?1",
            params![volume_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询最大序号失败：{}", e)))?;

    let sequence_number = max_seq.unwrap_or(0) + 1;
    // 与 chapter-001 对齐：序号 3 位零填充，保证排序稳定且格式一致。
    let id = format!("th-{}-{:03}", volume_id, sequence_number);

    conn.execute(
        "INSERT INTO threads (id, volume_id, name, description, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6)",
        params![id, volume_id, name, description, timestamp, timestamp],
    ).map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            CliError::thread_name_conflict(format!("线索名称 '{}' 在卷 {} 中已存在", name, volume_id))
        } else {
            CliError::storage(format!("创建线索失败：{}", e))
        }
    })?;

    get_thread(conn, &id)
}

/// 根据 ID 获取线索。
pub fn get_thread(conn: &Connection, thread_id: &str) -> Result<Thread, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, volume_id, name, description, status, created_at, updated_at FROM threads WHERE id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let thread = stmt
        .query_row(params![thread_id], |row| {
            let status_str: String = row.get(4)?;
            let status = ThreadStatus::from_str(&status_str).unwrap_or(ThreadStatus::Active);

            Ok(Thread {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                status,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::thread_not_found(format!("线索 '{}' 不存在", thread_id))
            }
            _ => CliError::storage(format!("查询线索失败：{}", e)),
        })?;

    Ok(thread)
}

/// 列出卷的所有线索（按创建时间排序）。
pub fn list_threads(conn: &Connection, volume_id: i64) -> Result<Vec<Thread>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, volume_id, name, description, status, created_at, updated_at FROM threads WHERE volume_id = ?1 ORDER BY created_at ASC")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let threads = stmt
        .query_map(params![volume_id], |row| {
            let status_str: String = row.get(4)?;
            let status = ThreadStatus::from_str(&status_str).unwrap_or(ThreadStatus::Active);

            Ok(Thread {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                status,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询线索列表失败：{}", e)))?;

    let mut result = Vec::new();
    for thread in threads {
        result.push(thread.map_err(|e| CliError::storage(format!("解析线索失败：{}", e)))?);
    }

    Ok(result)
}

/// 更新线索状态。
pub fn update_status(
    conn: &Connection,
    thread_id: &str,
    status: ThreadStatus,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows_affected = conn
        .execute(
            "UPDATE threads SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), timestamp, thread_id],
        )
        .map_err(|e| CliError::storage(format!("更新线索状态失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::thread_not_found(format!(
            "线索 '{}' 不存在",
            thread_id
        )));
    }

    Ok(())
}

/// 绑定线索到章节。
pub fn bind_chapter(
    conn: &Connection,
    thread_id: &str,
    chapter_id: &str,
    binding_type: BindingType,
    note: Option<&str>,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    conn.execute(
        "INSERT INTO thread_bindings (thread_id, chapter_id, binding_type, note, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![thread_id, chapter_id, binding_type.as_str(), note, timestamp],
    ).map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") || e.to_string().contains("PRIMARY KEY constraint failed") {
            CliError::binding_already_exists(format!("线索 '{}' 与章节 '{}' 的绑定已存在", thread_id, chapter_id))
        } else if e.to_string().contains("FOREIGN KEY constraint failed") {
            CliError::not_found(format!("线索或章节不存在"), None)
        } else {
            CliError::storage(format!("绑定线索失败：{}", e))
        }
    })?;

    Ok(())
}

/// 查询章节的所有线索绑定。
pub fn query_by_chapter(
    conn: &Connection,
    chapter_id: &str,
) -> Result<Vec<ThreadBinding>, CliError> {
    let mut stmt = conn
        .prepare("SELECT thread_id, chapter_id, binding_type, note, created_at FROM thread_bindings WHERE chapter_id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let bindings = stmt
        .query_map(params![chapter_id], |row| {
            let binding_type_str: String = row.get(2)?;
            let binding_type = BindingType::from_str(&binding_type_str).unwrap_or(BindingType::Mention);

            Ok(ThreadBinding {
                thread_id: row.get(0)?,
                chapter_id: row.get(1)?,
                binding_type,
                note: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询线索绑定失败：{}", e)))?;

    let mut result = Vec::new();
    for binding in bindings {
        result.push(binding.map_err(|e| CliError::storage(format!("解析线索绑定失败：{}", e)))?);
    }

    Ok(result)
}

/// 获取线索名称（用于 query_by_chapter 的辅助查询）。
pub fn get_thread_name(conn: &Connection, thread_id: &str) -> Result<String, CliError> {
    let name: String = conn
        .query_row(
            "SELECT name FROM threads WHERE id = ?1",
            params![thread_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::thread_not_found(format!("线索 '{}' 不存在", thread_id))
            }
            _ => CliError::storage(format!("查询线索名称失败：{}", e)),
        })?;

    Ok(name)
}
