// storage/chapters 模块：章节相关 SQL 操作封装（task 5.1）。

use crate::errors::CliError;
use rusqlite::{Connection, params, OptionalExtension};

/// 章节元数据结构（对应 SQLite chapters 表）。
#[derive(Debug, Clone)]
pub struct ChapterRow {
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

/// 插入新章节记录。
///
/// `volume_id` 为 `Some` 时把章节归属到指定卷（写入既有 `chapters.volume_id` 外键列），
/// 为 `None` 时 `volume_id` 留空（沿用既有不归属行为）。卷的存在性与状态校验由 service 层完成。
pub fn insert_chapter(
    conn: &Connection,
    id: &str,
    title: &str,
    content_path: &str,
    content_hash: &str,
    word_count: i64,
    volume_id: Option<i64>,
    created_at: &str,
) -> Result<(), CliError> {
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, volume_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![id, title, content_path, content_hash, word_count, volume_id, created_at],
    )
    .map_err(|e| CliError::storage(format!("插入章节记录失败：{}", e)))?;

    Ok(())
}

/// 更新章节元数据（正文 hash、字数、更新时间）。
pub fn update_chapter_metadata(
    conn: &Connection,
    id: &str,
    content_hash: &str,
    word_count: i64,
    updated_at: &str,
) -> Result<(), CliError> {
    let rows_affected = conn
        .execute(
            "UPDATE chapters SET content_hash = ?1, word_count = ?2, updated_at = ?3 WHERE id = ?4",
            params![content_hash, word_count, updated_at, id],
        )
        .map_err(|e| CliError::storage(format!("更新章节元数据失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::chapter_not_found(
            format!("章节不存在：{}", id),
            Some("使用 `novel chapter list` 查看所有章节。".to_string()),
        ));
    }

    Ok(())
}

/// 更新章节状态（status + updated_at）。
///
/// 在事务中由 chapter_state_service 调用，门禁通过后推进状态。
/// 返回 chapter_not_found 当 rows_affected=0。
pub fn update_status(
    conn: &Connection,
    id: &str,
    status: &str,
    updated_at: &str,
) -> Result<(), CliError> {
    let rows_affected = conn
        .execute(
            "UPDATE chapters SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, updated_at, id],
        )
        .map_err(|e| CliError::storage(format!("更新章节状态失败：{}", e)))?;

    if rows_affected == 0 {
        return Err(CliError::chapter_not_found(
            format!("章节不存在：{}", id),
            Some("使用 `novel chapter list` 查看所有章节。".to_string()),
        ));
    }

    Ok(())
}

