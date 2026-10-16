// errors 层：统一错误类型、结构化错误对象与退出码映射。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §12）：每类领域错误有稳定错误码并映射到明确退出码；
// `--json` 模式返回结构化错误对象（code/message/exitCode，可选 suggestion/details）；
// hard 阻断一律非零退出码，不静默降级。
//
// 退出码类别（contracts/cli/exit-codes.md）：0 成功 / 2 用法 / 3 前置条件阻断 /
// 4 资源冲突 / 5 外部修改检测 / 70 内部存储错误。
//
// V2 错误协议地基（contract-registry-v2-01-error-protocol-foundation）：`catalog` 子模块
// 定义 V2 错误类别/类型化错误目录（ErrorCatalog）/版本化 JSON 信封（V2ErrorEnvelope），
// 与本节既有 V1 `CliError`/`ErrorObject`/`ExitCode`/`ErrorCode` 隔离。本切片只建立类型地基，
// 不改动既有 V1 映射；命令接线与迁移归 02。

use serde::Serialize;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 ErrorObject/BudgetDetail
// 成为 json-schemas 结构契约源。partial_packet/details 为 serde_json::Value 字段，schemars
// 派生为任意 JSON（已知简化，design Risk 记录）。
use schemars::JsonSchema;
use thiserror::Error;

/// V2 错误协议地基（contract-registry-v2-01）。见 [`crate::errors::catalog`]。
pub mod catalog;

// V2 错误协议类型 re-export（crategisters/模块对齐清单 §12 错误协议方向）：
// - ErrorCategory：七类固定错误类别，退出码是类别的纯函数。
// - ErrorCatalog：类型化错误目录（错误 ID ↔ 类别 ↔ 退出码 ↔ 恢复/详情契约）。
// - ErrorEntry/DetailSpec：目录条目与结构化详情契约。
// - V2ErrorEnvelope：版本化 JSON 错误信封。
// - ErrorProtocolVersion：V1/V2 版本选择契约（缺省 V2；V1 为弃用兼容路径，须显式选择）。
pub use catalog::{
    DetailSpec, ErrorCatalog, ErrorCategory, ErrorEntry, ErrorProtocolVersion, V2ErrorEnvelope,
};

/// 进程退出码类别。
///
/// 退出码数值的唯一类型化权威源（contract-governance-02-error-enum-gen）：
/// 变体显式判别式（`= 0/2/3/…/70`）即退出码数值，spec-gen 遍历 `ExitCode::iter()` 取
/// `*variant as i32` 生成 `contracts/cli/exit-codes.md` §A 退出码数值列。退出码无字符串契约
/// （`exit-codes.md` §A 用数值 + 中文类别，非 snake_case 串），故本 enum 不加 strum serialize 注解。
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum ExitCode {
    Success = 0,
    Usage = 2,
    PreconditionBlocked = 3,
    ResourceConflict = 4,
    ExternalModification = 5,
    NotFound = 6,
    FileNotFound = 11,
    ChapterNotFound = 20,
    ContentModifiedExternally = 21,
    InvalidStateTransition = 41,
    ThreadNameConflict = 42,
    MainplotNodeNotFound = 43,
    ThreadNotFound = 44,
    EmergenceNotFound = 45,
    BindingAlreadyExists = 46,
    EmergenceArchived = 47,
    Storage = 70,
}

impl From<ExitCode> for i32 {
    fn from(c: ExitCode) -> Self {
        c as i32
    }
}

