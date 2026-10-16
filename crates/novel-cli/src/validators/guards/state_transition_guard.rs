// state_transition_guard 模块：章节状态推进的硬阻断门禁（s04-state-machine-and-verdicts）。
//
// 职责（CLAUDE.md §6、§10）：判定状态转换路径合法性（白名单）+ 对前向 review/approved
// 跑全部门禁（正文已写入、无 hard 违规、裁决 achieved=1 且绑定当前 hash、stale 检测）。
// 回退转换跳过门禁。门禁收集全部失败项一次性报告（复用 s05 SealCheckResult 模式），
// 不短路。外部修改检测由 service 层独立完成（退出码 21，优先于门禁），不混入 GuardCheckResult。

use crate::errors::CliError;
use crate::repositories::{rule_check_repository, verdict_repository};
use crate::storage::chapters::ChapterRow;
use crate::validators::chapter_validator;
use rusqlite::Connection;

/// 章节状态值常量。
pub const STATUS_OUTLINE: &str = "outline";
pub const STATUS_DRAFTING: &str = "drafting";
pub const STATUS_REVIEW: &str = "review";
pub const STATUS_APPROVED: &str = "approved";

/// 主线落地裁决类型常量。
pub const VERDICT_TYPE_MAINPLOT_LANDING: &str = "mainplot-landing";

/// 门禁检查结果（复用 s05 SealCheckResult 形态）。
#[derive(Debug, Clone)]
pub struct GuardCheckResult {
    pub check_type: String,
    pub passed: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// 判定状态转换是否合法并收集门禁结果。
///
/// 返回 `Vec<GuardCheckResult>`：
/// - 转换路径非法（含 approved 终态不可转换）：返回单项路径检查失败。
/// - 前向 review/approved：跑全部门禁，收集全部失败项（不短路）。
/// - 回退 review→drafting/outline：跳过门禁，返回空 Vec（service 层另行检测外部修改）。
///
/// 参数：
/// - conn: SQLite 连接。
/// - chapter: 当前章节元数据（含 status 与 content_hash）。
/// - target: 目标状态。
pub fn can_transition(
    conn: &Connection,
    chapter: &ChapterRow,
    target: &str,
) -> Result<Vec<GuardCheckResult>, CliError> {
    let from = chapter.status.as_str();

    // 1. 转换路径合法性白名单（D1）。
    if !is_valid_path(from, target) {
        return Ok(vec![GuardCheckResult {
            check_type: "转换路径".to_string(),
            passed: false,
            message: format!(
                "非法状态转换：{} → {}。当前状态 {} 允许的目标：{}。",
                from,
                target,
                from,
                allowed_targets(from).join(", ")
            ),
            details: vec![],
        }]);
    }

    // 2. 回退转换跳过门禁（D6、state-transition-guards spec）。
    if is_backward_transition(from, target) {
        return Ok(Vec::new());
    }

    // 3. outline → drafting 是前向转换但无需门禁（仅进入 review/approved 才跑门禁）。
    //    外部修改检测由 service 层独立完成。
    if !is_guarded_transition(from, target) {
        return Ok(Vec::new());
    }

    // 4. 前向 review/approved：跑全部门禁。
    let current_hash = &chapter.content_hash;
    let mut results = Vec::new();

    // 门禁 1：正文已写入（content_hash 非空字符串 hash）。
    results.push(check_content_written(current_hash));

    // 门禁 2：无 hard 违规（绑定当前 hash）。
    results.push(check_no_hard_violations(conn, &chapter.id, current_hash)?);

    // 门禁 3：裁决 achieved=1 且绑定当前 hash。
    results.push(check_verdict_achieved(conn, &chapter.id, current_hash)?);

    // 门禁 4：裁决与检查 stale 检测（存在绑定旧 hash 的裁决/检查）。
    results.push(check_not_stale(conn, &chapter.id, current_hash)?);

    Ok(results)
}

/// 判定转换路径是否合法（白名单）。
pub fn is_valid_path(from: &str, target: &str) -> bool {
    matches!(
        (from, target),
        ("outline", "drafting")
            | ("drafting", "review")
            | ("review", "approved")
            | ("review", "drafting")
            | ("review", "outline")
    )
}

/// 判定是否为回退转换。
pub fn is_backward_transition(from: &str, target: &str) -> bool {
    matches!(
        (from, target),
        ("review", "drafting") | ("review", "outline")
    )
}

/// 判定是否为需要门禁的前向转换（drafting→review 或 review→approved）。
///
/// `outline→drafting` 是前向转换但无需门禁。
pub fn is_guarded_transition(from: &str, target: &str) -> bool {
    matches!(
        (from, target),
        ("drafting", "review") | ("review", "approved")
    )
}

/// 返回当前状态允许的目标状态列表（用于错误消息）。
pub fn allowed_targets(from: &str) -> Vec<&'static str> {
    match from {
        "outline" => vec!["drafting"],
        "drafting" => vec!["review"],
        "review" => vec!["approved", "drafting", "outline"],
        "approved" => vec![],
        _ => vec![],
    }
}

