// 集成测试：emergence_state_validator（task 12.3）
// 用真实涌现生命周期测试所有非法状态转换场景（跳步、后退、重复）。

#[path = "../common/mod.rs"]
mod common;

use novel_cli::domain::{Decision, EmergenceStatus};
use novel_cli::errors::ExitCode;
use novel_cli::services::emergence_service;
use novel_cli::validators::emergence_state_validator;

// ---- 状态转换合法性 ----

#[test]
fn valid_forward_transitions() {
    assert!(emergence_state_validator::validate_state_transition(
        EmergenceStatus::Pending, EmergenceStatus::Rated).is_ok());
    assert!(emergence_state_validator::validate_state_transition(
        EmergenceStatus::Rated, EmergenceStatus::Decided).is_ok());
    assert!(emergence_state_validator::validate_state_transition(
        EmergenceStatus::Decided, EmergenceStatus::Executed).is_ok());
    assert!(emergence_state_validator::validate_state_transition(
        EmergenceStatus::Executed, EmergenceStatus::Archived).is_ok());
}

// ---- 跳步 ----

#[test]
fn skip_step_pending_to_decided_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Pending, EmergenceStatus::Decided);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn skip_step_rated_to_executed_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Rated, EmergenceStatus::Executed);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn skip_step_pending_to_archived_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Pending, EmergenceStatus::Archived);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

// ---- 后退 ----

#[test]
fn backward_rated_to_pending_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Rated, EmergenceStatus::Pending);
    assert!(result.is_err());
}

#[test]
fn backward_decided_to_rated_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Decided, EmergenceStatus::Rated);
    assert!(result.is_err());
}

#[test]
fn backward_archived_to_executed_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Archived, EmergenceStatus::Executed);
    assert!(result.is_err());
}

// ---- 重复（转换到相同状态）----

#[test]
fn same_state_pending_to_pending_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Pending, EmergenceStatus::Pending);
    assert!(result.is_err());
}

#[test]
fn same_state_rated_to_rated_rejected() {
    let result = emergence_state_validator::validate_state_transition(
        EmergenceStatus::Rated, EmergenceStatus::Rated);
    assert!(result.is_err());
}

// ---- 决策允许执行 ----

#[test]
fn decision_accept_allows_execution() {
    assert!(emergence_state_validator::validate_decision_allows_execution(Some(Decision::Accept)).is_ok());
}

#[test]
fn decision_defer_rejects_execution() {
    let result = emergence_state_validator::validate_decision_allows_execution(Some(Decision::Defer));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn decision_reject_rejects_execution() {
    let result = emergence_state_validator::validate_decision_allows_execution(Some(Decision::Reject));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn decision_none_rejects_execution() {
    let result = emergence_state_validator::validate_decision_allows_execution(None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}

// ---- 评分范围 ----

#[test]
fn rating_boundary_valid() {
    assert!(emergence_state_validator::validate_rating(0).is_ok());
    assert!(emergence_state_validator::validate_rating(10).is_ok());
}

#[test]
fn rating_out_of_range_rejected() {
    assert!(emergence_state_validator::validate_rating(-1).is_err());
    assert!(emergence_state_validator::validate_rating(11).is_err());
}

// ---- 归档保护 ----

#[test]
fn archived_emergence_blocks_modification() {
    assert!(emergence_state_validator::validate_not_archived(EmergenceStatus::Archived).is_err());
    assert_eq!(
        emergence_state_validator::validate_not_archived(EmergenceStatus::Archived)
            .unwrap_err().exit_code(),
        ExitCode::EmergenceArchived
    );
}

#[test]
fn non_archived_allows_modification() {
    assert!(emergence_state_validator::validate_not_archived(EmergenceStatus::Pending).is_ok());
    assert!(emergence_state_validator::validate_not_archived(EmergenceStatus::Executed).is_ok());
}

// ---- 真实生命周期中状态机被 service 层正确执行 ----

#[test]
fn service_blocks_skip_step_decide_before_rate() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let em = emergence_service::create(&target, vid, "涌现", "描述", None).unwrap();
    // 未 rate 直接 decide
    let result = emergence_service::decide(&target, &em.id, "accept", "理由");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::InvalidStateTransition);
}
