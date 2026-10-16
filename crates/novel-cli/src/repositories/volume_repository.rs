// volume_repository 模块：封装卷级 CRUD 操作。
//
// 职责（CLAUDE.md §6）：封装 SQL 查询，不做业务判断。
// 调用 storage 层的 transaction 函数。

use crate::errors::CliError;
use rusqlite::{Connection, params};

/// 卷完整数据（元数据）。
#[derive(Debug, Clone)]
pub struct Volume {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub volume_order: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub sealed_at: Option<String>,
    pub archived_at: Option<String>,
}

/// 卷目标数据。
#[derive(Debug, Clone)]
pub struct VolumeGoal {
    pub id: i64,
    pub volume_id: i64,
    pub mainplot_node_id: String,
    pub description: Option<String>,
    pub is_mandatory: bool,
    pub created_at: String,
}

/// 创建新卷（插入 SQLite 记录）。
pub fn create_volume(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    volume_order: i64,
) -> Result<i64, CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    conn.execute(
        "INSERT INTO volumes (name, description, volume_order, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![name, description, volume_order, "active", timestamp, timestamp],
    ).map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            CliError::resource_conflict(format!("卷名 '{}' 已存在", name))
        } else {
            CliError::storage(format!("创建卷失败：{}", e))
        }
    })?;

    Ok(conn.last_insert_rowid())
}

/// 根据 ID 获取卷信息。
pub fn get_volume_by_id(conn: &Connection, volume_id: i64) -> Result<Volume, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, volume_order, status, created_at, updated_at, sealed_at, archived_at FROM volumes WHERE id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let volume = stmt
        .query_row(params![volume_id], |row| {
            Ok(Volume {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                volume_order: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                sealed_at: row.get(7)?,
                archived_at: row.get(8)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::not_found(format!("卷 ID {} 不存在", volume_id), None)
            }
            _ => CliError::storage(format!("查询卷失败：{}", e)),
        })?;

    Ok(volume)
}

