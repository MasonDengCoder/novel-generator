// rule_check_repository 模块：封装规则检查结果 CRUD 操作。
//
// 职责：SQL 与检查结果读写封装，不做业务判断（CLAUDE.md §6、§7）。

use crate::errors::CliError;
use rusqlite::{Connection, params};
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 规则检查结果（对应 rule_checks 表行）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuleCheck {
    pub id: i64,
    pub chapter_id: String,
    pub constraint_type: String,
    pub constraint_scope: String,
    pub constraint_name: String,
    pub content_hash: String,
    pub severity: String,
    pub passed: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub checked_at: String,
}

/// 待插入的检查结果（不含自增 id 和时间戳）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResultInput {
    pub constraint_type: String,
    pub constraint_scope: String,
    pub constraint_name: String,
    pub severity: String,
    pub passed: bool,
    pub description: Option<String>,
    pub location: Option<String>,
}

/// 原子替换章节的当前有效检查集。
///
/// 参数：
/// - conn: SQLite 连接
/// - chapter_id: 章节 ID
/// - content_hash: 当前章节正文 hash
/// - checks: 完整的新检查结果数组
/// - now_iso: 当前时间戳（ISO 8601 格式）
pub fn insert_check_results(
    conn: &Connection,
    chapter_id: &str,
    content_hash: &str,
    checks: &[CheckResultInput],
    now_iso: &str,
) -> Result<(), CliError> {
    let tx = conn.unchecked_transaction()
        .map_err(|e| CliError::storage(format!("开启检查结果写入事务失败: {e}")))?;

    tx.execute(
        "DELETE FROM rule_checks WHERE chapter_id = ?1",
        params![chapter_id],
    )
    .map_err(|e| CliError::storage(format!("替换检查结果时删除旧检查集失败: {e}")))?;

    for check in checks {
        tx.execute(
            "INSERT OR REPLACE INTO rule_checks
             (chapter_id, constraint_type, constraint_scope, constraint_name, content_hash, severity, passed, description, location, checked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                chapter_id,
                &check.constraint_type,
                &check.constraint_scope,
                &check.constraint_name,
                content_hash,
                &check.severity,
                if check.passed { 1 } else { 0 },
                &check.description,
                &check.location,
                now_iso,
            ],
        )
        .map_err(|e| CliError::storage(format!("插入检查结果失败: {e}")))?;
    }

    tx.commit()
        .map_err(|e| CliError::storage(format!("提交检查结果事务失败: {e}")))?;

    Ok(())
}

/// 查询指定章节的所有检查结果。
///
/// 参数：
/// - conn: SQLite 连接
/// - chapter_id: 章节 ID
///
/// 返回：检查结果数组（按 checked_at 降序）。
pub fn list_check_results(
    conn: &Connection,
    chapter_id: &str,
) -> Result<Vec<RuleCheck>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, chapter_id, constraint_type, constraint_scope, constraint_name,
                    content_hash, severity, passed, description, location, checked_at
             FROM rule_checks
             WHERE chapter_id = ?1
             ORDER BY checked_at DESC",
        )
        .map_err(|e| CliError::storage(format!("准备查询检查结果语句失败: {e}")))?;

    let rows = stmt
        .query_map(params![chapter_id], |row| {
            Ok(RuleCheck {
                id: row.get(0)?,
                chapter_id: row.get(1)?,
                constraint_type: row.get(2)?,
                constraint_scope: row.get(3)?,
                constraint_name: row.get(4)?,
                content_hash: row.get(5)?,
                severity: row.get(6)?,
                passed: row.get::<_, i32>(7)? != 0,
                description: row.get(8)?,
                location: row.get(9)?,
                checked_at: row.get(10)?,
            })
        })
        .map_err(|e| CliError::storage(format!("查询检查结果失败: {e}")))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| CliError::storage(format!("读取检查结果行失败: {e}")))?);
    }

    Ok(results)
}

/// 判断指定章节是否存在 hard 违规（severity='hard' AND passed=0 AND content_hash=expected_hash）。
///
/// 参数：
/// - conn: SQLite 连接
/// - chapter_id: 章节 ID
/// - expected_hash: 预期的正文 hash
///
/// 返回：true 表示存在 hard 违规，false 表示不存在。
pub fn has_hard_violations(
    conn: &Connection,
    chapter_id: &str,
    expected_hash: &str,
) -> Result<bool, CliError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rule_checks
             WHERE chapter_id = ?1 AND content_hash = ?2 AND severity = 'hard' AND passed = 0",
            params![chapter_id, expected_hash],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询 hard 违规失败: {e}")))?;

    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{chapters, connection::run_migrations};

    fn check(name: &str, passed: bool) -> CheckResultInput {
        CheckResultInput {
            constraint_type: "plot".to_string(),
            constraint_scope: "world".to_string(),
            constraint_name: name.to_string(),
            severity: "hard".to_string(),
            passed,
            description: None,
            location: None,
        }
    }

    #[test]
    fn replacement_rolls_back_when_new_set_insert_fails() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn, "2026-09-29T00:00:00Z").unwrap();
        chapters::insert_chapter(
            &conn,
            "chapter-001",
            "第一章",
            "content/chapters/chapter-001.md",
            "hash-v1",
            10,
            None,
            "2026-09-29T00:00:00Z",
        )
        .unwrap();
        insert_check_results(
            &conn,
            "chapter-001",
            "hash-v1",
            &[check("旧检查", true)],
            "2026-09-29T00:01:00Z",
        )
        .unwrap();

        conn.execute_batch(
            "CREATE TRIGGER fail_hash_v2 BEFORE INSERT ON rule_checks
             WHEN NEW.content_hash = 'hash-v2'
             BEGIN SELECT RAISE(ABORT, 'forced insert failure'); END;",
        )
        .unwrap();

        let result = insert_check_results(
            &conn,
            "chapter-001",
            "hash-v2",
            &[check("新检查", true)],
            "2026-09-29T00:02:00Z",
        );
        assert!(result.is_err());

        let checks = list_check_results(&conn, "chapter-001").unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].constraint_name, "旧检查");
        assert_eq!(checks[0].content_hash, "hash-v1");
    }
}
