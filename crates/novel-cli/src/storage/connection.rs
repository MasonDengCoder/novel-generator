// SQLite 连接、migration runner 与 PRAGMA user_version（schemaVersion 单一数据源）。
//
// 职责（CLAUDE.md §6、§7、design D2、D-risk）：自研轻量 migration runner，逐 SQL 文件在
// 事务内执行，失败回滚；schemaVersion 单一数据源 = PRAGMA user_version，避免双轨。
//
// SQLite schema、迁移文件名均为非稳定实现细节（CLAUDE.md §9）。

use std::path::Path;
use std::sync::OnceLock;
use rusqlite::Connection;

use crate::errors::CliError;

/// 当前世界状态 schema 版本（与 migration 数量一致）。每加一个 migration 文件递增。
pub const CURRENT_SCHEMA_VERSION: u32 = 9;

/// 内嵌的 migration 文件清单，按版本号升序。
fn migrations() -> &'static [(u32, &'static str, &'static str)] {
    static M: OnceLock<Vec<(u32, &'static str, &'static str)>> = OnceLock::new();
    M.get_or_init(|| vec![
        // (版本, 文件名, SQL 内容)
        (1, "V001__initial.sql", include_str!("migrations/V001__initial.sql")),
        (2, "V002__settings_and_constraints.sql", include_str!("migrations/V002__settings_and_constraints.sql")),
        (3, "V003__chapters_table.sql", include_str!("migrations/V003__chapters_table.sql")),
        (4, "V004__rule_checks.sql", include_str!("migrations/V004__rule_checks.sql")),
        (5, "V005__volumes.sql", include_str!("migrations/V005__volumes.sql")),
        (6, "V006__verdicts.sql", include_str!("migrations/V006__verdicts.sql")),
        (7, "V007__destiny_weaving.sql", include_str!("migrations/V007__destiny_weaving.sql")),
        (8, "V008__volume_monitoring_report.sql", include_str!("migrations/V008__volume_monitoring_report.sql")),
        (9, "V009__constraint_severity_three_tier.sql", include_str!("migrations/V009__constraint_severity_three_tier.sql")),
    ])
}

/// 读取已应用的 migration 版本集合。schema_migrations 表由首个 migration 创建，
/// 若表不存在则视为尚未应用任何 migration。
fn applied_version_set(conn: &Connection) -> Result<std::collections::HashSet<u32>, rusqlite::Error> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_migrations')",
            [],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )?;
    if !exists {
        return Ok(Default::default());
    }
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations")?;
    let rows = stmt.query_map([], |r| r.get::<_, i32>(0))?;
    let mut set = std::collections::HashSet::new();
    for v in rows {
        set.insert(v? as u32);
    }
    Ok(set)
}

/// 在事务内逐文件执行未应用的 migration，并把 user_version 设为最新版本。
/// 任一文件失败则回滚整批。
pub fn run_migrations(conn: &Connection, now_iso: &str) -> Result<(), CliError> {
    let applied = applied_version_set(conn)
        .map_err(|e| CliError::storage(format!("读取 schema_migrations 失败: {e}")))?;

    // 显式开启事务。
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| CliError::storage(format!("开启 migration 事务失败: {e}")))?;

    for (version, filename, sql) in migrations() {
        if applied.contains(version) {
            continue;
        }
        tx.execute_batch(sql).map_err(|e| {
            // 事务会在 drop 时回滚；这里返回错误让上层决定。
            CliError::storage(format!("执行 migration {filename} (v{version}) 失败: {e}"))
        })?;
        tx.execute(
            "INSERT INTO schema_migrations (version, filename, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![*version as i64, filename, now_iso],
        )
        .map_err(|e| CliError::storage(format!("记录 migration {filename} 失败: {e}")))?;
    }

    // 提交 migration 事务后，将 PRAGMA user_version 设为当前版本（schemaVersion 单一数据源）。
    // user_version 独立于事务，单独设置即可；migration 本身已在事务内原子落库。
    tx.commit()
        .map_err(|e| CliError::storage(format!("提交 migration 事务失败: {e}")))?;

    conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_SCHEMA_VERSION}"))
        .map_err(|e| CliError::storage(format!("设置 user_version 失败: {e}")))?;

    Ok(())
}

