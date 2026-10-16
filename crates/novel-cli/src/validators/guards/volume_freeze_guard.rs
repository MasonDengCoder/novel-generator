// volume_freeze_guard 模块：归档卷跨实体冻结门禁（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6、§10、design D3）：卷状态为 archived 时，拒绝修改其下属实体
// （章节正文、裁决、涌现、thread binding、卷目标等）。冻结判定属业务规则，集中在 service
// 层共享 guard，各写 service 调用同一 guard，避免散落 repository/storage 层。
//
// 返回退出码 3 + `volume_already_archived`（hard 阻断，不静默降级）。

use crate::errors::CliError;
use rusqlite::Connection;

/// 检查卷是否处于冻结态（archived）。冻结时返回 `volume_already_archived` 错误。
///
/// 用于卷级写操作（如 volume goal set/remove、卷属性修改）。
pub fn ensure_volume_not_archived(conn: &Connection, volume_id: i64) -> Result<(), CliError> {
    let status: String = conn
        .query_row(
            "SELECT status FROM volumes WHERE id = ?1",
            [volume_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::volume_not_found(format!("卷 {} 不存在", volume_id))
            }
            _ => CliError::storage(format!("查询卷状态失败：{}", e)),
        })?;

    if status == "archived" {
        return Err(CliError::volume_already_archived(format!(
            "卷 {} 已归档，下属实体不可修改",
            volume_id
        )));
    }

    Ok(())
}

/// 通过章节 ID 检查其所属卷是否处于冻结态（archived）。
///
/// 用于章节级写操作（chapter update/status、emergence rate/decide/execute、verdict set 等）。
/// 查询章节所属 volume_id，再调用 `ensure_volume_not_archived`。
/// 章节无所属卷（volume_id 为 NULL）时不阻断（无所属卷则无冻结）。
pub fn ensure_chapter_volume_not_archived(
    conn: &Connection,
    chapter_id: &str,
) -> Result<(), CliError> {
    let volume_id: Option<i64> = conn
        .query_row(
            "SELECT volume_id FROM chapters WHERE id = ?1",
            [chapter_id],
            |row| row.get(0),
        )
        .ok();

    match volume_id {
        Some(vid) => ensure_volume_not_archived(conn, vid),
        None => Ok(()), // 无所属卷，无冻结约束
    }
}

/// 通过涌现 ID 检查其所属卷是否处于冻结态（archived）。
///
/// 用于涌现级写操作（emergence rate/decide/execute）。
pub fn ensure_emergence_volume_not_archived(
    conn: &Connection,
    emergence_id: &str,
) -> Result<(), CliError> {
    let volume_id: i64 = conn
        .query_row(
            "SELECT volume_id FROM emergences WHERE id = ?1",
            [emergence_id],
            |row| row.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CliError::emergence_not_found(format!("涌现 '{}' 不存在", emergence_id))
            }
            _ => CliError::storage(format!("查询涌现所属卷失败：{}", e)),
        })?;

    ensure_volume_not_archived(conn, volume_id)
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
        run_migrations(&conn, "2026-07-01T00:00:00Z").unwrap();
        (conn, dir)
    }

    #[test]
    fn test_ensure_volume_not_archived_active_ok() {
        let (conn, _dir) = setup();
        conn.execute(
            "INSERT INTO volumes (name, volume_order, status, created_at, updated_at) \
             VALUES ('第一卷', 1, 'active', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [],
        )
        .unwrap();
        assert!(ensure_volume_not_archived(&conn, 1).is_ok());
    }

    #[test]
    fn test_ensure_volume_not_archived_archived_blocked() {
        let (conn, _dir) = setup();
        conn.execute(
            "INSERT INTO volumes (name, volume_order, status, created_at, updated_at) \
             VALUES ('第一卷', 1, 'archived', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let result = ensure_volume_not_archived(&conn, 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().exit_code(), crate::errors::ExitCode::PreconditionBlocked);
    }

    #[test]
    fn test_ensure_chapter_volume_not_archived_blocks() {
        let (conn, _dir) = setup();
        conn.execute(
            "INSERT INTO volumes (name, volume_order, status, created_at, updated_at) \
             VALUES ('第一卷', 1, 'archived', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
             VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'h', 10, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let result = ensure_chapter_volume_not_archived(&conn, "chapter-001");
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_emergence_volume_not_archived_blocks() {
        let (conn, _dir) = setup();
        conn.execute(
            "INSERT INTO volumes (name, volume_order, status, created_at, updated_at) \
             VALUES ('第一卷', 1, 'archived', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at) \
             VALUES ('em-1-1', 1, '涌现A', '描述', '2026-07-01T00:00:00Z', 'pending', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let result = ensure_emergence_volume_not_archived(&conn, "em-1-1");
        assert!(result.is_err());
    }
}
