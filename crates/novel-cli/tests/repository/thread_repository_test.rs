// 单元测试：thread_repository（task 11.2）
// 测试 create_thread、bind_chapter、list_threads、query_by_chapter、update_status

use novel_cli::domain::{BindingType, ThreadStatus};
use novel_cli::repositories::thread_repository;
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
fn create_thread_inserts_record() {
    let (conn, vid, _) = setup_db();

    let thread = thread_repository::create_thread(&conn, vid, "线索A", Some("描述")).unwrap();
    assert_eq!(thread.volume_id, vid);
    assert_eq!(thread.name, "线索A");
    assert_eq!(thread.description, Some("描述".to_string()));
    assert_eq!(thread.status, ThreadStatus::Active);
    assert!(thread.id.starts_with(&format!("th-{}-", vid)));
}

#[test]
fn create_thread_assigns_increasing_sequence() {
    let (conn, vid, _) = setup_db();

    let t1 = thread_repository::create_thread(&conn, vid, "线索1", None).unwrap();
    let t2 = thread_repository::create_thread(&conn, vid, "线索2", None).unwrap();

    assert_ne!(t1.id, t2.id);
}

#[test]
fn create_thread_duplicate_name_returns_conflict() {
    let (conn, vid, _) = setup_db();

    thread_repository::create_thread(&conn, vid, "重名线索", None).unwrap();
    let result = thread_repository::create_thread(&conn, vid, "重名线索", None);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ThreadNameConflict);
}

#[test]
fn list_threads_returns_all_threads_in_volume() {
    let (conn, vid, _) = setup_db();

    thread_repository::create_thread(&conn, vid, "线索A", None).unwrap();
    thread_repository::create_thread(&conn, vid, "线索B", None).unwrap();

    let threads = thread_repository::list_threads(&conn, vid).unwrap();
    assert_eq!(threads.len(), 2);
}

#[test]
fn list_threads_empty_volume_returns_empty_vec() {
    let (conn, vid, _) = setup_db();

    let threads = thread_repository::list_threads(&conn, vid).unwrap();
    assert!(threads.is_empty());
}

#[test]
fn get_thread_returns_correct_thread() {
    let (conn, vid, _) = setup_db();

    let created = thread_repository::create_thread(&conn, vid, "目标线索", None).unwrap();
    let fetched = thread_repository::get_thread(&conn, &created.id).unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "目标线索");
}

#[test]
fn get_thread_not_found_returns_error() {
    let (conn, _, _) = setup_db();

    let result = thread_repository::get_thread(&conn, "no-such-thread");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ThreadNotFound);
}

#[test]
fn update_status_changes_thread_state() {
    let (conn, vid, _) = setup_db();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();
    assert_eq!(thread.status, ThreadStatus::Active);

    thread_repository::update_status(&conn, &thread.id, ThreadStatus::Paused).unwrap();
    let fetched = thread_repository::get_thread(&conn, &thread.id).unwrap();
    assert_eq!(fetched.status, ThreadStatus::Paused);

    thread_repository::update_status(&conn, &thread.id, ThreadStatus::Resolved).unwrap();
    let fetched = thread_repository::get_thread(&conn, &thread.id).unwrap();
    assert_eq!(fetched.status, ThreadStatus::Resolved);
}

#[test]
fn update_status_not_found_returns_error() {
    let (conn, _, _) = setup_db();

    let result = thread_repository::update_status(&conn, "no-such-thread", ThreadStatus::Resolved);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ThreadNotFound);
}

#[test]
fn bind_chapter_creates_binding() {
    let (conn, vid, chapter_id) = setup_db();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();
    thread_repository::bind_chapter(&conn, &thread.id, &chapter_id, BindingType::Mention, None).unwrap();

    let bindings = thread_repository::query_by_chapter(&conn, &chapter_id).unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].thread_id, thread.id);
    assert_eq!(bindings[0].chapter_id, chapter_id);
    assert_eq!(bindings[0].binding_type, BindingType::Mention);
}

#[test]
fn bind_chapter_with_note_and_develop_type() {
    let (conn, vid, chapter_id) = setup_db();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();
    thread_repository::bind_chapter(&conn, &thread.id, &chapter_id, BindingType::Develop, Some("备注")).unwrap();

    let bindings = thread_repository::query_by_chapter(&conn, &chapter_id).unwrap();
    assert_eq!(bindings[0].binding_type, BindingType::Develop);
    assert_eq!(bindings[0].note, Some("备注".to_string()));
}

#[test]
fn bind_chapter_duplicate_returns_conflict() {
    let (conn, vid, chapter_id) = setup_db();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();
    thread_repository::bind_chapter(&conn, &thread.id, &chapter_id, BindingType::Mention, None).unwrap();

    // 重复绑定同一线索到同一章节
    let result = thread_repository::bind_chapter(&conn, &thread.id, &chapter_id, BindingType::Develop, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::BindingAlreadyExists);
}

#[test]
fn query_by_chapter_returns_all_bindings() {
    let (conn, vid, chapter_id) = setup_db();

    let t1 = thread_repository::create_thread(&conn, vid, "线索1", None).unwrap();
    let t2 = thread_repository::create_thread(&conn, vid, "线索2", None).unwrap();

    thread_repository::bind_chapter(&conn, &t1.id, &chapter_id, BindingType::Mention, None).unwrap();
    thread_repository::bind_chapter(&conn, &t2.id, &chapter_id, BindingType::Resolve, None).unwrap();

    let bindings = thread_repository::query_by_chapter(&conn, &chapter_id).unwrap();
    assert_eq!(bindings.len(), 2);
}

#[test]
fn query_by_chapter_no_bindings_returns_empty() {
    let (conn, _, _) = setup_db();

    let bindings = thread_repository::query_by_chapter(&conn, "ch-empty").unwrap();
    assert!(bindings.is_empty());
}

#[test]
fn get_thread_name_returns_name() {
    let (conn, vid, _) = setup_db();

    let thread = thread_repository::create_thread(&conn, vid, "命名线索", None).unwrap();
    let name = thread_repository::get_thread_name(&conn, &thread.id).unwrap();
    assert_eq!(name, "命名线索");
}