/// 读取单个章节元数据。
pub fn get_chapter(conn: &Connection, id: &str) -> Result<ChapterRow, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at FROM chapters WHERE id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询章节语句失败：{}", e)))?;

    let row = stmt
        .query_row(params![id], |row| {
            Ok(ChapterRow {
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
        .map_err(|e| {
            if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                CliError::chapter_not_found(
                    format!("章节不存在：{}", id),
                    Some("使用 `novel chapter list` 查看所有章节。".to_string()),
                )
            } else {
                CliError::storage(format!("查询章节失败：{}", e))
            }
        })?;

    Ok(row)
}

/// 读取章节元数据（Option 版本）。
pub fn get_chapter_metadata(conn: &Connection, id: &str) -> Result<Option<ChapterRow>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at FROM chapters WHERE id = ?1")
        .map_err(|e| CliError::storage(format!("准备查询章节语句失败：{}", e)))?;

    let row = stmt
        .query_row(params![id], |row| {
            Ok(ChapterRow {
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
        .optional()
        .map_err(|e| CliError::storage(format!("查询章节失败：{}", e)))?;

    Ok(row)
}

/// 列出所有章节元数据（按创建时间升序）。
pub fn list_chapters(conn: &Connection) -> Result<Vec<ChapterRow>, CliError> {
    let mut stmt = conn
        .prepare("SELECT id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at FROM chapters ORDER BY created_at ASC")
        .map_err(|e| CliError::storage(format!("准备列出章节语句失败：{}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ChapterRow {
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
        .map_err(|e| CliError::storage(format!("查询章节列表失败：{}", e)))?;

    let mut chapters = Vec::new();
    for row in rows {
        chapters.push(row.map_err(|e| CliError::storage(format!("读取章节行失败：{}", e)))?);
    }

    Ok(chapters)
}

/// 生成下一个章节 ID（chapter-NNN 格式）。
pub fn generate_next_chapter_id(conn: &Connection) -> Result<String, CliError> {
    let max_id: Option<String> = conn
        .query_row(
            "SELECT id FROM chapters ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| CliError::storage(format!("查询最大章节 ID 失败：{}", e)))?;

    let next_num = if let Some(id) = max_id {
        // 提取数字部分并递增
        let num_str = id.strip_prefix("chapter-").unwrap_or("000");
        let num: u32 = num_str.parse().unwrap_or(0);
        num + 1
    } else {
        1
    };

    Ok(format!("chapter-{:03}", next_num))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::{run_migrations};
    use rusqlite::Connection;
    use tempfile::{tempdir, TempDir};

    fn setup_db() -> (Connection, TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
        (conn, dir)
    }

    #[test]
    fn test_generate_first_chapter_id() {
        let (conn, _dir) = setup_db();
        let id = generate_next_chapter_id(&conn).unwrap();
        assert_eq!(id, "chapter-001");
    }

    #[test]
    fn test_insert_and_get_chapter() {
        let (conn, _dir) = setup_db();

        insert_chapter(
            &conn,
            "chapter-001",
            "测试章节",
            ".novel/content/chapters/chapter-001.md",
            "hash123",
            100,
            None,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();

        let row = get_chapter(&conn, "chapter-001").unwrap();
        assert_eq!(row.id, "chapter-001");
        assert_eq!(row.title, "测试章节");
        assert_eq!(row.word_count, 100);
    }

    #[test]
    fn test_update_chapter_metadata() {
        let (conn, _dir) = setup_db();

        insert_chapter(
            &conn,
            "chapter-001",
            "测试",
            "path",
            "hash1",
            100,
            None,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();

        update_chapter_metadata(&conn, "chapter-001", "hash2", 200, "2026-06-29T01:00:00Z").unwrap();

        let row = get_chapter(&conn, "chapter-001").unwrap();
        assert_eq!(row.content_hash, "hash2");
        assert_eq!(row.word_count, 200);
        assert_eq!(row.updated_at, "2026-06-29T01:00:00Z");
    }

    #[test]
    fn test_update_status_ok() {
        let (conn, _dir) = setup_db();

        insert_chapter(
            &conn,
            "chapter-001",
            "测试",
            "path",
            "hash1",
            100,
            None,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();

        // 默认状态为 outline
        let row = get_chapter(&conn, "chapter-001").unwrap();
        assert_eq!(row.status, "outline");

        update_status(&conn, "chapter-001", "drafting", "2026-06-29T01:00:00Z").unwrap();

        let row = get_chapter(&conn, "chapter-001").unwrap();
        assert_eq!(row.status, "drafting");
        assert_eq!(row.updated_at, "2026-06-29T01:00:00Z");
    }

    #[test]
    fn test_update_status_chapter_not_found() {
        let (conn, _dir) = setup_db();

        let result = update_status(&conn, "chapter-999", "drafting", "2026-06-29T01:00:00Z");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_chapters() {
        let (conn, _dir) = setup_db();

        insert_chapter(&conn, "chapter-001", "第一章", "p1", "h1", 100, None, "2026-06-29T00:00:00Z").unwrap();
        insert_chapter(&conn, "chapter-002", "第二章", "p2", "h2", 200, None, "2026-06-29T01:00:00Z").unwrap();

        let chapters = list_chapters(&conn).unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].id, "chapter-001");
        assert_eq!(chapters[1].id, "chapter-002");
    }

    #[test]
    fn test_get_nonexistent_chapter() {
        let (conn, _dir) = setup_db();
        let result = get_chapter(&conn, "chapter-999");
        assert!(result.is_err());
    }
}