/// 稳定结构化错误码字符串。
///
/// 错误码字符串的唯一类型化权威源（contract-governance-02-error-enum-gen）：
/// `#[strum(serialize = "...")]` 注解的 snake_case 字面量即错误码字符串契约，逐字对应原手写
/// `as_str()` 返回值（契约字符串零漂移）。spec-gen 遍历 `ErrorCode::iter()` 经 `IntoStaticStr`
/// 取序列化名生成 `contracts/cli/error-codes.md` §A 错误码字符串列。
/// `as_str()` 退为 strum 生成（`IntoStaticStr` 的 `From<ErrorCode> for &'static str`）薄封装。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::EnumIter, strum_macros::IntoStaticStr)]
pub enum ErrorCode {
    #[strum(serialize = "usage")]
    Usage,
    #[strum(serialize = "precondition_blocked")]
    PreconditionBlocked,
    #[strum(serialize = "resource_conflict")]
    ResourceConflict,
    #[strum(serialize = "external_modification")]
    ExternalModification,
    #[strum(serialize = "not_found")]
    NotFound,
    #[strum(serialize = "storage")]
    Storage,
    #[strum(serialize = "setting_not_found")]
    SettingNotFound,
    #[strum(serialize = "constraint_not_found")]
    ConstraintNotFound,
    #[strum(serialize = "constraint_already_exists")]
    ConstraintAlreadyExists,
    #[strum(serialize = "world_not_initialized")]
    WorldNotInitialized,
    #[strum(serialize = "chapter_not_found")]
    ChapterNotFound,
    #[strum(serialize = "content_modified_externally")]
    ContentModifiedExternally,
    #[strum(serialize = "file_not_found")]
    FileNotFound,
    #[strum(serialize = "invalid_content")]
    InvalidContent,
    #[strum(serialize = "content_hash_mismatch")]
    ContentHashMismatch,
    #[strum(serialize = "stale_rule_checks")]
    StaleRuleChecks,
    #[strum(serialize = "invalid_input")]
    InvalidInput,
    #[strum(serialize = "volume_not_found")]
    VolumeNotFound,
    #[strum(serialize = "volume_already_sealed")]
    VolumeAlreadySealed,
    #[strum(serialize = "chapters_not_approved")]
    ChaptersNotApproved,
    #[strum(serialize = "emergences_not_handled")]
    EmergencesNotHandled,
    #[strum(serialize = "goals_not_achieved")]
    GoalsNotAchieved,
    #[strum(serialize = "volume_not_sealed")]
    VolumeNotSealed,
    #[strum(serialize = "volume_already_archived")]
    VolumeAlreadyArchived,
    #[strum(serialize = "invalid_state_transition")]
    InvalidStateTransition,
    #[strum(serialize = "thread_name_conflict")]
    ThreadNameConflict,
    #[strum(serialize = "mainplot_node_not_found")]
    MainplotNodeNotFound,
    #[strum(serialize = "thread_not_found")]
    ThreadNotFound,
    #[strum(serialize = "emergence_not_found")]
    EmergenceNotFound,
    #[strum(serialize = "binding_already_exists")]
    BindingAlreadyExists,
    #[strum(serialize = "emergence_archived")]
    EmergenceArchived,
    #[strum(serialize = "context_budget_exceeded")]
    ContextBudgetExceeded,
    /// 状态冲突（退出码 5）：状态不一致或版本冲突。
    /// 对齐 `error-codes.md` §A 与 `spec-gen/ssot.rs`（s03-rule-checking-engine 固化），
    /// 修复 s03 遗留的 enum 缺失不一致；不新增退出码，映射既有 `ExitCode::ExternalModification`=5。
    #[strum(serialize = "state_conflict")]
    StateConflict,
}

impl ErrorCode {
    /// 错误码字符串（snake_case 契约串）。
    ///
    /// 委托 strum `IntoStaticStr` 生成的 `From<ErrorCode> for &'static str`；
    /// 输出与替换前手写 `match` 逐字一致（契约字符串零漂移）。保持 `&'static str` 返回，
    /// 调用方（`ErrorObject::from_error`）零改动。
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    /// 经 `ErrorCatalog` 解析该错误 ID 绑定的 V2 错误类别（唯一绑定）。
    ///
    /// V2 目录化迁移（contract-registry-v2-02）：错误 ID 与类别的唯一关系以 `ErrorCatalog`
    /// 为权威源，此处经 `ErrorCatalog::get(self.as_str())` 解析，而非在调用点手写归属。
    /// 目录未登记该 ID 时返回 `None`（契约不一致，测试锁定全量覆盖）。
    pub fn category(&self) -> Option<ErrorCategory> {
        ErrorCatalog::get(self.as_str()).map(|e| e.category)
    }
}