/// 门禁：正文已写入（content_hash 非空字符串 hash）。
///
/// 空正文章节的 content_hash 等于空内容的 SHA-256；正文写入后 hash 改变。
fn check_content_written(current_hash: &str) -> GuardCheckResult {
    let empty_hash = chapter_validator::calculate_content_hash("");
    let passed = !current_hash.is_empty() && current_hash != empty_hash;
    GuardCheckResult {
        check_type: "正文已写入".to_string(),
        passed,
        message: if passed {
            "正文已写入".to_string()
        } else {
            "正文未写入（content_hash 为空内容 hash）".to_string()
        },
        details: vec![],
    }
}

/// 门禁：无 hard 违规（绑定当前 hash）。
fn check_no_hard_violations(
    conn: &Connection,
    chapter_id: &str,
    current_hash: &str,
) -> Result<GuardCheckResult, CliError> {
    let has_hard = rule_check_repository::has_hard_violations(conn, chapter_id, current_hash)?;
    Ok(GuardCheckResult {
        check_type: "无 hard 违规".to_string(),
        passed: !has_hard,
        message: if has_hard {
            "存在 hard 违规（绑定当前正文 hash 且 passed=0 的 hard 检查结果）".to_string()
        } else {
            "无 hard 违规".to_string()
        },
        details: vec![],
    })
}

/// 门禁：裁决 achieved=1 且绑定当前 hash。
fn check_verdict_achieved(
    conn: &Connection,
    chapter_id: &str,
    current_hash: &str,
) -> Result<GuardCheckResult, CliError> {
    let achieved = verdict_repository::has_achieved_mainplot_landing(conn, chapter_id, current_hash)?;
    Ok(GuardCheckResult {
        check_type: "裁决已达成".to_string(),
        passed: achieved,
        message: if achieved {
            "主线落地裁决已设置且 achieved=1".to_string()
        } else {
            "主线落地裁决未设置或 achieved=0 或绑定 hash 不匹配（stale）".to_string()
        },
        details: vec![],
    })
}

/// 门禁：裁决与检查 stale 检测。
///
/// 若该章节存在绑定非当前 hash 的裁决或检查结果，视为 stale。
fn check_not_stale(
    conn: &Connection,
    chapter_id: &str,
    current_hash: &str,
) -> Result<GuardCheckResult, CliError> {
    // 检查是否存在绑定旧 hash 的 rule_checks。
    let stale_checks = count_stale_checks(conn, chapter_id, current_hash)?;
    // 检查是否存在绑定旧 hash 的 verdicts。
    let stale_verdicts = count_stale_verdicts(conn, chapter_id, current_hash)?;

    let passed = stale_checks == 0 && stale_verdicts == 0;
    let mut details = Vec::new();
    if stale_checks > 0 {
        details.push(format!("存在 {} 条绑定旧 hash 的检查结果（stale）", stale_checks));
    }
    if stale_verdicts > 0 {
        details.push(format!("存在 {} 条绑定旧 hash 的裁决（stale）", stale_verdicts));
    }

    Ok(GuardCheckResult {
        check_type: "裁决与检查未 stale".to_string(),
        passed,
        message: if passed {
            "裁决与检查均绑定当前正文 hash".to_string()
        } else {
            "裁决或检查已 stale（正文已更新但旧结果仍绑定旧 hash）".to_string()
        },
        details,
    })
}

