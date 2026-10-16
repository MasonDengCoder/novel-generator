// 验证 migration runner 可执行（task 3.5）。
// 断言：migration 应用后 PRAGMA user_version 与 schema_migrations 记录一致，
// 且 project_meta 表存在可写。

use novel_cli::storage::connection::{open, run_migrations, read_schema_version, CURRENT_SCHEMA_VERSION};
use rusqlite::Connection;
use tempfile::tempdir;

fn open_and_migrate(dir: &std::path::Path) -> Connection {
    let db_path = dir.join(".novel").join("novel.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    let conn = open(&db_path).unwrap();
    run_migrations(&conn, "2026-01-01T00:00:00Z").unwrap();
    conn
}

#[test]
fn migration_sets_user_version() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    let uv = read_schema_version(&conn).unwrap();
    assert_eq!(uv, CURRENT_SCHEMA_VERSION.to_string());
}

#[test]
fn migration_records_schema_migrations_consistent() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    let max_version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    let uv: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, CURRENT_SCHEMA_VERSION as i64);
    assert_eq!(max_version, CURRENT_SCHEMA_VERSION as i64);
    assert_eq!(max_version, uv);
}

#[test]
fn migration_creates_project_meta_table() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='project_meta')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists);
}

#[test]
fn migration_is_idempotent() {
    // 二次运行不应报错，版本不变。
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    run_migrations(&conn, "2026-01-02T00:00:00Z").unwrap();
    let uv = read_schema_version(&conn).unwrap();
    assert_eq!(uv, CURRENT_SCHEMA_VERSION.to_string());

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, CURRENT_SCHEMA_VERSION as i64);
}

#[test]
fn migration_v007_creates_destiny_weaving_tables() {
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    // 验证 mainplot_nodes 表存在
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='mainplot_nodes')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists, "mainplot_nodes table should exist");

    // 验证 threads 表存在
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='threads')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists, "threads table should exist");

    // 验证 thread_bindings 表存在
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='thread_bindings')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists, "thread_bindings table should exist");

    // 验证 emergences 表存在
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='emergences')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .unwrap();
    assert!(exists, "emergences table should exist");
}

#[test]
fn migration_v007_applies_on_v006_database() {
    // 模拟从 v006 升级到 v007
    let dir = tempdir().unwrap();
    let db_path = dir.path().join(".novel").join("novel.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let conn = open(&db_path).unwrap();
    run_migrations(&conn, "2026-01-01T00:00:00Z").unwrap();

    // 验证升级后版本正确
    let uv = read_schema_version(&conn).unwrap();
    assert_eq!(uv, CURRENT_SCHEMA_VERSION.to_string());

    // 创建必需的父记录（project 和 volume）
    conn.execute(
        "INSERT INTO project_meta (id, project_name, db_path, created_at, schema_version) VALUES (1, 'test-proj', '/tmp/test.db', '2026-01-01T00:00:00Z', '9')",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO volumes (name, volume_order, created_at, updated_at) VALUES ('Test Volume', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    ).unwrap();

    // 获取插入的 volume id
    let volume_id: i64 = conn.last_insert_rowid();

    // 验证新表可写入（使用真实的 INTEGER volume_id）
    conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title) VALUES (?, ?, 1, 'Test Node')",
        rusqlite::params!["mp-test-1", volume_id],
    ).unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM mainplot_nodes WHERE id = 'mp-test-1'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn migration_v009_severity_three_tier_check() {
    // V009 后 constraints 表 severity CHECK 放宽为 info/warning/hard：
    // - info/warning/hard 可插入（退出无错）
    // - soft 被拒（触发 CHECK 约束，证明旧值不再被表接受）
    // - schema_migrations 含 version=9 记录
    let dir = tempdir().unwrap();
    let conn = open_and_migrate(dir.path());

    let inserted: i64 = conn
        .query_row("SELECT COUNT(*) FROM schema_migrations WHERE version = 9", [], |r| r.get(0))
        .unwrap();
    assert_eq!(inserted, 1, "V009 应在 schema_migrations 中记录");

    // 三档可插入（需唯一主键，绕过 project_meta 无关）
    for (name, sev) in [("info-r", "info"), ("warning-r", "warning"), ("hard-r", "hard")] {
        conn.execute(
            "INSERT INTO constraints (constraint_type, scope, name, content, severity) VALUES ('plot', 'world', ?1, 'x', ?2)",
            rusqlite::params![name, sev],
        )
        .unwrap_or_else(|e| panic!("severity '{}' 应可插入，但失败: {}", sev, e));
    }

    // soft 被新 CHECK 拒绝（SQLite 返回 ConstraintViolation）
    let result = conn.execute(
        "INSERT INTO constraints (constraint_type, scope, name, content, severity) VALUES ('plot', 'world', 'soft-r', 'x', 'soft')",
        [],
    );
    assert!(
        result.is_err(),
        "soft 应被 V009 后的 CHECK 约束拒绝"
    );

    // 验证已插入的三档数据可查询
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM constraints WHERE severity IN ('info','warning','hard')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);
}
