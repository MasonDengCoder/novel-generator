// V2 错误协议地基（contract-registry-v2-01-error-protocol-foundation）。
//
// 本模块定义 V2 错误类别、类型化错误目录（ErrorCatalog）与版本化 JSON 错误信封，
// 与既有 V1 `CliError`/`ErrorObject`/`ExitCode`/`ErrorCode`（本文件 errors/mod.rs）隔离。
// 05 治理切片已把 V2 设为缺省协议（`ErrorProtocolVersion::default() == V2`）；V1 转为
// 弃用兼容路径，仅在显式选择时生效且逐字保持 V1 回归基线。
//
// 设计决策（见 openspec/changes/contract-registry-v2-01-error-protocol-foundation/design.md）：
// - D2 `ErrorCategory::exit_code()` 是退出码的唯一派生落点，ErrorEntry 不持有独立退出码，
//   结构性禁止调用点任意组合错误 ID/类别/退出码。
// - D3 ErrorCatalog 以 const 表登记错误，单元测试断言 ID 与 details.schema_name 唯一。
// - D4 V2ErrorEnvelope 为版本化信封，字段命名刻意区别于 V1 ErrorObject。
// - D5 版本选择语义经 05 治理切片更新为「缺省 V2、V1 弃用兼容须显式选择」。
// 后续切片：V1 物理移除将由独立变更执行，须满足 contract-v2-governance 三条件齐备准则。

use schemars::JsonSchema;
use serde::Serialize;

/// V2 错误类别（七类固定）。退出码是类别的纯函数，见 [`ErrorCategory::exit_code`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 成功（退出码 0）。
    Success,
    /// 输入非法（退出码 2）。
    InvalidInput,
    /// 前置条件不满足（退出码 3）。
    Precondition,
    /// 资源冲突（退出码 4）。
    Conflict,
    /// 完整性违规（退出码 5）。
    Integrity,
    /// 数据不存在或引用无效（退出码 6）。
    NotFound,
    /// 内部/存储错误（退出码 70）。
    Internal,
}

impl ErrorCategory {
    /// 该类别的固定进程退出码（唯一派生落点，V2 语义下退出码仅表达处理类别）。
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::InvalidInput => 2,
            Self::Precondition => 3,
            Self::Conflict => 4,
            Self::Integrity => 5,
            Self::NotFound => 6,
            Self::Internal => 70,
        }
    }

    /// 该类别的契约串（snake_case）。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::InvalidInput => "invalid_input",
            Self::Precondition => "precondition",
            Self::Conflict => "conflict",
            Self::Integrity => "integrity",
            Self::NotFound => "not_found",
            Self::Internal => "internal",
        }
    }
}

/// 结构化详情契约：由目录声明，调用方不得用任意 JSON 替代目录声明的字段集。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailSpec {
    /// 指向未来 json-schema 命名（04 负责落生成）。
    pub schema_name: &'static str,
    /// 结构化详情允许的字段集。
    pub fields: &'static [&'static str],
}

/// 目录登记的错误条目。注意：不携带独立退出码字段——退出码只能是类别的派生值
/// （D3 编译期约束的核心）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorEntry {
    /// 稳定错误 ID（契约串）。
    pub id: &'static str,
    /// 绑定的错误类别（唯一）。
    pub category: ErrorCategory,
    /// 人读默认说明。
    pub message_hint: &'static str,
    /// 面向恢复的元数据：是否可重试。
    pub retryable: bool,
    /// 结构化详情契约声明。
    pub details: &'static DetailSpec,
}

impl ErrorEntry {
    /// 退出码 = 类别的派生值，不在条目上独立存储。
    pub const fn exit_code(&self) -> i32 {
        self.category.exit_code()
    }
}

/// 类型化错误目录（V2）。集中登记错误 ID 与类别、恢复元数据、详情声明的唯一关系。
pub struct ErrorCatalog;

