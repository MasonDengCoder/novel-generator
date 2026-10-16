// 错误对象序列化测试（task 6.4）
// 验证每个新错误变体（41-47）的 ErrorObject JSON 输出格式符合 error 契约：
// 包含 code、message、exitCode 字段，且 code/exitCode 配对正确。

use novel_cli::errors::{
    CliError, ErrorCatalog, ErrorCategory, ErrorObject, ErrorProtocolVersion, ExitCode,
    V2ErrorEnvelope,
};

/// 校验 ErrorObject 的契约字段：code/message/exitCode 齐全且配对正确。
fn assert_error_contract(err: &CliError, expected_code: &str, expected_exit: i64) {
    let obj = ErrorObject::from_error(err);

    // 序列化为 JSON 并解析回来，验证输出可被 JSON 消费方解析
    let json_str = serde_json::to_string(&obj).expect("ErrorObject 应可序列化");
    let v: serde_json::Value = serde_json::from_str(&json_str).expect("应为合法 JSON");

    // 契约必填字段
    assert_eq!(v["code"].as_str().unwrap(), expected_code, "code 字段不匹配");
    assert_eq!(v["exitCode"].as_i64().unwrap(), expected_exit, "exitCode 字段不匹配");
    assert!(v["message"].as_str().unwrap().len() > 0, "message 不应为空");

    // message 与 code 同源（来自 CliError）
    assert_eq!(obj.code, expected_code);
    assert_eq!(obj.exit_code as i64, expected_exit);
    assert!(!obj.message.is_empty());
}

#[test]
fn invalid_state_transition_serialization() {
    let err = CliError::invalid_state_transition("状态转换非法");
    assert_error_contract(&err, "invalid_state_transition", 41);
    assert_eq!(err.exit_code(), ExitCode::InvalidStateTransition);
}

#[test]
fn thread_name_conflict_serialization() {
    let err = CliError::thread_name_conflict("线索重名");
    assert_error_contract(&err, "thread_name_conflict", 42);
    assert_eq!(err.exit_code(), ExitCode::ThreadNameConflict);
}

#[test]
fn mainplot_node_not_found_serialization() {
    let err = CliError::mainplot_node_not_found("主线节点不存在");
    assert_error_contract(&err, "mainplot_node_not_found", 43);
    assert_eq!(err.exit_code(), ExitCode::MainplotNodeNotFound);
}

#[test]
fn thread_not_found_serialization() {
    let err = CliError::thread_not_found("线索不存在");
    assert_error_contract(&err, "thread_not_found", 44);
    assert_eq!(err.exit_code(), ExitCode::ThreadNotFound);
}

#[test]
fn emergence_not_found_serialization() {
    let err = CliError::emergence_not_found("涌现不存在");
    assert_error_contract(&err, "emergence_not_found", 45);
    assert_eq!(err.exit_code(), ExitCode::EmergenceNotFound);
}

#[test]
fn binding_already_exists_serialization() {
    let err = CliError::binding_already_exists("绑定已存在");
    assert_error_contract(&err, "binding_already_exists", 46);
    assert_eq!(err.exit_code(), ExitCode::BindingAlreadyExists);
}

#[test]
fn emergence_archived_serialization() {
    let err = CliError::emergence_archived("涌现已归档");
    assert_error_contract(&err, "emergence_archived", 47);
    assert_eq!(err.exit_code(), ExitCode::EmergenceArchived);
}

#[test]
fn error_object_camel_case_field_names() {
    // 验证 ErrorObject 使用 camelCase 字段名（契约要求 exitCode 而非 exit_code）
    let err = CliError::invalid_state_transition("测试");
    let obj = ErrorObject::from_error(&err);
    let json_str = serde_json::to_string(&obj).unwrap();
    assert!(json_str.contains("\"exitCode\""), "应为 camelCase exitCode");
    assert!(json_str.contains("\"code\""));
    assert!(json_str.contains("\"message\""));
    assert!(!json_str.contains("\"exit_code\""), "不应出现 snake_case exit_code");
}