/// 统一领域错误。
#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Usage { message: String, suggestion: Option<String> },

    #[error("{message}")]
    PreconditionBlocked { message: String, suggestion: Option<String>, code: ErrorCode },

    #[error("{message}")]
    ResourceConflict { message: String, suggestion: Option<String>, code: ErrorCode },

    #[error("{message}")]
    ExternalModification { message: String, suggestion: Option<String> },

    #[error("{message}")]
    NotFound { message: String, suggestion: Option<String>, code: ErrorCode },

    #[error("{message}")]
    FileNotFound { message: String, suggestion: Option<String> },

    #[error("{message}")]
    ChapterNotFound { message: String, suggestion: Option<String> },

    #[error("{message}")]
    ContentModifiedExternally { message: String, suggestion: Option<String> },

    #[error("{message}")]
    InvalidContent { message: String, suggestion: Option<String> },

    #[error("{message}")]
    ContentHashMismatch { message: String, suggestion: Option<String> },

    #[error("{message}")]
    StaleRuleChecks { message: String, suggestion: Option<String> },

    #[error("{message}")]
    InvalidInput { message: String, suggestion: Option<String> },

    #[error("{message}")]
    InvalidStateTransition { message: String, suggestion: Option<String> },

    #[error("{message}")]
    ThreadNameConflict { message: String, suggestion: Option<String> },

    #[error("{message}")]
    MainplotNodeNotFound { message: String, suggestion: Option<String> },

    #[error("{message}")]
    ThreadNotFound { message: String, suggestion: Option<String> },

    #[error("{message}")]
    EmergenceNotFound { message: String, suggestion: Option<String> },

    #[error("{message}")]
    BindingAlreadyExists { message: String, suggestion: Option<String> },

    #[error("{message}")]
    EmergenceArchived { message: String, suggestion: Option<String> },

    /// 上下文预算超限（chapter-context-packet）：P0/P1_min 装填超预算，可恢复前置阻断。
    /// 退出码 3（前置条件阻断类别），错误码 `context_budget_exceeded`（细分码）。
    /// 携带部分 packet（已装填的 fragments + audit.failures）作为 `partial_packet`
    /// （序列化为 serde_json::Value，避免 errors 层依赖 domain 结构）。
    #[error("{message}")]
    ContextBudgetExceeded {
        message: String,
        suggestion: Option<String>,
        partial_packet: serde_json::Value,
        budget_limit: i64,
        budget_used: i64,
        failures: Vec<crate::domain::BudgetFailure>,
    },

    #[error("{message}")]
    Storage { message: String, suggestion: Option<String> },

    /// `novel verify integrity` hard 违规（design D1）：完整性验证存在失败项。
    /// 独立承载 `exit_code` 与 `code`，因 verify 的退出码↔错误码组合不来自同一默认变体
    /// （悬空引用需 exit 70 + code `not_found`，而 `NotFound` 默认映射 exit 6）。
    /// `details` 承载完整 `IntegrityReport` 序列化（三分项失败项），结构同
    /// `verify-integrity.schema.json`，让 Spec 经错误对象获取失败项，不依赖裸退出码。
    #[error("{message}")]
    VerifyIntegrity {
        message: String,
        suggestion: Option<String>,
        exit_code: ExitCode,
        code: ErrorCode,
        details: serde_json::Value,
    },
}

impl CliError {
    pub fn usage(msg: impl Into<String>) -> Self {
        Self::Usage { message: msg.into(), suggestion: None }
    }

    pub fn precondition_blocked(msg: impl Into<String>) -> Self {
        Self::PreconditionBlocked {
            message: msg.into(),
            suggestion: None,
            code: ErrorCode::PreconditionBlocked,
        }
    }

    pub fn world_not_initialized(msg: impl Into<String>) -> Self {
        Self::PreconditionBlocked {
            message: msg.into(),
            suggestion: None,
            code: ErrorCode::WorldNotInitialized,
        }
    }

    pub fn resource_conflict(msg: impl Into<String>) -> Self {
        Self::ResourceConflict {
            message: msg.into(),
            suggestion: Some(
                "若需在新目录初始化，请指定一个尚无 .novel/ 的 --path。".to_string(),
            ),
            code: ErrorCode::ResourceConflict,
        }
    }

    pub fn constraint_already_exists(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::ResourceConflict {
            message: msg.into(),
            suggestion,
            code: ErrorCode::ConstraintAlreadyExists,
        }
    }

    pub fn not_found(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::NotFound {
            message: msg.into(),
            suggestion,
            code: ErrorCode::NotFound,
        }
    }

