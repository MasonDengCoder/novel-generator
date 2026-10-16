// 集成测试：mainplot_validator（task 12.1）
// 用真实迁移后 schema 测试 validate_target_chapters 引用完整性检查，
// 与 src 内联单元测试（手建简化表）互补。

#[path = "../common/mod.rs"]
mod common;

use novel_cli::validators::mainplot_validator;

#[test]
fn validate_target_chapters_all_exist() {
    let (conn, _target, vid, ch1, ch2) = common::setup_db_with_two_chapters();

    // 全部存在的章节
    assert!(mainplot_validator::validate_target_chapters(&conn, &[ch1, ch2]).is_ok());
}

#[test]
fn validate_target_chapters_empty_ok() {
    let (conn, _target, _vid, _ch1, _ch2) = common::setup_db_with_two_chapters();

    assert!(mainplot_validator::validate_target_chapters(&conn, &[]).is_ok());
}

#[test]
fn validate_target_chapters_not_found_rejected() {
    let (conn, _target, _vid, ch1, _ch2) = common::setup_db_with_two_chapters();

    // 混合存在与不存在的章节
    let result = mainplot_validator::validate_target_chapters(&conn, &[ch1, "ch-999".to_string()]);
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("ch-999"));
}

#[test]
fn validate_target_chapters_single_not_found_rejected() {
    let (conn, _target, _vid, _ch1, _ch2) = common::setup_db_with_two_chapters();

    let result = mainplot_validator::validate_target_chapters(&conn, &["ch-999".to_string()]);
    assert!(result.is_err());
}
