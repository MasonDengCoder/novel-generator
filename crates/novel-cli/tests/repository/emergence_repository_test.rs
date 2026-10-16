// 单元测试：emergence_repository（task 11.3）
// 测试涌现完整生命周期的 repository 操作：create、rate、decide、execute、archive、list、get

use novel_cli::domain::{Decision, EmergenceStatus};
use novel_cli::repositories::emergence_repository;
use novel_cli::storage::connection::{open, run_migrations};
use novel_cli::errors::ExitCode;
use rusqlite::{params, Connection};
use tempfile::tempdir;

/// 建立已迁移的临时数据库，插入一个测试卷与一个章节，返回 (conn, volume_id, chapter_id)。
fn setup_db() -> (Connection, i64, String) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    std::mem::forget(dir);
    let conn = open(&db_path).unwrap();
    run_migrations(&conn, "2026-01-01T00:00:00Z").unwrap();

    conn.execute(
        "INSERT INTO volumes (name, volume_order, created_at, updated_at) VALUES ('测试卷', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    let volume_id: i64 = conn.last_insert_rowid();

    let chapter_id = "ch-001".to_string();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, volume_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![chapter_id, "第一章", "chapters/ch-001.md", "hash001", volume_id],
    )
    .unwrap();

    (conn, volume_id, chapter_id)
}

#[test]
fn create_emergence_inserts_record() {
    let (conn, vid, _) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现标题", "涌现描述").unwrap();
    assert_eq!(em.volume_id, vid);
    assert_eq!(em.title, "涌现标题");
    assert_eq!(em.description, "涌现描述");
    assert_eq!(em.status, EmergenceStatus::Pending);
    assert!(em.chapter_id.is_none());
    assert!(em.rating.is_none());
    assert!(em.decision.is_none());
    assert!(em.id.starts_with(&format!("em-{}-", vid)));
}

#[test]
fn create_emergence_with_chapter() {
    let (conn, vid, chapter_id) = setup_db();

    let em = emergence_repository::create(&conn, vid, Some(&chapter_id), "涌现", "描述").unwrap();
    assert_eq!(em.chapter_id, Some(chapter_id));
}

#[test]
fn create_emergence_assigns_increasing_sequence() {
    let (conn, vid, _) = setup_db();

    let e1 = emergence_repository::create(&conn, vid, None, "涌现1", "描述").unwrap();
    let e2 = emergence_repository::create(&conn, vid, None, "涌现2", "描述").unwrap();

    assert_ne!(e1.id, e2.id);
}

#[test]
fn get_emergence_returns_correct_record() {
    let (conn, vid, _) = setup_db();

    let created = emergence_repository::create(&conn, vid, None, "目标涌现", "详情").unwrap();
    let fetched = emergence_repository::get(&conn, &created.id).unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "目标涌现");
}