/// 列举所有卷（按 volume_order 排序）。
pub fn list_volumes(conn: &Connection) -> Result<Vec<Volume>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, name, description, volume_order, status, created_at, updated_at, sealed_at, archived_at FROM volumes ORDER BY volume_order ASC")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let volumes = stmt
        .query_map([], |row| {
            Ok(Volume {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                volume_order: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                sealed_at: row.get(7)?,
                archived_at: row.get(8)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询卷列表失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析卷列表失败：{}", e)))?;

    Ok(volumes)
}

/// 更新卷状态。
pub fn update_volume_status(
    conn: &Connection,
    volume_id: i64,
    status: &str,
) -> Result<(), CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let rows = conn
        .execute(
            "UPDATE volumes SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, timestamp, volume_id],
        )
        .map_err(|e| CliError::storage(format!("更新卷状态失败：{}", e)))?;

    if rows == 0 {
        return Err(CliError::not_found(format!("卷 ID {} 不存在", volume_id), None));
    }

    Ok(())
}

/// 设置卷的封卷时间戳。
pub fn set_volume_sealed_at(
    conn: &Connection,
    volume_id: i64,
    timestamp: &str,
) -> Result<(), CliError> {
    let rows = conn
        .execute(
            "UPDATE volumes SET sealed_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![timestamp, volume_id],
        )
        .map_err(|e| CliError::storage(format!("设置封卷时间失败：{}", e)))?;

    if rows == 0 {
        return Err(CliError::not_found(format!("卷 ID {} 不存在", volume_id), None));
    }

    Ok(())
}

/// 设置卷的归档时间戳。
pub fn set_volume_archived_at(
    conn: &Connection,
    volume_id: i64,
    timestamp: &str,
) -> Result<(), CliError> {
    let rows = conn
        .execute(
            "UPDATE volumes SET archived_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![timestamp, volume_id],
        )
        .map_err(|e| CliError::storage(format!("设置归档时间失败：{}", e)))?;

    if rows == 0 {
        return Err(CliError::not_found(format!("卷 ID {} 不存在", volume_id), None));
    }

    Ok(())
}

/// 添加卷目标。
pub fn add_goal(
    conn: &Connection,
    volume_id: i64,
    mainplot_node_id: &str,
    description: Option<&str>,
    is_mandatory: bool,
) -> Result<i64, CliError> {
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    conn.execute(
        "INSERT INTO volume_goals (volume_id, mainplot_node_id, description, is_mandatory, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![volume_id, mainplot_node_id, description, is_mandatory, timestamp],
    ).map_err(|e| CliError::storage(format!("添加卷目标失败：{}", e)))?;

    Ok(conn.last_insert_rowid())
}

/// 获取卷的所有目标。
pub fn get_goals_by_volume(conn: &Connection, volume_id: i64) -> Result<Vec<VolumeGoal>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, volume_id, mainplot_node_id, description, is_mandatory, created_at FROM volume_goals WHERE volume_id = ?1 ORDER BY id ASC")
        .map_err(|e| CliError::storage(format!("准备查询失败：{}", e)))?;

    let goals = stmt
        .query_map(params![volume_id], |row| {
            Ok(VolumeGoal {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                mainplot_node_id: row.get(2)?,
                description: row.get(3)?,
                is_mandatory: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询卷目标失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析卷目标失败：{}", e)))?;

    Ok(goals)
}

/// 删除卷目标。
pub fn remove_goal(conn: &Connection, goal_id: i64) -> Result<(), CliError> {
    let rows = conn
        .execute("DELETE FROM volume_goals WHERE id = ?1", params![goal_id])
        .map_err(|e| CliError::storage(format!("删除卷目标失败：{}", e)))?;

    if rows == 0 {
        return Err(CliError::not_found(format!("目标 ID {} 不存在", goal_id), None));
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
    fn test_create_volume() {
        let (conn, _dir) = setup();

        let volume_id = create_volume(&conn, "第一卷", Some("测试卷"), 1).unwrap();
        assert_eq!(volume_id, 1);

        let volume = get_volume_by_id(&conn, volume_id).unwrap();
        assert_eq!(volume.name, "第一卷");
        assert_eq!(volume.description, Some("测试卷".to_string()));
        assert_eq!(volume.volume_order, 1);
        assert_eq!(volume.status, "active");
    }

    #[test]
    fn test_create_volume_duplicate_name() {
        let (conn, _dir) = setup();

        create_volume(&conn, "第一卷", None, 1).unwrap();
        let result = create_volume(&conn, "第一卷", None, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_volumes_sorted() {
        let (conn, _dir) = setup();

        create_volume(&conn, "第三卷", None, 3).unwrap();
        create_volume(&conn, "第一卷", None, 1).unwrap();
        create_volume(&conn, "第二卷", None, 2).unwrap();

        let volumes = list_volumes(&conn).unwrap();
        assert_eq!(volumes.len(), 3);
        assert_eq!(volumes[0].name, "第一卷");
        assert_eq!(volumes[1].name, "第二卷");
        assert_eq!(volumes[2].name, "第三卷");
    }

    #[test]
    fn test_update_volume_status() {
        let (conn, _dir) = setup();

        let volume_id = create_volume(&conn, "第一卷", None, 1).unwrap();
        update_volume_status(&conn, volume_id, "sealing").unwrap();

        let volume = get_volume_by_id(&conn, volume_id).unwrap();
        assert_eq!(volume.status, "sealing");
    }

    #[test]
    fn test_add_and_get_goals() {
        let (conn, _dir) = setup();

        let volume_id = create_volume(&conn, "第一卷", None, 1).unwrap();
        let goal_id = add_goal(&conn, volume_id, "node-001", Some("完成主线A"), true).unwrap();
        assert_eq!(goal_id, 1);

        let goals = get_goals_by_volume(&conn, volume_id).unwrap();
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].mainplot_node_id, "node-001");
        assert_eq!(goals[0].is_mandatory, true);
    }

    #[test]
    fn test_remove_goal() {
        let (conn, _dir) = setup();

        let volume_id = create_volume(&conn, "第一卷", None, 1).unwrap();
        let goal_id = add_goal(&conn, volume_id, "node-001", None, true).unwrap();

        remove_goal(&conn, goal_id).unwrap();

        let goals = get_goals_by_volume(&conn, volume_id).unwrap();
        assert_eq!(goals.len(), 0);
    }
}

