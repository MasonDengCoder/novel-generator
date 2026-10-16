// thread_validator 模块：线索校验（名称唯一性、绑定类型合法性）。

use crate::errors::CliError;
use rusqlite::Connection;

/// 校验绑定类型：必须为 mention、develop 或 resolve。
pub fn validate_binding_type(binding_type: &str) -> Result<(), CliError> {
    match binding_type {
        "mention" | "develop" | "resolve" => Ok(()),
        _ => Err(CliError::usage(format!(
            "绑定类型无效：{}，必须为 mention、develop 或 resolve",
            binding_type
        ))),
    }
}

/// 校验线索名称唯一性：在指定卷内名称不能重复。
pub fn validate_thread_name_uniqueness(
    conn: &Connection,
    volume_id: i64,
    name: &str,
) -> Result<(), CliError> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM threads WHERE volume_id = ?1 AND name = ?2)",
            rusqlite::params![volume_id, name],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查线索名称唯一性失败：{}", e)))?;

    if exists {
        return Err(CliError::thread_name_conflict(format!(
            "线索名称在卷内已存在：{}",
            name
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // 创建必要的表
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                volume_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                status TEXT DEFAULT 'active'
            );

            INSERT INTO threads (id, volume_id, name) VALUES ('th-1-001', 1, 'revenge-arc');
            INSERT INTO threads (id, volume_id, name) VALUES ('th-1-002', 1, 'mentor-bond');
            INSERT INTO threads (id, volume_id, name) VALUES ('th-2-001', 2, 'revenge-arc');
            "#,
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_validate_binding_type_valid() {
        assert!(validate_binding_type("mention").is_ok());
        assert!(validate_binding_type("develop").is_ok());
        assert!(validate_binding_type("resolve").is_ok());
    }

    #[test]
    fn test_validate_binding_type_invalid() {
        assert!(validate_binding_type("invalid").is_err());
        assert!(validate_binding_type("MENTION").is_err());
        assert!(validate_binding_type("").is_err());
    }

    #[test]
    fn test_validate_thread_name_uniqueness_ok() {
        let conn = setup_test_db();
        // 卷 1 中不存在的名称
        assert!(validate_thread_name_uniqueness(&conn, 1, "new-thread").is_ok());
        // 卷 2 中可以使用卷 1 已有的名称（不同卷）
        assert!(validate_thread_name_uniqueness(&conn, 2, "mentor-bond").is_ok());
    }

    #[test]
    fn test_validate_thread_name_uniqueness_conflict() {
        let conn = setup_test_db();
        // 卷 1 中已存在 "revenge-arc"
        let result = validate_thread_name_uniqueness(&conn, 1, "revenge-arc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("revenge-arc"));
    }

    #[test]
    fn test_validate_thread_name_uniqueness_different_volume() {
        let conn = setup_test_db();
        // 卷 2 中已存在 "revenge-arc"，但卷 3 中可以使用
        assert!(validate_thread_name_uniqueness(&conn, 3, "revenge-arc").is_ok());
    }
}
