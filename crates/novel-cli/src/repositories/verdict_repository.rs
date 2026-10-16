// verdict_repository 模块：封装主线落地裁决 CRUD 操作（s04-state-machine-and-verdicts）。
//
// 职责（CLAUDE.md §6、§7）：SQL 与裁决读写封装，不做业务判断。
// 裁决绑定章节正文 hash，作为状态推进门禁的输入之一。

use crate::errors::CliError;
use rusqlite::{Connection, OptionalExtension, params};
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 裁决记录（对应 verdicts 表行）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    pub id: i64,
    pub chapter_id: String,
    pub verdict_type: String,
    pub node_id: String,
    pub achieved: bool,
    pub content_hash: String,
    pub created_at: String,
}

/// 写入（UPSERT）主线落地裁决。
///
/// 同一 `(chapter_id, verdict_type, node_id)` 的新裁决覆盖旧裁决（INSERT OR REPLACE），
/// `content_hash` 由 service 读取当前 `chapters.content_hash` 填入，反映当前正文版本。
///
/// 参数：
/// - conn: SQLite 连接（service 在事务内传入 tx，Transaction deref 到 Connection）。
/// - chapter_id: 章节 ID。
/// - verdict_type: 裁决类型（本切片仅 'mainplot-landing'）。
/// - node_id: 主线节点 ID。
/// - achieved: 是否达成。
/// - content_hash: 当前正文 hash（版本锚点）。
/// - now_iso: 当前时间戳（ISO 8601）。
pub fn upsert_verdict(
    conn: &Connection,
    chapter_id: &str,
    verdict_type: &str,
    node_id: &str,
    achieved: bool,
    content_hash: &str,
    now_iso: &str,
) -> Result<(), CliError> {
    conn.execute(
        "INSERT OR REPLACE INTO verdicts
         (chapter_id, verdict_type, node_id, achieved, content_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            chapter_id,
            verdict_type,
            node_id,
            if achieved { 1 } else { 0 },
            content_hash,
            now_iso,
        ],
    )
    .map_err(|e| CliError::storage(format!("写入裁决失败: {e}")))?;

    Ok(())
}