/// 统计该章节绑定非当前 hash 的 rule_checks 数量。
fn count_stale_checks(
    conn: &Connection,
    chapter_id: &str,
    current_hash: &str,
) -> Result<i64, CliError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM rule_checks
             WHERE chapter_id = ?1 AND content_hash != ?2",
            rusqlite::params![chapter_id, current_hash],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询 stale 检查结果失败: {e}")))?;
    Ok(count)
}

/// 统计该章节绑定非当前 hash 的 verdicts 数量。
fn count_stale_verdicts(
    conn: &Connection,
    chapter_id: &str,
    current_hash: &str,
) -> Result<i64, CliError> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM verdicts
             WHERE chapter_id = ?1 AND content_hash != ?2",
            rusqlite::params![chapter_id, current_hash],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("查询 stale 裁决失败: {e}")))?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::rule_check_repository::{CheckResultInput, insert_check_results};
    use crate::repositories::verdict_repository::upsert_verdict;
    use crate::storage::chapters::{insert_chapter, update_chapter_metadata};
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

    fn write_passing_check(conn: &Connection, chapter_id: &str, hash: &str) {
        insert_check_results(
            conn,
            chapter_id,
            hash,
            &[CheckResultInput {
                constraint_type: "plot".to_string(),
                constraint_scope: "world".to_string(),
                constraint_name: "时间线".to_string(),
                severity: "hard".to_string(),
                passed: true,
                description: None,
                location: None,
            }],
            "2026-07-01T01:00:00Z",
        )
        .unwrap();
    }

    #[test]
    fn test_is_valid_path_forward() {
        assert!(is_valid_path("outline", "drafting"));
        assert!(is_valid_path("drafting", "review"));
        assert!(is_valid_path("review", "approved"));
    }

    #[test]
    fn test_is_valid_path_backward() {
        assert!(is_valid_path("review", "drafting"));
        assert!(is_valid_path("review", "outline"));
    }

    #[test]
    fn test_is_guarded_transition() {
        // 仅 drafting→review 与 review→approved 需要门禁
        assert!(is_guarded_transition("drafting", "review"));
        assert!(is_guarded_transition("review", "approved"));
        // outline→drafting 前向但无需门禁
        assert!(!is_guarded_transition("outline", "drafting"));
        // 回退无需门禁
        assert!(!is_guarded_transition("review", "drafting"));
    }

    #[test]
    fn test_is_valid_path_illegal_skip() {
        assert!(!is_valid_path("outline", "approved"));
        assert!(!is_valid_path("outline", "review"));
        assert!(!is_valid_path("drafting", "approved"));
    }

    #[test]
    fn test_is_valid_path_approved_terminal() {
        // approved 不可转换为任何状态
        assert!(!is_valid_path("approved", "drafting"));
        assert!(!is_valid_path("approved", "outline"));
        assert!(!is_valid_path("approved", "review"));
        assert!(!is_valid_path("approved", "approved"));
    }

    #[test]
    fn test_can_transition_illegal_path() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "approved").unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(results[0].message.contains("非法状态转换"));
    }

    #[test]
    fn test_can_transition_approved_terminal_blocked() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        // 直接把状态设为 approved
        crate::storage::chapters::update_status(&conn, "chapter-001", "approved", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "drafting").unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn test_can_transition_backward_skips_guards() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        crate::storage::chapters::update_status(&conn, "chapter-001", "review", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        // 回退跳过门禁：即使无裁决/检查也通过
        let results = can_transition(&conn, &row, "drafting").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_can_transition_review_all_pass() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        // 写入通过的 hard 检查
        write_passing_check(&conn, "chapter-001", "hash-001");
        // 写入已达成裁决
        upsert_verdict(&conn, "chapter-001", VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", true, "hash-001", "2026-07-01T01:00:00Z").unwrap();

        crate::storage::chapters::update_status(&conn, "chapter-001", "drafting", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "review").unwrap();
        assert!(results.iter().all(|r| r.passed), "全部门禁应通过: {:?}", results);
    }

    #[test]
    fn test_can_transition_review_content_not_written() {
        let (conn, _dir) = setup();
        // 空内容章节（content_hash = empty hash）
        seed_chapter(&conn, "chapter-001", &chapter_validator::calculate_content_hash(""));
        crate::storage::chapters::update_status(&conn, "chapter-001", "drafting", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "review").unwrap();
        let content_check = results.iter().find(|r| r.check_type == "正文已写入").unwrap();
        assert!(!content_check.passed);
    }

    #[test]
    fn test_can_transition_review_hard_violation_blocked() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        // 写入未通过的 hard 检查
        insert_check_results(
            &conn,
            "chapter-001",
            "hash-001",
            &[CheckResultInput {
                constraint_type: "plot".to_string(),
                constraint_scope: "world".to_string(),
                constraint_name: "时间线".to_string(),
                severity: "hard".to_string(),
                passed: false,
                description: None,
                location: None,
            }],
            "2026-07-01T01:00:00Z",
        )
        .unwrap();
        upsert_verdict(&conn, "chapter-001", VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", true, "hash-001", "2026-07-01T01:00:00Z").unwrap();

        crate::storage::chapters::update_status(&conn, "chapter-001", "drafting", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "review").unwrap();
        let hard_check = results.iter().find(|r| r.check_type == "无 hard 违规").unwrap();
        assert!(!hard_check.passed);
    }

    #[test]
    fn test_can_transition_review_verdict_not_achieved_blocked() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        write_passing_check(&conn, "chapter-001", "hash-001");
        // 裁决 achieved=0
        upsert_verdict(&conn, "chapter-001", VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", false, "hash-001", "2026-07-01T01:00:00Z").unwrap();

        crate::storage::chapters::update_status(&conn, "chapter-001", "drafting", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "review").unwrap();
        let verdict_check = results.iter().find(|r| r.check_type == "裁决已达成").unwrap();
        assert!(!verdict_check.passed);
    }

    #[test]
    fn test_can_transition_review_stale_blocked() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-old");
        // 检查与裁决绑定旧 hash
        write_passing_check(&conn, "chapter-001", "hash-old");
        upsert_verdict(&conn, "chapter-001", VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", true, "hash-old", "2026-07-01T01:00:00Z").unwrap();
        // 正文更新为新 hash
        update_chapter_metadata(&conn, "chapter-001", "hash-new", 200, "2026-07-01T02:00:00Z").unwrap();

        crate::storage::chapters::update_status(&conn, "chapter-001", "drafting", "2026-07-01T02:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "review").unwrap();
        let stale_check = results.iter().find(|r| r.check_type == "裁决与检查未 stale").unwrap();
        assert!(!stale_check.passed);
    }

    #[test]
    fn test_can_transition_review_multiple_failures_reported() {
        let (conn, _dir) = setup();
        seed_chapter(&conn, "chapter-001", "hash-001");
        // hard 违规 + 裁决未达成：两项同时失败
        insert_check_results(
            &conn,
            "chapter-001",
            "hash-001",
            &[CheckResultInput {
                constraint_type: "plot".to_string(),
                constraint_scope: "world".to_string(),
                constraint_name: "时间线".to_string(),
                severity: "hard".to_string(),
                passed: false,
                description: None,
                location: None,
            }],
            "2026-07-01T01:00:00Z",
        )
        .unwrap();
        // 不写裁决 → 裁决未达成

        crate::storage::chapters::update_status(&conn, "chapter-001", "drafting", "2026-07-01T01:00:00Z").unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, "chapter-001").unwrap();

        let results = can_transition(&conn, &row, "review").unwrap();
        let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();
        // 应同时报告 hard 违规与裁决未达成（不短路）
        assert!(failed.iter().any(|r| r.check_type == "无 hard 违规"));
        assert!(failed.iter().any(|r| r.check_type == "裁决已达成"));
    }
}