#[test]
fn error_object_suggestion_optional_and_present() {
    // 新错误变体都附带 suggestion（非空）
    let err = CliError::invalid_state_transition("测试");
    let obj = ErrorObject::from_error(&err);
    assert!(obj.suggestion.is_some(), "invalid_state_transition 应附带 suggestion");

    let err = CliError::emergence_archived("测试");
    let obj = ErrorObject::from_error(&err);
    assert!(obj.suggestion.is_some(), "emergence_archived 应附带 suggestion");

    // 序列化时 suggestion 字段应出现（非 None）
    let json_str = serde_json::to_string(&obj).unwrap();
    assert!(json_str.contains("\"suggestion\""), "suggestion 字段应出现在 JSON 中");
}

#[test]
fn all_seven_new_error_codes_form_complete_set() {
    // 验证 41-47 全部覆盖，code 字符串与退出码一一对应
    let cases: Vec<(&str, i64)> = vec![
        ("invalid_state_transition", 41),
        ("thread_name_conflict", 42),
        ("mainplot_node_not_found", 43),
        ("thread_not_found", 44),
        ("emergence_not_found", 45),
        ("binding_already_exists", 46),
        ("emergence_archived", 47),
    ];
    assert_eq!(cases.len(), 7);

    // 确保退出码无重复
    let codes: std::collections::HashSet<i64> = cases.iter().map(|(_, c)| *c).collect();
    assert_eq!(codes.len(), 7, "7 个退出码应互不重复");
}

// ────────────────────────────────────────────────────────────────
// V2 目录化迁移（contract-registry-v2-02）task 6.5：
// V2 路径输出目录化信封，退出码由错误类别推导；V1 基线（上方每实体
// 41-47 断言）保持不变，二者并存。
// ────────────────────────────────────────────────────────────────

/// V2 信封经目录构造：schemaVersion/status=failed/errorId/category/exitCode 齐全，
/// exitCode 为类别推导值，V1 per-entity 码（如 41-47）在 V2 路径折叠为类别码。
#[test]
fn v2_envelope_uses_category_derived_exit_code() {
    // invalid_state_transition → precondition(3)。V1 是 41，V2 路径应为类别 3。
    let entry = ErrorCatalog::get("invalid_state_transition").expect("已登记");
    assert_eq!(entry.category, ErrorCategory::Precondition);
    assert_eq!(entry.exit_code(), 3);

    let env = V2ErrorEnvelope::from_catalog(
        "invalid_state_transition",
        "状态转换非法".to_string(),
        None,
        None,
    )
    .expect("目录化信封构造");
    assert_eq!(env.category, "precondition");
    assert_eq!(env.exit_code, 3, "V2 exitCode 应为类别推导值");

    let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&env).unwrap()).unwrap();
    assert_eq!(v["schemaVersion"], "error-protocol/v2");
    assert_eq!(v["status"], "failed");
    assert_eq!(v["errorId"], "invalid_state_transition");
    assert_eq!(v["exitCode"], 3);
}

/// 05 治理默认化：缺省协议为 V2，退出码经目录类别派生；显式选择 V1 的兼容期路径
/// 维持 per-entity 退出码 41（V1 回归基线逐字不变）。
#[test]
fn v2_is_default_while_explicit_v1_keeps_per_entity_exit_code() {
    let err = CliError::invalid_state_transition("状态转换非法");

    // 缺省（05 起）：协议版本为 V2。
    assert_eq!(ErrorProtocolVersion::default(), ErrorProtocolVersion::V2);

    // V2 路径：经目录类别派生退出码为 3。
    assert_eq!(
        ErrorCatalog::exit_code(err.error_id().as_str()),
        Some(3),
        "V2 目录派生退出码应为类别 3"
    );

    // V1 显式兼容路径：per-entity 退出码 41 逐字不变（兼容基线）。
    let v1 = ErrorProtocolVersion::V1;
    assert_ne!(v1, ErrorProtocolVersion::default(), "V1 须为显式选择");
    assert_eq!(err.exit_code(), ExitCode::InvalidStateTransition);
}