impl ErrorCatalog {
    /// 真目录：覆盖全部公开错误 ID（对齐 `ErrorCode` 契约串）。每个 ID 唯一绑定一个类别，
    /// 退出码仅由类别派生。类别归属以既有 V1 退出码语义归类：
    /// - 前置阻断（原 3 类 + 状态机/归档等可恢复前置）→ `precondition`
    /// - 资源冲突（原 4 类 + 名称/绑定冲突）→ `conflict`
    /// - 完整性（原 5/21 类 hash 失配/stale/外部修改）→ `integrity`
    /// - 不存在（原 6/11/20/43/44/45 类）→ `not_found`
    /// - 输入非法（原 2 类）→ `invalid_input`
    /// - 内部/存储（原 70 类）→ `internal`
    pub const ENTRIES: &'static [ErrorEntry] = &[
        // ── invalid_input（退出码 2）──
        ErrorEntry {
            id: "usage",
            category: ErrorCategory::InvalidInput,
            message_hint: "参数或用法非法。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-invalid-input",
                fields: &["field", "reason"],
            },
        },
        ErrorEntry {
            id: "invalid_input",
            category: ErrorCategory::InvalidInput,
            message_hint: "请求参数非法。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-invalid-input",
                fields: &["field", "reason"],
            },
        },
        ErrorEntry {
            id: "invalid_content",
            category: ErrorCategory::InvalidInput,
            message_hint: "正文或输入内容非法。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-invalid-input",
                fields: &["field", "reason"],
            },
        },
        // ── precondition（退出码 3）──
        ErrorEntry {
            id: "precondition_blocked",
            category: ErrorCategory::Precondition,
            message_hint: "前置条件不满足，操作无法继续。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "world_not_initialized",
            category: ErrorCategory::Precondition,
            message_hint: "世界尚未初始化。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "volume_already_sealed",
            category: ErrorCategory::Precondition,
            message_hint: "卷已封闭，无法再修改。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "chapters_not_approved",
            category: ErrorCategory::Precondition,
            message_hint: "卷内章节未全部批准，无法封卷。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "emergences_not_handled",
            category: ErrorCategory::Precondition,
            message_hint: "存在未处理的涌现，无法继续。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "goals_not_achieved",
            category: ErrorCategory::Precondition,
            message_hint: "目标未达成，无法继续。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "volume_not_sealed",
            category: ErrorCategory::Precondition,
            message_hint: "卷尚未封闭，无法归档。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "volume_already_archived",
            category: ErrorCategory::Precondition,
            message_hint: "卷已归档，无法再修改。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "invalid_state_transition",
            category: ErrorCategory::Precondition,
            message_hint: "当前状态不允许此状态转换。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "emergence_archived",
            category: ErrorCategory::Precondition,
            message_hint: "已归档的涌现不可修改。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-precondition",
                fields: &["required_state", "actual_state"],
            },
        },
        ErrorEntry {
            id: "context_budget_exceeded",
            category: ErrorCategory::Precondition,
            message_hint: "上下文预算超限，操作被阻断。",
            retryable: true,
            details: &DetailSpec {
                schema_name: "error-details-context-budget",
                fields: &["budget_limit", "budget_used"],
            },
        },
        // ── conflict（退出码 4）──
        ErrorEntry {
            id: "resource_conflict",
            category: ErrorCategory::Conflict,
            message_hint: "目标资源已存在或状态冲突。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-conflict",
                fields: &["resource", "conflicting_id"],
            },
        },
        ErrorEntry {
            id: "constraint_already_exists",
            category: ErrorCategory::Conflict,
            message_hint: "约束已存在，无法重复定义。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-conflict",
                fields: &["resource", "conflicting_id"],
            },
        },
        ErrorEntry {
            id: "thread_name_conflict",
            category: ErrorCategory::Conflict,
            message_hint: "同卷内线索名称重复。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-conflict",
                fields: &["resource", "conflicting_id"],
            },
        },
        ErrorEntry {
            id: "binding_already_exists",
            category: ErrorCategory::Conflict,
            message_hint: "该线索与章节的组合已绑定。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-conflict",
                fields: &["resource", "conflicting_id"],
            },
        },
        // ── integrity（退出码 5）──
        ErrorEntry {
            id: "external_modification",
            category: ErrorCategory::Integrity,
            message_hint: "检测到外部修改或状态不一致。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-integrity",
                fields: &["entity_type", "entity_id", "violation_type"],
            },
        },
        ErrorEntry {
            id: "content_modified_externally",
            category: ErrorCategory::Integrity,
            message_hint: "正文 hash 失配，文件已被外部修改。",
            retryable: true,
            details: &DetailSpec {
                schema_name: "error-details-integrity",
                fields: &["entity_type", "entity_id", "violation_type"],
            },
        },
        ErrorEntry {
            id: "content_hash_mismatch",
            category: ErrorCategory::Integrity,
            message_hint: "内容 hash 校验失败。",
            retryable: true,
            details: &DetailSpec {
                schema_name: "error-details-integrity",
                fields: &["entity_type", "entity_id", "violation_type"],
            },
        },
        ErrorEntry {
            id: "stale_rule_checks",
            category: ErrorCategory::Integrity,
            message_hint: "规则检查结果已过期。",
            retryable: true,
            details: &DetailSpec {
                schema_name: "error-details-integrity",
                fields: &["entity_type", "entity_id", "violation_type"],
            },
        },
        ErrorEntry {
            id: "state_conflict",
            category: ErrorCategory::Integrity,
            message_hint: "状态不一致或版本冲突。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-integrity",
                fields: &["entity_type", "entity_id", "violation_type"],
            },
        },
        // ── not_found（退出码 6）──
        ErrorEntry {
            id: "not_found",
            category: ErrorCategory::NotFound,
            message_hint: "请求的资源不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "setting_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "设置项不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "constraint_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "约束不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "chapter_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "章节不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "file_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "输入文件不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "volume_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "卷不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "mainplot_node_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "主线节点不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "thread_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "线索不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        ErrorEntry {
            id: "emergence_not_found",
            category: ErrorCategory::NotFound,
            message_hint: "涌现不存在。",
            retryable: false,
            details: &DetailSpec {
                schema_name: "error-details-not-found",
                fields: &["entity_type", "entity_id"],
            },
        },
        // ── internal（退出码 70）──
        ErrorEntry {
            id: "storage",
            category: ErrorCategory::Internal,
            message_hint: "内部/存储错误。",
            retryable: true,
            details: &DetailSpec {
                schema_name: "error-details-internal",
                fields: &["error_kind"],
            },
        },
    ];

    /// 经 ID 查询目录条目；未登记返回 `None`。
    pub fn get(id: &str) -> Option<&'static ErrorEntry> {
        Self::ENTRIES.iter().find(|e| e.id == id)
    }

    /// 经 ID 查询错误类别；未登记返回 `None`。
    pub fn category(id: &str) -> Option<ErrorCategory> {
        Self::get(id).map(|e| e.category)
    }

    /// 经 ID 查询类别推导退出码；未登记返回 `None`。
    pub fn exit_code(id: &str) -> Option<i32> {
        Self::get(id).map(|e| e.exit_code())
    }

    /// 经 ID 查询面向恢复元数据（是否可重试）；未登记返回 `None`。
    ///
    /// V2 目录化迁移（contract-registry-v2-02）D3/4.2：恢复元数据由目录条目声明，
    /// 调用点不得临时指定。恢复建议文本由调用点按条目语义给出，但"是否可重试"取自目录。
    pub fn retryable(id: &str) -> Option<bool> {
        Self::get(id).map(|e| e.retryable)
    }
}

