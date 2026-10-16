// 验证 V003 migration 创建 chapters 表（task 1.2 & 1.3）。

use novel_cli::storage::connection::{open, run_migrations};
use rusqlite::Connection;
use tempfile::tempdir;

fn open_and_migrate(dir: &std::path::Path) -> Connection {
    let db_path = dir.join(".novel").join("novel.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let conn = open(&db_path).unwrap();
    run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
    conn
}

#[test]
fn migration_v003_creates_chapters_table() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='chapters')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists, "chapters 表应该被创建");
}

#[test]
fn migration_v003_creates_required_columns() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    // 验证所有必需字段存在
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(chapters)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"title".to_string()));
    assert!(columns.contains(&"content_path".to_string()));
    assert!(columns.contains(&"content_hash".to_string()));
    assert!(columns.contains(&"word_count".to_string()));
    assert!(columns.contains(&"created_at".to_string()));
    assert!(columns.contains(&"updated_at".to_string()));
}

#[test]
fn migration_v003_creates_indexes() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='chapters'")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(indexes.iter().any(|name| name.contains("created_at")),
        "应该创建 created_at 索引");
    assert!(indexes.iter().any(|name| name.contains("word_count")),
        "应该创建 word_count 索引");
}

#[test]
fn migration_v003_is_idempotent() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    // 二次运行不应报错
    run_migrations(&conn, "2026-06-29T01:00:00Z").unwrap();

    // 验证表仍然存在且结构正确
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='chapters')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists);

    // 验证没有重复记录
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = 3",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "V003 migration 应该只记录一次");
}