#[test]
fn get_emergence_not_found_returns_error() {
    let (conn, _, _) = setup_db();

    let result = emergence_repository::get(&conn, "no-such-emergence");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn rate_transitions_to_rated() {
    let (conn, vid, _) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();
    assert_eq!(em.status, EmergenceStatus::Pending);

    emergence_repository::rate(&conn, &em.id, 8, "高价值").unwrap();
    let fetched = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(fetched.status, EmergenceStatus::Rated);
    assert_eq!(fetched.rating, Some(8));
    assert_eq!(fetched.rating_reason, Some("高价值".to_string()));
    assert!(fetched.rated_at.is_some());
}

#[test]
fn rate_not_found_returns_error() {
    let (conn, _, _) = setup_db();

    let result = emergence_repository::rate(&conn, "no-such-emergence", 5, "理由");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn decide_transitions_to_decided() {
    let (conn, vid, _) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();
    emergence_repository::rate(&conn, &em.id, 7, "理由").unwrap();
    emergence_repository::decide(&conn, &em.id, Decision::Accept, "采纳").unwrap();

    let fetched = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(fetched.status, EmergenceStatus::Decided);
    assert_eq!(fetched.decision, Some(Decision::Accept));
    assert_eq!(fetched.decision_reason, Some("采纳".to_string()));
    assert!(fetched.decided_at.is_some());
}

#[test]
fn decide_with_defer() {
    let (conn, vid, _) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();
    emergence_repository::rate(&conn, &em.id, 5, "理由").unwrap();
    emergence_repository::decide(&conn, &em.id, Decision::Defer, "暂缓").unwrap();

    let fetched = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(fetched.decision, Some(Decision::Defer));
}

#[test]
fn decide_with_reject() {
    let (conn, vid, _) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();
    emergence_repository::rate(&conn, &em.id, 2, "理由").unwrap();
    emergence_repository::decide(&conn, &em.id, Decision::Reject, "拒绝").unwrap();

    let fetched = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(fetched.decision, Some(Decision::Reject));
}

#[test]
fn execute_transitions_to_executed() {
    let (conn, vid, chapter_id) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();
    emergence_repository::rate(&conn, &em.id, 9, "理由").unwrap();
    emergence_repository::decide(&conn, &em.id, Decision::Accept, "采纳").unwrap();
    emergence_repository::execute(&conn, &em.id, &chapter_id, Some("执行备注")).unwrap();

    let fetched = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(fetched.status, EmergenceStatus::Executed);
    assert_eq!(fetched.execution_target, Some(chapter_id));
    assert_eq!(fetched.execution_note, Some("执行备注".to_string()));
    assert!(fetched.executed_at.is_some());
}

#[test]
fn execute_not_found_returns_error() {
    let (conn, _, _) = setup_db();

    let result = emergence_repository::execute(&conn, "no-such-emergence", "ch-001", None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn archive_transitions_to_archived() {
    let (conn, vid, chapter_id) = setup_db();

    let em = emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();
    emergence_repository::rate(&conn, &em.id, 9, "理由").unwrap();
    emergence_repository::decide(&conn, &em.id, Decision::Accept, "采纳").unwrap();
    emergence_repository::execute(&conn, &em.id, &chapter_id, None).unwrap();
    emergence_repository::archive(&conn, &em.id).unwrap();

    let fetched = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(fetched.status, EmergenceStatus::Archived);
    assert!(fetched.archived_at.is_some());
}

#[test]
fn archive_not_found_returns_error() {
    let (conn, _, _) = setup_db();

    let result = emergence_repository::archive(&conn, "no-such-emergence");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn list_returns_all_emergences_in_volume() {
    let (conn, vid, _) = setup_db();

    emergence_repository::create(&conn, vid, None, "涌现1", "描述").unwrap();
    emergence_repository::create(&conn, vid, None, "涌现2", "描述").unwrap();
    emergence_repository::create(&conn, vid, None, "涌现3", "描述").unwrap();

    let list = emergence_repository::list(&conn, vid, None).unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn list_empty_volume_returns_empty() {
    let (conn, vid, _) = setup_db();

    let list = emergence_repository::list(&conn, vid, None).unwrap();
    assert!(list.is_empty());
}

#[test]
fn list_filter_by_status() {
    let (conn, vid, _) = setup_db();

    // pending
    emergence_repository::create(&conn, vid, None, "待评1", "描述").unwrap();
    emergence_repository::create(&conn, vid, None, "待评2", "描述").unwrap();
    // rated
    let r = emergence_repository::create(&conn, vid, None, "已评", "描述").unwrap();
    emergence_repository::rate(&conn, &r.id, 5, "理由").unwrap();

    let pending = emergence_repository::list(&conn, vid, Some(EmergenceStatus::Pending)).unwrap();
    assert_eq!(pending.len(), 2);

    let rated = emergence_repository::list(&conn, vid, Some(EmergenceStatus::Rated)).unwrap();
    assert_eq!(rated.len(), 1);
    assert_eq!(rated[0].title, "已评");
}

#[test]
fn full_lifecycle_all_fields_correct() {
    let (conn, vid, chapter_id) = setup_db();

    // pending
    let em = emergence_repository::create(&conn, vid, Some(&chapter_id), "完整涌现", "完整描述").unwrap();
    assert_eq!(em.status, EmergenceStatus::Pending);
    assert!(em.rating.is_none() && em.decision.is_none() && em.execution_target.is_none() && em.archived_at.is_none());

    // rated
    emergence_repository::rate(&conn, &em.id, 10, "满分理由").unwrap();
    let em = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(em.status, EmergenceStatus::Rated);

    // decided
    emergence_repository::decide(&conn, &em.id, Decision::Accept, "决定采纳").unwrap();
    let em = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(em.status, EmergenceStatus::Decided);

    // executed
    emergence_repository::execute(&conn, &em.id, &chapter_id, Some("执行完毕")).unwrap();
    let em = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(em.status, EmergenceStatus::Executed);

    // archived
    emergence_repository::archive(&conn, &em.id).unwrap();
    let em = emergence_repository::get(&conn, &em.id).unwrap();
    assert_eq!(em.status, EmergenceStatus::Archived);
    assert!(em.archived_at.is_some());
}