/// 查询指定章节、类型、节点的主线落地裁决。
pub fn get_verdict(
    conn: &Connection,
    chapter_id: &str,
    verdict_type: &str,
    node_id: &str,
) -> Result<Option<Verdict>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, chapter_id, verdict_type, node_id, achieved, content_hash, created_at
             FROM verdicts
             WHERE chapter_id = ?1 AND verdict_type = ?2 AND node_id = ?3",
        )
        .map_err(|e| CliError::storage(format!("准备查询裁决语句失败: {e}")))?;

    let row = stmt
        .query_row(params![chapter_id, verdict_type, node_id], |row| {
            Ok(Verdict {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                verdict_type: row.get(2)?,
                node_id: row.get(3)?,
                achieved: row.get::<_, i32>(4)? != 0,
                content_hash: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .optional()
        .map_err(|e| CliError::storage(format!("查询裁决失败: {e}")))?;

    Ok(row)
}

/// 列出指定章节的全部裁决（chapter-context-packet 切片：P0 装填用）。
///
/// 返回该章节所有 verdict_type / node_id 的裁决，按 created_at 升序。service 层据
/// verdict_type/achieved/content_hash 自行过滤。只读，不做业务判断。
pub fn list_verdicts_by_chapter(conn: &Connection, chapter_id: &str) -> Result<Vec<Verdict>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, chapter_id, verdict_type, node_id, achieved, content_hash, created_at
             FROM verdicts
             WHERE chapter_id = ?1
             ORDER BY created_at ASC",
        )
        .map_err(|e| CliError::storage(format!("准备列出裁决语句失败: {e}")))?;

    let rows = stmt
        .query_map(params![chapter_id], |row| {
            Ok(Verdict {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                verdict_type: row.get(2)?,
                node_id: row.get(3)?,
                achieved: row.get::<_, i32>(4)? != 0,
                content_hash: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| CliError::storage(format!("列出裁决失败: {e}")))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| CliError::storage(format!("读取裁决行失败: {e}")))?);
    }
    Ok(out)
}

/// 判断是否存在「绑定当前正文 hash 且 achieved=1」的 mainplot-landing 裁决。
///
/// 用于状态推进门禁：进入 review/approved 前验证裁决已达成且未 stale。
///
/// 参数：
/// - conn: SQLite 连接。
/// - chapter_id: 章节 ID。
/// - expected_hash: 当前正文 hash（与裁决 content_hash 比对，stale 检测）。
///
/// 返回：true 表示存在绑定当前 hash 且 achieved=1 的裁决。
pub fn has_achieved_mainplot_landing(
    conn: &Connection,
    chapter_id: &str,
    expected_hash: &str,
) -> Result<bool, CliError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM verdicts
             WHERE chapter_id = ?1 AND verdict_type = 'mainplot-landing'
               AND achieved = 1 AND content_hash = ?2",
            params![chapter_id, expected_hash],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询已达成裁决失败: {e}")))?;

    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::chapters::insert_chapter;
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

    fn seed_chapter(conn: &Connection, id: &str, hash: &str) {
        insert_chapter(
            conn,
            id,
            "测试章节",
            &format!("content/chapters/{}.md", id),
            hash,
            100,
            None,
            "2026-07-01T00:00:00Z",
        )
        .unwrap();
    }

    #[test]
    fn test_upsert_and_get_verdict() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");

        upsert_verdict(
            &conn,
            "chapter-001",
            "mainplot-landing",
            "mp-1-001",
            true,
            "hash-001",
            "2026-07-01T01:00:00Z",
        )
        .unwrap();

        let verdict = get_verdict(&conn, "chapter-001", "mainplot-landing", "mp-1-001")
            .unwrap()
            .expect("裁决应存在");
        assert!(verdict.achieved);
        assert_eq!(verdict.content_hash, "hash-001");
        assert_eq!(verdict.node_id, "mp-1-001");
    }

    #[test]
    fn test_upsert_overwrites_same_node() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");

        // 第一次写入 achieved=0
        upsert_verdict(
            &conn,
            "chapter-001",
            "mainplot-landing",
            "mp-1-001",
            false,
            "hash-001",
            "2026-07-01T01:00:00Z",
        )
        .unwrap();

        // 同节点覆盖为 achieved=1 + 新 hash
        upsert_verdict(
            &conn,
            "chapter-001",
            "mainplot-landing",
            "mp-1-001",
            true,
            "hash-002",
            "2026-07-01T02:00:00Z",
        )
        .unwrap();

        let verdict = get_verdict(&conn, "chapter-001", "mainplot-landing", "mp-1-001")
            .unwrap()
            .expect("裁决应存在");
        assert!(verdict.achieved);
        assert_eq!(verdict.content_hash, "hash-002");
    }

    #[test]
    fn test_has_achieved_mainplot_landing_true() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");

        upsert_verdict(
            &conn,
            "chapter-001",
            "mainplot-landing",
            "mp-1-001",
            true,
            "hash-001",
            "2026-07-01T01:00:00Z",
        )
        .unwrap();

        assert!(has_achieved_mainplot_landing(&conn, "chapter-001", "hash-001").unwrap());
    }

    #[test]
    fn test_has_achieved_mainplot_landing_stale_hash() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");

        // 裁决绑定旧 hash
        upsert_verdict(
            &conn,
            "chapter-001",
            "mainplot-landing",
            "mp-1-001",
            true,
            "hash-old",
            "2026-07-01T01:00:00Z",
        )
        .unwrap();

        // 当前正文 hash 为 hash-001，裁决绑定 hash-old → stale → 不算达成
        assert!(!has_achieved_mainplot_landing(&conn, "chapter-001", "hash-001").unwrap());
    }

    #[test]
    fn test_has_achieved_mainplot_landing_not_achieved() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");

        upsert_verdict(
            &conn,
            "chapter-001",
            "mainplot-landing",
            "mp-1-001",
            false,
            "hash-001",
            "2026-07-01T01:00:00Z",
        )
        .unwrap();

        assert!(!has_achieved_mainplot_landing(&conn, "chapter-001", "hash-001").unwrap());
    }

    #[test]
    fn test_has_achieved_mainplot_landing_missing() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");

        // 未写入裁决
        assert!(!has_achieved_mainplot_landing(&conn, "chapter-001", "hash-001").unwrap());
    }
}