    pub fn setting_not_found(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::NotFound {
            message: msg.into(),
            suggestion,
            code: ErrorCode::SettingNotFound,
        }
    }

    pub fn constraint_not_found(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::NotFound {
            message: msg.into(),
            suggestion,
            code: ErrorCode::ConstraintNotFound,
        }
    }

    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage { message: msg.into(), suggestion: None }
    }

    pub fn chapter_not_found(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::ChapterNotFound {
            message: msg.into(),
            suggestion,
        }
    }

    pub fn content_modified_externally(msg: impl Into<String>) -> Self {
        Self::ContentModifiedExternally {
            message: msg.into(),
            suggestion: Some(
                "正文已被外部修改。请运行 `novel chapter reimport <id>` 重新导入，或使用 `novel chapter update <id> --content <file>` 覆盖。".to_string(),
            ),
        }
    }

    pub fn file_not_found(msg: impl Into<String>) -> Self {
        Self::FileNotFound {
            message: msg.into(),
            suggestion: None,
        }
    }

    pub fn invalid_content(msg: impl Into<String>) -> Self {
        Self::InvalidContent {
            message: msg.into(),
            suggestion: Some("请确保文件为 UTF-8 编码的文本文件。".to_string()),
        }
    }

    pub fn content_hash_mismatch(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::ContentHashMismatch {
            message: msg.into(),
            suggestion,
        }
    }

    pub fn stale_rule_checks(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::StaleRuleChecks {
            message: msg.into(),
            suggestion,
        }
    }

    pub fn invalid_input(msg: impl Into<String>, suggestion: Option<String>) -> Self {
        Self::InvalidInput {
            message: msg.into(),
            suggestion,
        }
    }

    pub fn volume_not_found(msg: impl Into<String>) -> Self {
        Self::NotFound {
            message: msg.into(),
            suggestion: Some("使用 `novel volume list` 查看所有卷。".to_string()),
            code: ErrorCode::VolumeNotFound,
        }
    }

    pub fn volume_already_sealed(msg: impl Into<String>) -> Self {
        Self::PreconditionBlocked {
            message: msg.into(),
            suggestion: Some("卷已封闭，无法再修改。若需归档，请使用 `novel volume archive <id>`。".to_string()),
            code: ErrorCode::VolumeAlreadySealed,
        }
    }

    pub fn chapters_not_approved(msg: impl Into<String>) -> Self {
        Self::PreconditionBlocked {
            message: msg.into(),
            suggestion: Some("请确保卷内所有章节状态为 approved 后再封卷。".to_string()),
            code: ErrorCode::ChaptersNotApproved,
        }
    }

    pub fn volume_not_sealed(msg: impl Into<String>) -> Self {
        Self::PreconditionBlocked {
            message: msg.into(),
            suggestion: Some("只有 sealed 状态的卷可以归档。请先执行 `novel volume seal <id>`。".to_string()),
            code: ErrorCode::VolumeNotSealed,
        }
    }

    pub fn volume_already_archived(msg: impl Into<String>) -> Self {
        Self::PreconditionBlocked {
            message: msg.into(),
            suggestion: Some("卷已归档，无法再修改。".to_string()),
            code: ErrorCode::VolumeAlreadyArchived,
        }
    }

    pub fn invalid_state_transition(msg: impl Into<String>) -> Self {
        Self::InvalidStateTransition {
            message: msg.into(),
            suggestion: Some("请检查当前状态是否允许此操作。".to_string()),
        }
    }

    pub fn thread_name_conflict(msg: impl Into<String>) -> Self {
        Self::ThreadNameConflict {
            message: msg.into(),
            suggestion: Some("请使用不同的线索名称。".to_string()),
        }
    }

    pub fn mainplot_node_not_found(msg: impl Into<String>) -> Self {
        Self::MainplotNodeNotFound {
            message: msg.into(),
            suggestion: Some("使用 `novel mainplot node list <volume-id>` 查看所有主线节点。".to_string()),
        }
    }

    pub fn thread_not_found(msg: impl Into<String>) -> Self {
        Self::ThreadNotFound {
            message: msg.into(),
            suggestion: Some("使用 `novel thread list <volume-id>` 查看所有线索。".to_string()),
        }
    }

    pub fn emergence_not_found(msg: impl Into<String>) -> Self {
        Self::EmergenceNotFound {
            message: msg.into(),
            suggestion: Some("使用 `novel emergence list <volume-id>` 查看所有涌现。".to_string()),
        }
    }

    pub fn binding_already_exists(msg: impl Into<String>) -> Self {
        Self::BindingAlreadyExists {
            message: msg.into(),
            suggestion: Some("该线索已绑定到此章节。".to_string()),
        }
    }

    pub fn emergence_archived(msg: impl Into<String>) -> Self {
        Self::EmergenceArchived {
            message: msg.into(),
            suggestion: Some("已归档的涌现不可修改。".to_string()),
        }
    }

    /// 上下文预算超限（chapter-context-packet）。退出码 3，错误码 `context_budget_exceeded`。
    /// 携带部分 packet + budget + failures，让 Spec 看到已装填内容与超限详情。
    pub fn context_budget_exceeded(
        msg: impl Into<String>,
        suggestion: Option<String>,
        partial_packet: crate::domain::ContextPacket,
        failures: Vec<crate::domain::BudgetFailure>,
    ) -> Self {
        let budget_limit = partial_packet.budget.limit;
        let budget_used = partial_packet.budget.used;
        Self::ContextBudgetExceeded {
            message: msg.into(),
            suggestion,
            partial_packet: serde_json::to_value(&partial_packet)
                .unwrap_or(serde_json::Value::Null),
            budget_limit,
            budget_used,
            failures,
        }
    }

    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage { .. } => ExitCode::Usage,
            Self::PreconditionBlocked { .. } => ExitCode::PreconditionBlocked,
            Self::ResourceConflict { .. } => ExitCode::ResourceConflict,
            Self::ExternalModification { .. } => ExitCode::ExternalModification,
            Self::NotFound { .. } => ExitCode::NotFound,
            Self::FileNotFound { .. } => ExitCode::FileNotFound,
            Self::ChapterNotFound { .. } => ExitCode::ChapterNotFound,
            Self::ContentModifiedExternally { .. } => ExitCode::ContentModifiedExternally,
            Self::InvalidContent { .. } => ExitCode::Usage,
            Self::ContentHashMismatch { .. } => ExitCode::ExternalModification,
            Self::StaleRuleChecks { .. } => ExitCode::ExternalModification,
            Self::InvalidInput { .. } => ExitCode::Usage,
            Self::InvalidStateTransition { .. } => ExitCode::InvalidStateTransition,
            Self::ThreadNameConflict { .. } => ExitCode::ThreadNameConflict,
            Self::MainplotNodeNotFound { .. } => ExitCode::MainplotNodeNotFound,
            Self::ThreadNotFound { .. } => ExitCode::ThreadNotFound,
            Self::EmergenceNotFound { .. } => ExitCode::EmergenceNotFound,
            Self::BindingAlreadyExists { .. } => ExitCode::BindingAlreadyExists,
            Self::EmergenceArchived { .. } => ExitCode::EmergenceArchived,
            Self::ContextBudgetExceeded { .. } => ExitCode::PreconditionBlocked,
            Self::Storage { .. } => ExitCode::Storage,
            Self::VerifyIntegrity { exit_code, .. } => *exit_code,
        }
    }

    /// 稳定错误 ID（`ErrorCode` 契约串）。目录化迁移（contract-registry-v2-02）后
    /// 错误 ID 是唯一机器标识，类别与退出码是其派生值；`ErrorCode::category()` 经
    /// `ErrorCatalog` 解析绑定类别。
    pub fn error_id(&self) -> ErrorCode {
        self.code()
    }

    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Usage { .. } => ErrorCode::Usage,
            Self::PreconditionBlocked { code, .. } => code.clone(),
            Self::ResourceConflict { code, .. } => code.clone(),
            Self::ExternalModification { .. } => ErrorCode::ExternalModification,
            Self::NotFound { code, .. } => code.clone(),
            Self::FileNotFound { .. } => ErrorCode::FileNotFound,
            Self::ChapterNotFound { .. } => ErrorCode::ChapterNotFound,
            Self::ContentModifiedExternally { .. } => ErrorCode::ContentModifiedExternally,
            Self::InvalidContent { .. } => ErrorCode::InvalidContent,
            Self::ContentHashMismatch { .. } => ErrorCode::ContentHashMismatch,
            Self::StaleRuleChecks { .. } => ErrorCode::StaleRuleChecks,
            Self::InvalidInput { .. } => ErrorCode::InvalidInput,
            Self::InvalidStateTransition { .. } => ErrorCode::InvalidStateTransition,
            Self::ThreadNameConflict { .. } => ErrorCode::ThreadNameConflict,
            Self::MainplotNodeNotFound { .. } => ErrorCode::MainplotNodeNotFound,
            Self::ThreadNotFound { .. } => ErrorCode::ThreadNotFound,
            Self::EmergenceNotFound { .. } => ErrorCode::EmergenceNotFound,
            Self::BindingAlreadyExists { .. } => ErrorCode::BindingAlreadyExists,
            Self::EmergenceArchived { .. } => ErrorCode::EmergenceArchived,
            Self::ContextBudgetExceeded { .. } => ErrorCode::ContextBudgetExceeded,
            Self::Storage { .. } => ErrorCode::Storage,
            Self::VerifyIntegrity { code, .. } => code.clone(),
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Usage { message, .. }
            | Self::PreconditionBlocked { message, .. }
            | Self::ResourceConflict { message, .. }
            | Self::ExternalModification { message, .. }
            | Self::NotFound { message, .. }
            | Self::FileNotFound { message, .. }
            | Self::ChapterNotFound { message, .. }
            | Self::ContentModifiedExternally { message, .. }
            | Self::InvalidContent { message, .. }
            | Self::ContentHashMismatch { message, .. }
            | Self::StaleRuleChecks { message, .. }
            | Self::InvalidInput { message, .. }
            | Self::InvalidStateTransition { message, .. }
            | Self::ThreadNameConflict { message, .. }
            | Self::MainplotNodeNotFound { message, .. }
            | Self::ThreadNotFound { message, .. }
            | Self::EmergenceNotFound { message, .. }
            | Self::BindingAlreadyExists { message, .. }
            | Self::EmergenceArchived { message, .. }
            | Self::ContextBudgetExceeded { message, .. }
            | Self::Storage { message, .. }
            | Self::VerifyIntegrity { message, .. } => message,
        }
    }

    pub fn suggestion(&self) -> Option<&str> {
        match self {
            Self::Usage { suggestion, .. }
            | Self::PreconditionBlocked { suggestion, .. }
            | Self::ResourceConflict { suggestion, .. }
            | Self::ExternalModification { suggestion, .. }
            | Self::NotFound { suggestion, .. }
            | Self::FileNotFound { suggestion, .. }
            | Self::ChapterNotFound { suggestion, .. }
            | Self::ContentModifiedExternally { suggestion, .. }
            | Self::InvalidContent { suggestion, .. }
            | Self::ContentHashMismatch { suggestion, .. }
            | Self::StaleRuleChecks { suggestion, .. }
            | Self::InvalidInput { suggestion, .. }
            | Self::InvalidStateTransition { suggestion, .. }
            | Self::ThreadNameConflict { suggestion, .. }
            | Self::MainplotNodeNotFound { suggestion, .. }
            | Self::ThreadNotFound { suggestion, .. }
            | Self::EmergenceNotFound { suggestion, .. }
            | Self::BindingAlreadyExists { suggestion, .. }
            | Self::EmergenceArchived { suggestion, .. }
            | Self::ContextBudgetExceeded { suggestion, .. }
            | Self::Storage { suggestion, .. }
            | Self::VerifyIntegrity { suggestion, .. } => suggestion.as_deref(),
        }
    }
}

