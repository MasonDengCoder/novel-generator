// domain/chapter_context.rs
// chapter-context-packet 切片：Chapter Context Packet 输出结构与受控 view 枚举。
//
// 职责（CLAUDE.md §6）：定义 ChapterContextView 受控枚举（views-scopes.md §A 组 4）
// 与 ContextPacket/Fragment/Audit/Overflow 输出结构（serde Serialize，sourceRefs 派生）。
// 不承载组合策略与装填编排（策略在 service 层）。

use std::fmt;
use std::str::FromStr;

// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct/enum
// 成为 json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。ContextPriority
// 枚举 serde rename_all="UPPERCASE" 与 schemars 交互经实测验证（design Risk）。
use schemars::JsonSchema;
use serde::Serialize;

/// `novel chapter context --view` 受控名（views-scopes.md §A 组 4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterContextView {
    /// 写作前上下文。
    Writer,
    /// 验证前上下文。
    Verifier,
    /// 修复前上下文。
    Repair,
}

impl ChapterContextView {
    /// 全部受控名（用于错误提示与校验）。
    pub const ALL: [&'static str; 3] = ["writer", "verifier", "repair"];
}

impl fmt::Display for ChapterContextView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChapterContextView::Writer => write!(f, "writer"),
            ChapterContextView::Verifier => write!(f, "verifier"),
            ChapterContextView::Repair => write!(f, "repair"),
        }
    }
}

impl FromStr for ChapterContextView {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "writer" => Ok(ChapterContextView::Writer),
            "verifier" => Ok(ChapterContextView::Verifier),
            "repair" => Ok(ChapterContextView::Repair),
            _ => Err(format!(
                "Invalid chapter context view: '{}'. Must be one of: {}.",
                s,
                ChapterContextView::ALL.join("|")
            )),
        }
    }
}

/// 上下文优先级（views-scopes.md §A 组 4 引用 docs/CLI架构设计.md §9 P0/P1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContextPriority {
    /// Authority/Gate，不可压缩。
    P0,
    /// Chapter Contract 最小集 + 关键绑定，不可压缩。
    P1,
}

impl ContextPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContextPriority::P0 => "P0",
            ContextPriority::P1 => "P1",
        }
    }
}

impl fmt::Display for ContextPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Chapter Context Packet 顶层结构。
///
/// 引用 contracts/cli/json-schemas/chapter-context.schema.json。renderer 只输出此结构，
/// 不感知 view 差异（view 过滤在 service 层）。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextPacket {
    pub schema_version: String,
    pub chapter_id: String,
    pub view: String,
    pub budget: BudgetAccount,
    pub fragments: Vec<Fragment>,
    pub audit: Audit,
    pub overflow: Overflow,
}

/// 预算记账（limit 来自 --budget，used 为已装填 token 估算）。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BudgetAccount {
    pub limit: i64,
    pub used: i64,
}

/// 上下文片段（P0/P1 装填单元）。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Fragment {
    pub id: String,
    pub r#type: String,
    pub priority: ContextPriority,
    pub source_refs: Vec<String>,
    pub content: String,
}

/// 审计记录（压缩记录 + 预算失败记录）。
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    pub compressed: Vec<CompressedRecord>,
    pub failures: Vec<BudgetFailure>,
}

/// 压缩记录（full→summary 阶梯，每次压缩记一条）。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompressedRecord {
    pub id: String,
    pub from: String,
    pub to: String,
    pub reason: String,
    pub source_refs: Vec<String>,
}

/// 预算失败记录（P0/P1_min 超限时记一条，随部分 packet 返回）。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BudgetFailure {
    pub layer: String,
    pub estimated_used: i64,
    pub reason: String,
}

/// 溢出记录（无数据层 omitted 或预算耗尽 omitted）。
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Overflow {
    pub omitted: Vec<OmittedRecord>,
}

/// 省略记录。
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OmittedRecord {
    pub id: String,
    pub reason: String,
    pub source_refs: Vec<String>,
}
