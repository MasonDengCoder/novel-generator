// domain/emergence.rs
// s06-destiny-weaving：Emergence 实体与状态机。
//
// Emergence 是卷内出现的非规划内容机会，需评分、决策、执行并归档。

use serde::{Deserialize, Serialize};

/// 涌现状态枚举（状态机：pending → rated → decided → executed → archived）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmergenceStatus {
    Pending,
    Rated,
    Decided,
    Executed,
    Archived,
}

impl EmergenceStatus {
    /// 从字符串解析涌现状态
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "rated" => Some(Self::Rated),
            "decided" => Some(Self::Decided),
            "executed" => Some(Self::Executed),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Rated => "rated",
            Self::Decided => "decided",
            Self::Executed => "executed",
            Self::Archived => "archived",
        }
    }

    /// 检查是否可以转换到目标状态
    pub fn can_transition_to(&self, target: EmergenceStatus) -> bool {
        match (self, target) {
            (Self::Pending, Self::Rated) => true,
            (Self::Rated, Self::Decided) => true,
            (Self::Decided, Self::Executed) => true,
            (Self::Executed, Self::Archived) => true,
            _ => false,
        }
    }
}

/// 决策类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Accept,
    Defer,
    Reject,
}

impl Decision {
    /// 从字符串解析决策
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "accept" => Some(Self::Accept),
            "defer" => Some(Self::Defer),
            "reject" => Some(Self::Reject),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Defer => "defer",
            Self::Reject => "reject",
        }
    }
}

/// 涌现实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Emergence {
    /// 涌现 ID（格式：em-<volume_id>-<seq>）
    pub id: String,
    /// 卷 ID
    pub volume_id: i64,
    /// 发现涌现的章节 ID（可选）
    pub chapter_id: Option<String>,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 发现时间（ISO 8601 格式）
    pub discovered_at: String,

    // 状态机字段
    /// 当前状态
    pub status: EmergenceStatus,

    // rated 阶段字段
    /// 评分（0-10）
    pub rating: Option<i64>,
    /// 评分理由
    pub rating_reason: Option<String>,
    /// 评分时间
    pub rated_at: Option<String>,

    // decided 阶段字段
    /// 决策
    pub decision: Option<Decision>,
    /// 决策理由
    pub decision_reason: Option<String>,
    /// 决策时间
    pub decided_at: Option<String>,

    // executed 阶段字段
    /// 执行目标（章节 ID 或其他标识符）
    pub execution_target: Option<String>,
    /// 执行备注
    pub execution_note: Option<String>,
    /// 执行时间
    pub executed_at: Option<String>,

    // archived 阶段字段
    /// 归档时间
    pub archived_at: Option<String>,

    /// 创建时间（ISO 8601 格式）
    pub created_at: String,
    /// 更新时间（ISO 8601 格式）
    pub updated_at: String,
}

impl Emergence {
    /// 创建新的 Emergence 实例
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        volume_id: i64,
        chapter_id: Option<String>,
        title: String,
        description: String,
        discovered_at: String,
        status: EmergenceStatus,
        rating: Option<i64>,
        rating_reason: Option<String>,
        rated_at: Option<String>,
        decision: Option<Decision>,
        decision_reason: Option<String>,
        decided_at: Option<String>,
        execution_target: Option<String>,
        execution_note: Option<String>,
        executed_at: Option<String>,
        archived_at: Option<String>,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            volume_id,
            chapter_id,
            title,
            description,
            discovered_at,
            status,
            rating,
            rating_reason,
            rated_at,
            decision,
            decision_reason,
            decided_at,
            execution_target,
            execution_note,
            executed_at,
            archived_at,
            created_at,
            updated_at,
        }
    }
}
