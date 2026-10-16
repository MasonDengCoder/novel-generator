// 集成测试：thread_validator（task 12.2）
// 用真实迁移后 schema 测试线索名称唯一性与绑定类型验证。

#[path = "../common/mod.rs"]
mod common;

use novel_cli::services::thread_service;
use novel_cli::validators::thread_validator;

#[test]
fn validate_binding_type_valid() {
    assert!(thread_validator::validate_binding_type("mention").is_ok());
    assert!(thread_validator::validate_binding_type("develop").is_ok());
    assert!(thread_validator::validate_binding_type("resolve").is_ok());
}

#[test]
fn validate_binding_type_invalid_rejected() {
    assert!(thread_validator::validate_binding_type("invalid").is_err());
    assert!(thread_validator::validate_binding_type("").is_err());
    assert!(thread_validator::validate_binding_type("MENTION").is_err());
}

#[test]
fn validate_thread_name_uniqueness_ok() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    // 卷内尚无线索，名称可用
    assert!(thread_validator::validate_thread_name_uniqueness(&conn, vid, "新线索").is_ok());
}

#[test]
fn validate_thread_name_uniqueness_conflict_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    // 先创建一条线索
    thread_service::create_thread(&target, vid, "已有线索", None).unwrap();
    // 同卷再用同名应被拒
    let result = thread_validator::validate_thread_name_uniqueness(&conn, vid, "已有线索");
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("已有线索"));
}

#[test]
fn validate_thread_name_uniqueness_different_volume_ok() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    thread_service::create_thread(&target, vid, "跨卷线索", None).unwrap();
    // 在另一个不存在的卷号（9999）中，同名可用
    assert!(thread_validator::validate_thread_name_uniqueness(&conn, 9999, "跨卷线索").is_ok());
}
