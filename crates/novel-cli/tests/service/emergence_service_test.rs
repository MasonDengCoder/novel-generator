// 单元测试：emergence_service（task 13.3）
// 测试涌现状态机完整流程与边界条件

// 单元测试：emergence_service（task 13.3）
// 测试涌现状态机完整流程与边界条件

#[path = "../common/mod.rs"]
mod common;

use novel_cli::domain::EmergenceStatus;
use novel_cli::errors::ExitCode;
use novel_cli::services::emergence_service;

#[test]
fn create_emergence_success() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    assert_eq!(em.volume_id, vid);
    assert_eq!(em.title, "涌现");
    assert_eq!(em.status, EmergenceStatus::Pending);
}

#[test]
fn create_emergence_with_chapter() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", Some(&ch1)).unwrap();
    assert_eq!(em.chapter_id, Some(ch1));
}

#[test]
fn create_emergence_volume_not_found() {
    let (conn, target, _vid, _) = common::setup_db_with_chapter();

    let result = emergence_service::create(&target, 9999, "涌现", "描述", None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::NotFound);
}

#[test]
fn create_emergence_chapter_not_found() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let result = emergence_service::create(&target, vid, "涌现", "描述", Some("ch-999"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ChapterNotFound);
}

#[test]
fn rate_success() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    let rated = emergence_service::rate(&target, &em.id, 7, "理由").unwrap();
    assert_eq!(rated.status, EmergenceStatus::Rated);
    assert_eq!(rated.rating, Some(7));
}

#[test]
fn rate_invalid_range_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    // rating = 11 越界
    let result = emergence_service::rate(&target, &em.id, 11, "理由");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::Usage);

    // rating = -1 越界
    let result = emergence_service::rate(&target, &em.id, -1, "理由");
    assert!(result.is_err());
}

#[test]
fn rate_archived_rejected() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 9, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    emergence_service::execute(&target, &em.id, &ch1, None).unwrap();
    emergence_service::archive(&target, &em.id).unwrap();

    // 已归档，再 rate 应被拒
    let result = emergence_service::rate(&target, &em.id, 5, "理由");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceArchived);
}

#[test]
fn decide_success() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 8, "理由").unwrap();
    let decided = emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    assert_eq!(decided.status, EmergenceStatus::Decided);
}

#[test]
fn decide_invalid_value_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 8, "理由").unwrap();
    let result = emergence_service::decide(&target, &em.id, "invalid", "理由");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::Usage);
}

#[test]
fn decide_skip_rate_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    // 未 rate 直接 decide：pending → decided 非法
    let result = emergence_service::decide(&target, &em.id, "accept", "理由");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn execute_accept_success() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 9, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    let executed = emergence_service::execute(&target, &em.id, &ch1, Some("备注")).unwrap();
    assert_eq!(executed.status, EmergenceStatus::Executed);
    assert_eq!(executed.execution_target, Some(ch1));
}

#[test]
fn execute_defer_rejected() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 5, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "defer", "暂缓").unwrap();
    let result = emergence_service::execute(&target, &em.id, &ch1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn execute_reject_rejected() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 2, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "reject", "拒绝").unwrap();
    let result = emergence_service::execute(&target, &em.id, &ch1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn execute_target_chapter_not_found() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 9, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    let result = emergence_service::execute(&target, &em.id, "ch-999", None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::ChapterNotFound);
}

#[test]
fn execute_skip_decide_rejected() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 9, "理由").unwrap();
    // 未 decide 直接 execute：rated → executed 非法
    let result = emergence_service::execute(&target, &em.id, &ch1, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn archive_success() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 9, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    emergence_service::execute(&target, &em.id, &ch1, None).unwrap();
    let archived = emergence_service::archive(&target, &em.id).unwrap();
    assert_eq!(archived.status, EmergenceStatus::Archived);
}

#[test]
fn archive_skip_execute_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    emergence_service::rate(&target, &em.id, 9, "理由").unwrap();
    emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    // 未 execute 直接 archive：decided → archived 非法
    let result = emergence_service::archive(&target, &em.id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn archive_not_found() {
    let (conn, target, _vid, _) = common::setup_db_with_chapter();

    let result = emergence_service::archive(&target, "no-such-emergence");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn list_returns_all() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    emergence_service::create(&target, vid, "涌现1", "描述", None).unwrap();
    emergence_service::create(&target, vid, "涌现2", "描述", None).unwrap();

    let list = emergence_service::list(&target, vid, None).unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn list_filter_by_status() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    emergence_service::create(&target, vid, "待评1", "描述", None).unwrap();
    emergence_service::create(&target, vid, "待评2", "描述", None).unwrap();
    let r = emergence_service::create(&target, vid, "已评", "描述", None).unwrap();
    emergence_service::rate(&target, &r.id, 5, "理由").unwrap();

    let pending = emergence_service::list(&target, vid, Some("pending")).unwrap();
    assert_eq!(pending.len(), 2);

    let rated = emergence_service::list(&target, vid, Some("rated")).unwrap();
    assert_eq!(rated.len(), 1);
    assert_eq!(rated[0].title, "已评");
}

#[test]
fn list_invalid_status_filter_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let result = emergence_service::list(&target, vid, Some("invalid"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::Usage);
}

#[test]
fn list_volume_not_found() {
    let (conn, target, _vid, _) = common::setup_db_with_chapter();

    let result = emergence_service::list(&target, 9999, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::NotFound);
}

#[test]
fn get_not_found() {
    let (conn, target, _vid, _) = common::setup_db_with_chapter();

    let result = emergence_service::get(&target, "no-such-emergence");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn full_lifecycle_accept_executed_archived() {
    let (conn, target, vid, ch1) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "完整涌现", "描述", Some(&ch1)).unwrap();
    assert_eq!(em.status, EmergenceStatus::Pending);

    let em = emergence_service::rate(&target, &em.id, 10, "满分").unwrap();
    assert_eq!(em.status, EmergenceStatus::Rated);

    let em = emergence_service::decide(&target, &em.id, "accept", "采纳").unwrap();
    assert_eq!(em.status, EmergenceStatus::Decided);

    let em = emergence_service::execute(&target, &em.id, &ch1, Some("完成")).unwrap();
    assert_eq!(em.status, EmergenceStatus::Executed);

    let em = emergence_service::archive(&target, &em.id).unwrap();
    assert_eq!(em.status, EmergenceStatus::Archived);
}
