// chapter_state_service 模块：章节状态推进用例（s04-state-machine-and-verdicts）。
//
// 职责（CLAUDE.md §6、§7、§10）：编排写锁/事务/外部修改检测/门禁/状态更新，
// 组装错误。门禁未全过则 precondition_blocked（聚合全部失败项 details）。
// hard 阻断非零退出码，不静默降级。

use crate::errors::CliError;
use crate::storage::chapters;
use crate::storage::connection;
use crate::storage::file_ops::ProjectWriteLock;
use crate::validators::chapter_validator;
use crate::validators::guards::state_transition_guard::{self, GuardCheckResult};
use std::path::Path;

/// 状态推进结果（用于 renderer）。
#[derive(Debug)]
pub struct StatusTransitioned {
    pub chapter_id: String,
    pub from: String,
    pub to: String,
    pub updated_at: String,
}

/// 推进章节状态至目标状态。
///
/// 调用链（design D6）：
///   写锁 → open_existing_at → 读取 ChapterRow → 外部修改检测（退出码 21）
///   → state_transition_guard::can_transition → 门禁未全过则 precondition_blocked
///   → 事务 update_status → commit
pub fn transition_status(
    target_dir: &Path,
    chapter_id: &str,
    target: &str,
) -> Result<StatusTransitioned, CliError> {
    let novel_dir = target_dir.join(".novel");

    // 1. 获取写锁（防止与并发 chapter update 交错）。
    let _lock = ProjectWriteLock::acquire(&novel_dir)?;

    // 2. 打开数据库（自动检查 world_not_initialized）。
    let mut conn = connection::open_existing_at(target_dir)?;

    // 2.5 s07 归档冻结门禁：章节所属卷 archived 时拒绝推进状态。
    crate::validators::guards::volume_freeze_guard::ensure_chapter_volume_not_archived(
        &conn, chapter_id,
    )?;

    // 3. 读取章节元数据（chapter_not_found 检测）。
    let chapter = chapters::get_chapter(&conn, chapter_id)?;

    // 4. 外部修改检测（退出码 21，优先于门禁，独立检测不混入 GuardCheckResult）。
    let content_path = novel_dir.join(&chapter.content_path);
    let current_hash = match std::fs::read_to_string(&content_path) {
        Ok(content) => chapter_validator::calculate_content_hash(&content),
        Err(_) => {
            return Err(CliError::content_modified_externally(format!(
                "章节 {} 的正文文件无法读取，无法验证完整性：{}",
                chapter_id,
                content_path.display()
            )));
        }
    };
    if current_hash != chapter.content_hash {
        return Err(CliError::content_modified_externally(format!(
            "章节 {} 的正文已被外部修改，请运行 `novel chapter reimport {}` 重新导入。",
            chapter_id, chapter_id
        )));
    }

    // 5. 状态机门禁（路径合法性 + 前向门禁；回退跳过门禁）。
    let results = state_transition_guard::can_transition(&conn, &chapter, target)?;

    // 6. 门禁未全过 → precondition_blocked（聚合全部失败项 details）。
    let failed: Vec<&GuardCheckResult> = results.iter().filter(|r| !r.passed).collect();
    if !failed.is_empty() {
        let mut lines: Vec<String> = failed
            .iter()
            .flat_map(|r| {
                let mut l = vec![format!("【{}】{}", r.check_type, r.message)];
                l.extend(r.details.iter().map(|d| format!("  - {}", d)));
                l
            })
            .collect();
        let _ = &mut lines;
        return Err(CliError::precondition_blocked(format!(
            "状态推进前置条件未满足（{} → {}）：\n{}",
            chapter.status,
            target,
            lines.join("\n")
        )));
    }

    // 7. 事务内更新状态。
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let from = chapter.status.clone();
    let tx = conn
        .transaction()
        .map_err(|e| CliError::storage(format!("开启事务失败：{}", e)))?;

    chapters::update_status(&tx, chapter_id, target, &timestamp)?;

    tx.commit()
        .map_err(|e| CliError::storage(format!("提交事务失败：{}", e)))?;

    Ok(StatusTransitioned {
        chapter_id: chapter_id.to_string(),
        from,
        to: target.to_string(),
        updated_at: timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::chapter_repository;
    use crate::repositories::rule_check_repository::{CheckResultInput, insert_check_results};
    use crate::repositories::verdict_repository::upsert_verdict;
    use crate::storage::connection::run_migrations;
    use crate::validators::guards::state_transition_guard::VERDICT_TYPE_MAINPLOT_LANDING;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().to_path_buf();
        let novel_dir = project_dir.join(".novel");
        fs::create_dir_all(&novel_dir).unwrap();

        let db_path = novel_dir.join("novel.db");
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn, "2026-07-01T00:00:00Z").unwrap();
        }

        (dir, project_dir)
    }

    fn create_and_write_chapter(project_dir: &std::path::PathBuf, title: &str, content: &str) -> String {
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let chapter_id = chapter_repository::create_chapter(&conn, &novel_dir, title, None).unwrap();
        chapter_repository::update_chapter_content(&conn, &novel_dir, &chapter_id, content).unwrap();
        chapter_id
    }

    /// 构造满足 review 全部门禁的章节（正文已写入 + 通过的 hard 检查 + 已达成裁决）。
    fn prepare_passing_chapter(project_dir: &std::path::PathBuf) -> String {
        let chapter_id = create_and_write_chapter(project_dir, "测试", "正文内容");
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();

        // 写入通过的 hard 检查
        insert_check_results(
            &conn,
            &chapter_id,
            &row.content_hash,
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

        // 写入已达成裁决
        upsert_verdict(&conn, &chapter_id, VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", true, &row.content_hash, "2026-07-01T01:00:00Z").unwrap();

        // 推进到 drafting
        chapters::update_status(&conn, &chapter_id, "drafting", "2026-07-01T01:00:00Z").unwrap();

        chapter_id
    }

    #[test]
    fn test_transition_outline_to_drafting_ok() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");

        let result = transition_status(&project_dir, &chapter_id, "drafting").unwrap();
        assert_eq!(result.from, "outline");
        assert_eq!(result.to, "drafting");
    }

    #[test]
    fn test_transition_drafting_to_review_ok() {
        let (_dir, project_dir) = setup();
        let chapter_id = prepare_passing_chapter(&project_dir);

        let result = transition_status(&project_dir, &chapter_id, "review").unwrap();
        assert_eq!(result.from, "drafting");
        assert_eq!(result.to, "review");
    }

    #[test]
    fn test_transition_review_to_approved_ok() {
        let (_dir, project_dir) = setup();
        let chapter_id = prepare_passing_chapter(&project_dir);

        // 先推进到 review
        transition_status(&project_dir, &chapter_id, "review").unwrap();

        // 再推进到 approved
        let result = transition_status(&project_dir, &chapter_id, "approved").unwrap();
        assert_eq!(result.from, "review");
        assert_eq!(result.to, "approved");
    }

    #[test]
    fn test_transition_illegal_path_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");

        let result = transition_status(&project_dir, &chapter_id, "approved");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_transition_approved_terminal_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = prepare_passing_chapter(&project_dir);
        transition_status(&project_dir, &chapter_id, "review").unwrap();
        transition_status(&project_dir, &chapter_id, "approved").unwrap();

        // approved 不可转换
        let result = transition_status(&project_dir, &chapter_id, "drafting");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_transition_hard_violation_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();

        // 写入未通过的 hard 检查
        insert_check_results(
            &conn,
            &chapter_id,
            &row.content_hash,
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
        upsert_verdict(&conn, &chapter_id, VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", true, &row.content_hash, "2026-07-01T01:00:00Z").unwrap();
        chapters::update_status(&conn, &chapter_id, "drafting", "2026-07-01T01:00:00Z").unwrap();

        let result = transition_status(&project_dir, &chapter_id, "review");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_transition_verdict_not_achieved_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();

        // 通过的检查但裁决 achieved=0
        insert_check_results(
            &conn,
            &chapter_id,
            &row.content_hash,
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
        upsert_verdict(&conn, &chapter_id, VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", false, &row.content_hash, "2026-07-01T01:00:00Z").unwrap();
        chapters::update_status(&conn, &chapter_id, "drafting", "2026-07-01T01:00:00Z").unwrap();

        let result = transition_status(&project_dir, &chapter_id, "review");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_transition_stale_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "原始正文");
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();

        // 检查与裁决绑定旧 hash
        insert_check_results(
            &conn,
            &chapter_id,
            &row.content_hash,
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
        upsert_verdict(&conn, &chapter_id, VERDICT_TYPE_MAINPLOT_LANDING, "mp-1-001", true, &row.content_hash, "2026-07-01T01:00:00Z").unwrap();

        // 更新正文（hash 改变），旧检查/裁决变 stale
        chapter_repository::update_chapter_content(&conn, &novel_dir, &chapter_id, "更新后的正文").unwrap();
        chapters::update_status(&conn, &chapter_id, "drafting", "2026-07-01T02:00:00Z").unwrap();

        let result = transition_status(&project_dir, &chapter_id, "review");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_transition_external_modification_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "原始正文");

        // 外部修改正文
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let content_path = novel_dir.join(&row.content_path);
        fs::write(&content_path, "外部修改").unwrap();

        let result = transition_status(&project_dir, &chapter_id, "drafting");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::ContentModifiedExternally { .. }));
    }

    #[test]
    fn test_transition_backward_ok() {
        let (_dir, project_dir) = setup();
        let chapter_id = prepare_passing_chapter(&project_dir);
        transition_status(&project_dir, &chapter_id, "review").unwrap();

        // 回退到 drafting（跳过门禁）
        let result = transition_status(&project_dir, &chapter_id, "drafting").unwrap();
        assert_eq!(result.from, "review");
        assert_eq!(result.to, "drafting");
    }

    #[test]
    fn test_transition_chapter_not_found() {
        let (_dir, project_dir) = setup();
        let result = transition_status(&project_dir, "chapter-999", "drafting");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::ChapterNotFound { .. }));
    }

    #[test]
    fn test_transition_multiple_failures_reported() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();

        // hard 违规 + 无裁决：两项同时失败
        insert_check_results(
            &conn,
            &chapter_id,
            &row.content_hash,
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
        chapters::update_status(&conn, &chapter_id, "drafting", "2026-07-01T01:00:00Z").unwrap();

        let result = transition_status(&project_dir, &chapter_id, "review");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.message();
        // 应同时报告 hard 违规与裁决未达成
        assert!(msg.contains("hard"));
        assert!(msg.contains("裁决"));
    }
}