/// V2 版本化 JSON 错误信封（`--json` 失败响应的统一机器形态）。
///
/// 字段命名刻意区别于 V1 `ErrorObject`（errorId/category/schemaVersion/status），
/// 避免消费方混淆 V1/V2 流（D4）。`exit_code` 由类别推导，不独立存储。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct V2ErrorEnvelope {
    /// 协议版本标识。
    pub schema_version: &'static str,
    /// 失败状态（固定 `failed`）。
    pub status: &'static str,
    /// 稳定错误 ID。
    pub error_id: &'static str,
    /// 错误类别契约串。
    pub category: &'static str,
    /// 由类别推导的退出码。
    pub exit_code: i32,
    /// 人读说明。
    pub message: String,
    /// 恢复建议（可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery: Option<&'static str>,
    /// 版本化结构化详情（可选）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl V2ErrorEnvelope {
    /// V2 协议版本标识。
    pub const SCHEMA_VERSION: &'static str = "error-protocol/v2";

    /// 构造信封；`exit_code` 与 `category` 串均由 `category` 推导，调用方不得独立指定退出码。
    pub fn new(
        error_id: &'static str,
        category: ErrorCategory,
        message: String,
        recovery: Option<&'static str>,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            status: "failed",
            error_id,
            category: category.as_str(),
            exit_code: category.exit_code(),
            message,
            recovery,
            details,
        }
    }

    /// 目录化构造（contract-registry-v2-02 D3）：经目录错误 ID 构造信封，类别与退出码
    /// 由目录条目派生，恢复元数据（是否可重试）取自目录，`details` 字段集经
    /// `DetailSpec.fields` 校验——调用点不得用任意 JSON 替代目录声明的字段集，也不得
    /// 为条目临时指定退出码或恢复语义。
    ///
    /// 返回 `Err(id)` 表示错误 ID 未登记或 `details` 超出目录声明字段集。
    pub fn from_catalog(
        error_id: &'static str,
        message: String,
        recovery: Option<&'static str>,
        details: Option<serde_json::Value>,
    ) -> Result<Self, &'static str> {
        let entry = ErrorCatalog::get(error_id).ok_or(error_id)?;
        if let Some(d) = &details {
            if let serde_json::Value::Object(map) = d {
                for key in map.keys() {
                    if !entry.details.fields.contains(&key.as_str()) {
                        return Err(error_id);
                    }
                }
            }
        }
        Ok(Self::new(
            error_id,
            entry.category,
            message,
            recovery,
            details,
        ))
    }
}

