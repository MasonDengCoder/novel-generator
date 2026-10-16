// volume_validator 模块：卷级参数验证。

use crate::errors::CliError;
use rusqlite::Connection;

/// 检查卷名是否唯一。
pub fn validate_unique_name(conn: &Connection, name: &str) -> Result<(), CliError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM volumes WHERE name = ?1", [name], |row| {
            row.get(0)
        })
        .map_err(|e| CliError::storage(format!("查询卷名失败：{}", e)))?;

    if count > 0 {
        return Err(CliError::resource_conflict(format!("卷名 '{}' 已存在", name)));
    }

    Ok(())
}

/// 检查卷序号范围（>= 1）。
pub fn validate_volume_order(order: i64) -> Result<(), CliError> {
    if order < 1 {
        return Err(CliError::usage(format!("卷序号必须 >= 1，当前值：{}", order)));
    }
    Ok(())
}

/// 检查主线节点 ID 格式（非空、无非法字符）。
pub fn validate_mainplot_node_id(id: &str) -> Result<(), CliError> {
    if id.is_empty() {
        return Err(CliError::usage("主线节点 ID 不能为空".to_string()));
    }

    // 简单验证：只允许字母、数字、连字符、下划线
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(CliError::usage(format!(
            "主线节点 ID 包含非法字符：'{}'（仅允许字母、数字、连字符、下划线）",
            id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;
    use tempfile::tempdir;

    fn setup() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn, "2026-06-30T00:00:00Z").unwrap();
        (conn, dir)
    }

    #[test]
    fn test_validate_volume_order_valid() {
        assert!(validate_volume_order(1).is_ok());
        assert!(validate_volume_order(100).is_ok());
    }

    #[test]
    fn test_validate_volume_order_invalid() {
        assert!(validate_volume_order(0).is_err());
        assert!(validate_volume_order(-1).is_err());
    }

    #[test]
    fn test_validate_mainplot_node_id_valid() {
        assert!(validate_mainplot_node_id("node-001").is_ok());
        assert!(validate_mainplot_node_id("main_plot_A").is_ok());
        assert!(validate_mainplot_node_id("NODE123").is_ok());
    }

    #[test]
    fn test_validate_mainplot_node_id_invalid() {
        assert!(validate_mainplot_node_id("").is_err());
        assert!(validate_mainplot_node_id("node@001").is_err());
        assert!(validate_mainplot_node_id("node 001").is_err());
    }

    #[test]
    fn test_validate_unique_name() {
        let (conn, _dir) = setup();

        // 首次使用卷名应通过
        assert!(validate_unique_name(&conn, "第一卷").is_ok());

        // 创建卷后，重复卷名应失败
        conn.execute(
            "INSERT INTO volumes (name, volume_order, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            ["第一卷", "1", "active", "2026-06-30T00:00:00Z", "2026-06-30T00:00:00Z"],
        ).unwrap();

        assert!(validate_unique_name(&conn, "第一卷").is_err());
    }
}

