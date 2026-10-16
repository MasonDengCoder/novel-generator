// storage/settings.rs
// s01-project-bootstrap：settings 表的数据访问层。
//
// 职责：封装 settings 表的 SQL 读写，支持 UPSERT、查询、列表。
// 不做业务判断（如世界未初始化检查），业务规则由 service 层负责。

use rusqlite::{params, Connection};

use crate::domain::Setting;
use crate::errors::CliError;

/// 插入或更新 setting（UPSERT 语义）。
pub fn insert_or_update_setting(
    conn: &Connection,
    name: &str,
    value: &str,
    created_at: &str,
) -> Result<(), CliError> {
    conn.execute(
        "INSERT INTO settings (name, value, created_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(name) DO UPDATE SET value = excluded.value",
        params![name, value, created_at],
    )
    .map_err(|e| CliError::storage(format!("写入 setting 失败: {}", e)))?;
    Ok(())
}

/// 根据 name 查询 setting。不存在则返回 None。
pub fn get_setting_by_name(conn: &Connection, name: &str) -> Result<Option<Setting>, CliError> {
    let result = conn.query_row(
        "SELECT name, value, created_at FROM settings WHERE name = ?1",
        params![name],
        |row| {
            Ok(Setting::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        },
    );

    match result {
        Ok(setting) => Ok(Some(setting)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(CliError::storage(format!("查询 setting 失败: {}", e))),
    }
}

/// 列出所有 settings，按 name 排序。
pub fn list_all_settings(conn: &Connection) -> Result<Vec<Setting>, CliError> {
    let mut stmt = conn
        .prepare("SELECT name, value, created_at FROM settings ORDER BY name")
        .map_err(|e| CliError::storage(format!("准备 settings 查询失败: {}", e)))?;

    let settings = stmt
        .query_map([], |row| {
            Ok(Setting::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })
        .map_err(|e| CliError::storage(format!("查询 settings 失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("读取 settings 行失败: {}", e)))?;

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
        conn
    }

    #[test]
    fn upsert_updates_existing_setting() {
        let conn = setup_conn();

        insert_or_update_setting(&conn, "world-style", "奇幻史诗", "2026-06-29T00:00:00Z").unwrap();
        insert_or_update_setting(&conn, "world-style", "赛博朋克", "2026-06-29T01:00:00Z").unwrap();

        let setting = get_setting_by_name(&conn, "world-style").unwrap().unwrap();
        assert_eq!(setting.value, "赛博朋克");
        assert_eq!(setting.created_at, "2026-06-29T00:00:00Z");
        assert_eq!(list_all_settings(&conn).unwrap().len(), 1);
    }

    #[test]
    fn get_missing_setting_returns_none() {
        let conn = setup_conn();

        let setting = get_setting_by_name(&conn, "missing-key").unwrap();
        assert!(setting.is_none());
    }

    #[test]
    fn list_empty_settings_returns_empty() {
        let conn = setup_conn();

        let settings = list_all_settings(&conn).unwrap();
        assert!(settings.is_empty());
    }
}