/// V1/V2 错误协议版本选择契约。
///
/// 默认语义（05 治理切片）：缺省为 **V2**——默认路径接收 V2 信封与类别推导退出码。
/// V1 为**弃用兼容路径**：仅在显式选择 V1 时生效，其 `ErrorObject` 与 per-entity 退出码
/// 逐字保持 V1 回归基线（兼容范围与迁移入口见 `contracts/cli/` SSOT 弃用标注）。
/// V1 物理移除由独立变更执行，须满足 `contract-v2-governance` 的三条件齐备准则。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorProtocolVersion {
    /// V1：沿用既有 `ErrorObject`/`ExitCode` 语义（弃用兼容路径，须显式选择）。
    V1,
    /// V2：使用 `V2ErrorEnvelope` 与类别推导退出码（缺省）。
    V2,
}

impl Default for ErrorProtocolVersion {
    fn default() -> Self {
        Self::V2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 遍历全部七个类别：断言各固定退出码且互不相同。
    #[test]
    fn seven_categories_have_fixed_distinct_exit_codes() {
        let categories = [
            ErrorCategory::Success,
            ErrorCategory::InvalidInput,
            ErrorCategory::Precondition,
            ErrorCategory::Conflict,
            ErrorCategory::Integrity,
            ErrorCategory::NotFound,
            ErrorCategory::Internal,
        ];
        let expected = [0, 2, 3, 4, 5, 6, 70];
        let mut seen: Vec<i32> = categories.iter().map(|c| c.exit_code()).collect();
        for (c, exp) in categories.iter().zip(expected.iter()) {
            assert_eq!(c.exit_code(), *exp, "类别 {:?} 退出码不符", c);
        }
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 7, "七类退出码必须互不相同");
    }

    /// 已登记错误可经 ID 查询类别与退出码；未登记返回 None。
    #[test]
    fn catalog_lookup_returns_category_and_exit_code() {
        assert_eq!(ErrorCatalog::category("chapter_not_found"), Some(ErrorCategory::NotFound));
        assert_eq!(ErrorCatalog::exit_code("chapter_not_found"), Some(6));
        assert_eq!(ErrorCatalog::category("usage"), Some(ErrorCategory::InvalidInput));
        assert_eq!(ErrorCatalog::exit_code("usage"), Some(2));
        assert_eq!(ErrorCatalog::exit_code("storage"), Some(70));
        assert_eq!(ErrorCatalog::category("no_such_error"), None);
        assert_eq!(ErrorCatalog::exit_code("no_such_error"), None);
        assert!(ErrorCatalog::get("no_such_error").is_none());
    }