/// 读取 schemaVersion（PRAGMA user_version），返回字符串形态。
pub fn read_schema_version(conn: &Connection) -> Result<String, CliError> {
    let v: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| CliError::storage(format!("读取 user_version 失败: {e}")))?;
    Ok(v.to_string())
}

/// 打开（或创建）指定路径的 SQLite 数据库文件，启用 WAL、外键与 busy_timeout。
///
/// WAL（write-ahead logging）持久化于数据库文件头，`open` 时执行一次即对所有后续连接
/// 生效；但本函数显式执行以使 docstring 与实现一致。`busy_timeout` 配合写路径的
/// `BEGIN IMMEDIATE`，使绕过 CLI 的裸 SQLite 写入在 CLI 持锁期间等待超时后失败。
pub fn open(db_path: &std::path::Path) -> Result<Connection, CliError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CliError::storage(format!("打开 SQLite 数据库失败: {e}")))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 3000;",
    )
    .map_err(|e| CliError::storage(format!("设置 PRAGMA (WAL/foreign_keys/busy_timeout) 失败: {e}")))?;
    Ok(conn)
}

/// 打开指定世界目录下的 .novel/novel.db，如果不存在则返回 PreconditionBlocked 错误（退出码 3）。
/// 用于所有需要世界已初始化的命令。
pub fn open_existing_at(target: &Path) -> Result<Connection, CliError> {
    let db_path = target.join(".novel").join("novel.db");

    if !db_path.exists() {
        return Err(CliError::world_not_initialized(
            "世界未初始化：目标目录下不存在 .novel/novel.db。请先运行 `novel init <project-name>` 初始化世界。"
                .to_string(),
        ));
    }

    open(&db_path)
}

/// 打开当前目录下的 .novel/novel.db，如果不存在则返回 PreconditionBlocked 错误（退出码 3）。
pub fn open_existing() -> Result<Connection, CliError> {
    let target = std::env::current_dir()
        .map_err(|e| CliError::storage(format!("获取当前目录失败: {e}")))?;
    open_existing_at(&target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// 读取连接的 PRAGMA 值。
    fn pragma_value(conn: &Connection, key: &str) -> String {
        conn.query_row(&format!("PRAGMA {key}"), [], |r| {
            let v: std::result::Result<String, rusqlite::Error> = r.get(0);
            // journal_mode 返回 TEXT，foreign_keys 返回 INTEGER；统一转字符串。
            v.or_else(|_| {
                let i: i64 = r.get(0)?;
                Ok(i.to_string())
            })
        })
        .unwrap()
    }

    #[test]
    fn test_open_enables_wal_mode() {
        // F-05：open 执行后 journal_mode 返回 wal（而非默认 delete），foreign_keys 返回 1。
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("novel.db");

        let conn = open(&db_path).unwrap();

        assert_eq!(
            pragma_value(&conn, "journal_mode"),
            "wal",
            "open 应启用 WAL（journal_mode=wal，非 delete）"
        );
        assert_eq!(
            pragma_value(&conn, "foreign_keys"),
            "1",
            "open 应启用外键（foreign_keys=1）"
        );
    }

    #[test]
    fn test_open_existing_at_enables_wal() {
        // F-05：经 open_existing_at 打开已初始化世界，该连接 journal_mode 为 wal。
        let dir = tempdir().unwrap();
        let project_dir = dir.path();
        let novel_dir = project_dir.join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();

        // 初始化世界：创建 novel.db 并跑 migration（模拟 init 后状态）
        {
            let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
        }

        let conn = open_existing_at(project_dir).unwrap();
        assert_eq!(
            pragma_value(&conn, "journal_mode"),
            "wal",
            "open_existing_at 应启用 WAL"
        );
        assert_eq!(
            pragma_value(&conn, "foreign_keys"),
            "1",
            "open_existing_at 应启用外键"
        );
    }
}
