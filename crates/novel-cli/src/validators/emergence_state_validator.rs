// emergence_state_validator 模块：涌现状态机校验。

use crate::domain::emergence::{Decision, EmergenceStatus};
use crate::errors::CliError;

/// 校验状态转换规则：检查从当前状态到目标状态的转换是否合法。
///
/// 合法转换：
/// - pending → rated
/// - rated → decided
/// - decided → executed
/// - executed → archived
pub fn validate_state_transition(
    current_status: EmergenceStatus,
    target_status: EmergenceStatus,
) -> Result<(), CliError> {
    if !current_status.can_transition_to(target_status) {
        return Err(CliError::invalid_state_transition(format!(
            "无法从状态 {} 转换到 {}，必须按照 pending→rated→decided→executed→archived 顺序推进",
            current_status.as_str(),
            target_status.as_str()
        )));
    }
    Ok(())
}

/// 校验决策是否允许执行：仅 accept 决策可以执行。
pub fn validate_decision_allows_execution(decision: Option<Decision>) -> Result<(), CliError> {
    match decision {
        Some(Decision::Accept) => Ok(()),
        Some(Decision::Defer) => Err(CliError::invalid_state_transition(
            "无法执行 defer 决策的涌现，只有 accept 决策可以执行".to_string(),
        )),
        Some(Decision::Reject) => Err(CliError::invalid_state_transition(
            "无法执行 reject 决策的涌现，只有 accept 决策可以执行".to_string(),
        )),
        None => Err(CliError::invalid_state_transition(
            "涌现尚未决策，无法执行".to_string(),
        )),
    }
}

/// 校验评分范围：必须在 0-10 之间。
pub fn validate_rating(rating: i64) -> Result<(), CliError> {
    if !(0..=10).contains(&rating) {
        return Err(CliError::usage(format!(
            "评分无效：{}，必须在 0-10 范围内",
            rating
        )));
    }
    Ok(())
}

/// 校验涌现是否已归档：归档后的涌现不可修改。
pub fn validate_not_archived(status: EmergenceStatus) -> Result<(), CliError> {
    if status == EmergenceStatus::Archived {
        return Err(CliError::emergence_archived(
            "涌现已归档，无法修改".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_state_transition_valid() {
        // 合法转换
        assert!(validate_state_transition(
            EmergenceStatus::Pending,
            EmergenceStatus::Rated
        )
        .is_ok());

        assert!(validate_state_transition(
            EmergenceStatus::Rated,
            EmergenceStatus::Decided
        )
        .is_ok());

        assert!(validate_state_transition(
            EmergenceStatus::Decided,
            EmergenceStatus::Executed
        )
        .is_ok());

        assert!(validate_state_transition(
            EmergenceStatus::Executed,
            EmergenceStatus::Archived
        )
        .is_ok());
    }

    #[test]
    fn test_validate_state_transition_skip_step() {
        // 跳步转换应失败
        assert!(validate_state_transition(
            EmergenceStatus::Pending,
            EmergenceStatus::Decided
        )
        .is_err());

        assert!(validate_state_transition(
            EmergenceStatus::Rated,
            EmergenceStatus::Executed
        )
        .is_err());

        assert!(validate_state_transition(
            EmergenceStatus::Pending,
            EmergenceStatus::Archived
        )
        .is_err());
    }

    #[test]
    fn test_validate_state_transition_backward() {
        // 后退转换应失败
        assert!(validate_state_transition(
            EmergenceStatus::Rated,
            EmergenceStatus::Pending
        )
        .is_err());

        assert!(validate_state_transition(
            EmergenceStatus::Decided,
            EmergenceStatus::Rated
        )
        .is_err());

        assert!(validate_state_transition(
            EmergenceStatus::Archived,
            EmergenceStatus::Executed
        )
        .is_err());
    }

    #[test]
    fn test_validate_state_transition_same_state() {
        // 转换到相同状态应失败
        assert!(validate_state_transition(
            EmergenceStatus::Pending,
            EmergenceStatus::Pending
        )
        .is_err());

        assert!(validate_state_transition(
            EmergenceStatus::Rated,
            EmergenceStatus::Rated
        )
        .is_err());
    }

    #[test]
    fn test_validate_decision_allows_execution_accept() {
        assert!(validate_decision_allows_execution(Some(Decision::Accept)).is_ok());
    }

    #[test]
    fn test_validate_decision_allows_execution_defer() {
        let result = validate_decision_allows_execution(Some(Decision::Defer));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("defer"));
    }

    #[test]
    fn test_validate_decision_allows_execution_reject() {
        let result = validate_decision_allows_execution(Some(Decision::Reject));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("reject"));
    }

    #[test]
    fn test_validate_decision_allows_execution_none() {
        let result = validate_decision_allows_execution(None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("尚未决策"));
    }

    #[test]
    fn test_validate_rating_valid() {
        assert!(validate_rating(0).is_ok());
        assert!(validate_rating(5).is_ok());
        assert!(validate_rating(10).is_ok());
    }

    #[test]
    fn test_validate_rating_invalid() {
        assert!(validate_rating(-1).is_err());
        assert!(validate_rating(11).is_err());
        assert!(validate_rating(100).is_err());
    }

    #[test]
    fn test_validate_not_archived_ok() {
        assert!(validate_not_archived(EmergenceStatus::Pending).is_ok());
        assert!(validate_not_archived(EmergenceStatus::Rated).is_ok());
        assert!(validate_not_archived(EmergenceStatus::Decided).is_ok());
        assert!(validate_not_archived(EmergenceStatus::Executed).is_ok());
    }

    #[test]
    fn test_validate_not_archived_fail() {
        let result = validate_not_archived(EmergenceStatus::Archived);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("已归档"));
    }
}