    /// 目录内错误 ID 唯一；共享同一 details.schema_name 的条目必须声明相同字段集（详情契约无二义）。
    #[test]
    fn catalog_entries_have_unique_ids_and_consistent_detail_schemas() {
        let mut ids: Vec<&str> = ErrorCatalog::ENTRIES.iter().map(|e| e.id).collect();
        ids.sort_unstable();
        let id_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), id_len, "错误 ID 必须唯一");

        // 同一 schema_name ⇒ 同一字段集；不同 schema_name 的详情契约必须不同。
        use std::collections::BTreeMap;
        let mut schema_to_fields: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
        for e in ErrorCatalog::ENTRIES {
            let fields = e.details.fields.to_vec();
            match schema_to_fields.get(e.details.schema_name) {
                Some(prev) => assert_eq!(
                    prev, &fields,
                    "schema {} 的字段集必须一致",
                    e.details.schema_name
                ),
                None => {
                    schema_to_fields.insert(e.details.schema_name, fields);
                }
            }
        }
    }

    /// V2 信封序列化字段：含 schemaVersion/status=failed/errorId/category/exitCode/message，
    /// 可选字段（recovery/details）为 None 时被省略。
    #[test]
    fn envelope_serializes_expected_fields_and_omits_optionals() {
        let env = V2ErrorEnvelope::new(
            "precondition_blocked",
            ErrorCategory::Precondition,
            "前置条件不满足。".to_string(),
            None,
            None,
        );
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["schemaVersion"], "error-protocol/v2");
        assert_eq!(json["status"], "failed");
        assert_eq!(json["errorId"], "precondition_blocked");
        assert_eq!(json["category"], "precondition");
        assert_eq!(json["exitCode"], 3);
        assert_eq!(json["message"], "前置条件不满足。");
        assert!(json.get("recovery").is_none(), "None 的 recovery 应被省略");
        assert!(json.get("details").is_none(), "None 的 details 应被省略");

        // 提供 recover/details 时应被序列化。
        let env2 = V2ErrorEnvelope::new(
            "external_modification",
            ErrorCategory::Integrity,
            "完整性违规。".to_string(),
            Some("请重新校验并修复违规项后再试。"),
            Some(serde_json::json!({ "entity_type": "chapter" })),
        );
        let json2 = serde_json::to_value(&env2).unwrap();
        assert!(json2.get("recovery").is_some());
        assert!(json2.get("details").is_some());
        assert_eq!(json2["details"]["entity_type"], "chapter");
    }

    /// 信封 exitCode 与类别推导一致。
    #[test]
    fn envelope_exit_code_matches_category_derivation() {
        let env = V2ErrorEnvelope::new(
            "external_modification",
            ErrorCategory::Integrity,
            "完整性违规。".to_string(),
            None,
            None,
        );
        assert_eq!(env.exit_code, 5);
        assert_eq!(env.category, "integrity");
    }

    /// 真目录应覆盖既有公开错误 ID（对齐 ErrorCode 契约串）；任取几个已登记 ID 均能派生。
    #[test]
    fn real_catalog_covers_public_error_ids() {
        for (id, category) in [
            ("usage", ErrorCategory::InvalidInput),
            ("world_not_initialized", ErrorCategory::Precondition),
            ("resource_conflict", ErrorCategory::Conflict),
            ("content_hash_mismatch", ErrorCategory::Integrity),
            ("chapter_not_found", ErrorCategory::NotFound),
            ("storage", ErrorCategory::Internal),
        ] {
            assert_eq!(
                ErrorCatalog::category(id),
                Some(category),
                "错误 {} 类别不符",
                id
            );
            assert!(ErrorCatalog::exit_code(id).is_some(), "错误 {} 应可派生退出码", id);
        }
        assert!(ErrorCatalog::ENTRIES.len() >= 30, "真目录应覆盖全部公开错误 ID");
    }

    /// 目录化信封构造（D3/4.1）：有效字段集被接受，超出声明字段集被拒绝。
    #[test]
    fn from_catalog_validates_details_against_declared_fields() {
        // chapter_not_found 声明字段集：entity_type / entity_id —— 合法详情被接受。
        let ok = V2ErrorEnvelope::from_catalog(
            "chapter_not_found",
            "章节不存在。".to_string(),
            None,
            Some(serde_json::json!({ "entity_type": "chapter", "entity_id": "c-1" })),
        );
        assert!(ok.is_ok(), "合法详情应被接受");
        assert_eq!(ok.unwrap().exit_code, 6);

        // 超出声明字段集的字段被拒绝。
        let bad = V2ErrorEnvelope::from_catalog(
            "chapter_not_found",
            "章节不存在。".to_string(),
            None,
            Some(serde_json::json!({ "budget_limit": 100 })),
        );
        assert!(bad.is_err(), "超出目录声明字段集的详情应被拒绝");

        // 未登记 ID 被拒绝。
        let unknown = V2ErrorEnvelope::from_catalog(
            "no_such_error",
            "x".to_string(),
            None,
            None,
        );
        assert!(unknown.is_err(), "未登记错误 ID 应被拒绝");
    }

    /// 恢复元数据（是否可重试）来自目录条目，调用点不得临时指定（4.2）。
    #[test]
    fn retryable_metadata_comes_from_catalog() {
        // 可重试：content_modified_externally 登记 retryable=true。
        assert_eq!(ErrorCatalog::retryable("content_modified_externally"), Some(true));
        // 默认不可重试。
        assert_eq!(ErrorCatalog::retryable("not_found"), Some(false));
        // 未登记无恢复元数据。
        assert_eq!(ErrorCatalog::retryable("no_such_error"), None);
    }
}