/// 结构化错误对象（`--json` 输出形态）。字段对应 contracts/cli/json-output.md §3。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorObject {
    pub code: String,
    pub message: String,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// chapter-context 预算超限时附带的预算记账（limit/used）。仅 ContextBudgetExceeded 时出现。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<BudgetDetail>,
    /// chapter-context 预算超限时的失败层记录。仅 ContextBudgetExceeded 时出现。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failures: Option<Vec<crate::domain::BudgetFailure>>,
    /// chapter-context 预算超限时的部分 packet（已装填 fragments + audit.failures）。
    /// 仅 ContextBudgetExceeded 时出现，让 Spec 看到已装填内容。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_packet: Option<serde_json::Value>,
    /// 通用结构化附加信息（design D4）。
    /// `json-output.md` §3 声明的可选 `details` 字段。`VerifyIntegrity` hard 违规时承载
    /// 完整 `IntegrityReport` 序列化（三分项失败项，结构同 `verify-integrity.schema.json`）。
    /// 既有变体为 `None`（向后兼容，不含 `details`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// 错误对象附带的预算记账（chapter-context 预算超限专用）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BudgetDetail {
    pub limit: i64,
    pub used: i64,
}

impl ErrorObject {
    pub fn from_error(e: &CliError) -> Self {
        let code = e.code();
        let message = e.message().to_string();
        let exit_code: i32 = e.exit_code().into();
        let suggestion = e.suggestion().map(|s| s.to_string());

        // chapter-context 预算超限：附带 budget/failures/partialPacket，让 Spec 看到已装填内容。
        if let CliError::ContextBudgetExceeded {
            partial_packet,
            budget_limit,
            budget_used,
            failures,
            ..
        } = e
        {
            return Self {
                code: code.as_str().to_string(),
                message,
                exit_code,
                suggestion,
                budget: Some(BudgetDetail {
                    limit: *budget_limit,
                    used: *budget_used,
                }),
                failures: Some(failures.clone()),
                partial_packet: Some(partial_packet.clone()),
                details: None,
            };
        }

        // verify integrity hard 违规：details 承载完整 IntegrityReport 序列化（三分项失败项）。
        if let CliError::VerifyIntegrity { details, .. } = e {
            return Self {
                code: code.as_str().to_string(),
                message,
                exit_code,
                suggestion,
                budget: None,
                failures: None,
                partial_packet: None,
                details: Some(details.clone()),
            };
        }

        Self {
            code: code.as_str().to_string(),
            message,
            exit_code,
            suggestion,
            budget: None,
            failures: None,
            partial_packet: None,
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    /// V2 目录化迁移（contract-registry-v2-02）D1：全部公开错误 ID 必须登记进
    /// `ErrorCatalog` 真目录，且每个 ID 唯一绑定一个类别。遍历 `ErrorCode` 全变体
    /// 断言目录覆盖（防止新增错误遗漏登记）、类别可解析、退出码可派生。
    #[test]
    fn all_public_error_codes_are_registered_in_catalog() {
        for code in ErrorCode::iter() {
            let id = code.as_str();
            let entry = ErrorCatalog::get(id)
                .unwrap_or_else(|| panic!("错误 {} 未登记进 ErrorCatalog", id));
            // 错误 ID 与目录登记的 ID 一致（as_str 即契约串，目录即权威源）。
            assert_eq!(entry.id, id);
            // 类别唯一绑定可解析。
            assert_eq!(
                code.category(),
                Some(entry.category),
                "错误 {} 经目录解析类别与登记不一致",
                id
            );
            // 退出码可由目录条目派生。
            assert!(ErrorCatalog::exit_code(id).is_some(), "错误 {} 应可派生退出码", id);
        }
    }

    /// V2 目录化迁移 D2 的编译期约束佐证：目录条目与 V2 信封**不携带**独立可写退出码字段，
    /// 退出码只能是 `ErrorCategory::exit_code()` 的派生值——调用点无法为单个错误覆盖退出码。
    /// （`ErrorEntry`/`V2ErrorEnvelope` 类型层面已无独立 exit_code 字段，此处以派生一致性复证。）
    #[test]
    fn catalog_entry_exit_code_is_always_category_derived() {
        for e in ErrorCatalog::ENTRIES {
            assert_eq!(
                e.exit_code(),
                e.category.exit_code(),
                "条目 {} 的退出码必须等于其类别派生值",
                e.id
            );
        }
    }

    /// `CliError::error_id()` 与 `code()` 恒一致（错误 ID 即错误码契约串）。
    #[test]
    fn cli_error_error_id_matches_code() {
        let e = CliError::not_found("缺章节", None);
        assert!(std::ptr::eq(
            e.error_id().as_str(),
            e.code().as_str()
        ) || e.error_id() == e.code());
    }
}
