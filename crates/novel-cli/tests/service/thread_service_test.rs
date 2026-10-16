// 单元测试：thread_service（task 13.2）
// 测试 service 层业务逻辑与错误处理：create_thread、bind_chapter、list_threads、query_by_chapter、update_status

// 单元测试：thread_service（task 13.2）
// 测试 service 层业务逻辑与错误处理：create_thread、bind_chapter、list_threads、query_by_chapter、update_status

#[path = "../common/mod.rs"]
mod common;

use novel_cli::errors::ExitCode;
use novel_cli::services::thread_service;
use rusqlite::params;

#[test]
fn create_thread_success() {
    let (conn, target, vid) = common::setup_db();

    let thread = thread_service::create_thread(&target, vid, "线索A", Some("描述")).unwrap();
    assert_eq!(thread.volume_id, vid);
    assert_eq!(thread.name, "线索A");
}

#[test]
fn create_thread_volume_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = thread_service::create_thread(&target, 9999, "线索", None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::NotFound);
}

#[test]
fn create_thread_duplicate_name_rejected() {
    let (conn, target, vid) = common::setup_db();

    thread_service::create_thread(&target, vid, "重名", None).unwrap();
    let result = thread_service::create_thread(&target, vid, "重名", None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ThreadNameConflict);
}

#[test]
fn create_thread_same_name_different_volume_ok() {
    let (conn, target, vid1) = common::setup_db();
    // 插入第二个卷
    conn.execute(
        "INSERT INTO volumes (name, volume_order, created_at, updated_at) VALUES ('卷二', 2, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        params![],
    )
    .unwrap();
    let vid2: i64 = conn.last_insert_rowid();

    thread_service::create_thread(&target, vid1, "共享名", None).unwrap();
    // 在卷二中可以用相同名称
    let result = thread_service::create_thread(&target, vid2, "共享名", None);
    assert!(result.is_ok());
}

#[test]
fn bind_chapter_success() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let thread = thread_service::create_thread(&target, vid, "线索", None).unwrap();
    thread_service::bind_chapter(&target, &thread.id, &ch1, Some("mention"), None).unwrap();

    let bindings = thread_service::query_by_chapter(&target, &ch1).unwrap();
    assert_eq!(bindings.len(), 1);
}

#[test]
fn bind_chapter_invalid_type_rejected() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let thread = thread_service::create_thread(&target, vid, "线索", None).unwrap();
    let result = thread_service::bind_chapter(&target, &thread.id, &ch1, Some("invalid"), None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::Usage);
}

#[test]
fn bind_chapter_thread_not_found() {
    let (conn, target, _vid, ch1) = common::setup_db_with_chapter();

    let result = thread_service::bind_chapter(&target, "no-such-thread", &ch1, None, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ThreadNotFound);
}

#[test]
fn bind_chapter_chapter_not_found() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let thread = thread_service::create_thread(&target, vid, "线索", None).unwrap();
    let result = thread_service::bind_chapter(&target, &thread.id, "ch-999", None, None);
    assert!(result.is_err());
    // 章节不存在走 ChapterNotFound（退出码 20）
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ChapterNotFound);
}

#[test]
fn bind_chapter_duplicate_rejected() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let thread = thread_service::create_thread(&target, vid, "线索", None).unwrap();
    thread_service::bind_chapter(&target, &thread.id, &ch1, Some("mention"), None).unwrap();

    let result = thread_service::bind_chapter(&target, &thread.id, &ch1, Some("develop"), None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::BindingAlreadyExists);
}

#[test]
fn list_threads_returns_all() {
    let (conn, target, vid) = common::setup_db();

    thread_service::create_thread(&target, vid, "线索1", None).unwrap();
    thread_service::create_thread(&target, vid, "线索2", None).unwrap();

    let threads = thread_service::list_threads(&target, vid).unwrap();
    assert_eq!(threads.len(), 2);
}

#[test]
fn list_threads_volume_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = thread_service::list_threads(&target, 9999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::NotFound);
}

#[test]
fn query_by_chapter_returns_bindings() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let t1 = thread_service::create_thread(&target, vid, "线索1", None).unwrap();
    let t2 = thread_service::create_thread(&target, vid, "线索2", None).unwrap();

    thread_service::bind_chapter(&target, &t1.id, &ch1, Some("mention"), None).unwrap();
    thread_service::bind_chapter(&target, &t2.id, &ch1, Some("resolve"), None).unwrap();

    let bindings = thread_service::query_by_chapter(&target, &ch1).unwrap();
    assert_eq!(bindings.len(), 2);
}

#[test]
fn query_by_chapter_not_found() {
    let (conn, target, _vid, _ch) = common::setup_db_with_chapter();

    let result = thread_service::query_by_chapter(&target, "ch-999");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ChapterNotFound);
}

#[test]
fn update_status_success() {
    let (conn, target, vid) = common::setup_db();

    let thread = thread_service::create_thread(&target, vid, "线索", None).unwrap();
    thread_service::update_status(&target, &thread.id, "paused").unwrap();

    let threads = thread_service::list_threads(&target, vid).unwrap();
    assert_eq!(threads[0].status.as_str(), "paused");
}

#[test]
fn update_status_invalid_value_rejected() {
    let (conn, target, vid) = common::setup_db();

    let thread = thread_service::create_thread(&target, vid, "线索", None).unwrap();
    let result = thread_service::update_status(&target, &thread.id, "invalid");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::Usage);
}

#[test]
fn update_status_thread_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = thread_service::update_status(&target, "no-such-thread", "paused");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ThreadNotFound);
}
