// 测试公共辅助：建立已迁移的临时数据库与测试卷、章节。
// 被 s06 service / validator 集成测试复用。
//
// service 层（s03 后）持有 DB-open：公开函数收 `target: &Path`，内部 `open_existing_at(target)`
// 解析到 `target/.novel/novel.db`。故 helper 在 `<tmp>/.novel/novel.db` 建库，并返回 `target: PathBuf`
//（世界目录）；保留 `conn` 供测试直接操作 DB / validator（不走 service 入口时）。
// tempdir 被泄漏（std::mem::forget）以保证测试期间文件存活。

use novel_cli::storage::connection::{open, run_migrations};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use tempfile::tempdir;

/// 建立已迁移的临时数据库，插入一个测试卷，返回 (conn, target, volume_id)。
pub fn setup_db() -> (Connection, PathBuf, i64) {
    let dir = tempdir().unwrap();
    let novel_dir = dir.path().join(".novel");
    std::fs::create_dir_all(&novel_dir).unwrap();
    let db_path = novel_dir.join("novel.db");
    let target = dir.path().to_path_buf();
    std::mem::forget(dir);
    let conn = open(&db_path).unwrap();
    run_migrations(&conn, "2026-01-01T00:00:00Z").unwrap();

    conn.execute(
        "INSERT INTO volumes (name, volume_order, created_at, updated_at) VALUES ('测试卷', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    let volume_id: i64 = conn.last_insert_rowid();

    (conn, target, volume_id)
}

/// 建立已迁移的临时数据库，插入一个测试卷与一个章节，返回 (conn, target, volume_id, chapter_id)。
pub fn setup_db_with_chapter() -> (Connection, PathBuf, i64, String) {
    let (conn, target, volume_id) = setup_db();

    let chapter_id = "ch-001".to_string();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, volume_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![chapter_id, "第一章", "chapters/ch-001.md", "hash001", volume_id],
    )
    .unwrap();

    (conn, target, volume_id, chapter_id)
}

/// 建立已迁移的临时数据库，插入一个测试卷与两个章节，返回 (conn, target, volume_id, ch1, ch2)。
pub fn setup_db_with_two_chapters() -> (Connection, PathBuf, i64, String, String) {
    let (conn, target, volume_id) = setup_db();

    let ch1 = "ch-001".to_string();
    let ch2 = "ch-002".to_string();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, volume_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![ch1, "第一章", "chapters/ch-001.md", "hash001", volume_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, volume_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![ch2, "第二章", "chapters/ch-002.md", "hash002", volume_id],
    )
    .unwrap();

    (conn, target, volume_id, ch1, ch2)
}
