// domain/thread.rs
// s06-destiny-weaving：Thread 和 ThreadBinding 实体。
//
// Thread 是跨章节的情节线，ThreadBinding 是线索与章节的绑定关系。

use serde::{Deserialize, Serialize};

/// 线索状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadStatus {
    Active,
    Paused,
    Resolved,
}

impl ThreadStatus {
    /// 从字符串解析线索状态
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "resolved" => Some(Self::Resolved),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Resolved => "resolved",
        }
    }
}

/// 绑定类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindingType {
    Mention,
    Develop,
    Resolve,
}

impl BindingType {
    /// 从字符串解析绑定类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mention" => Some(Self::Mention),
            "develop" => Some(Self::Develop),
            "resolve" => Some(Self::Resolve),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mention => "mention",
            Self::Develop => "develop",
            Self::Resolve => "resolve",
        }
    }
}

/// 线索实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thread {
    /// 线索 ID（格式：th-<volume_id>-<seq>）
    pub id: String,
    /// 卷 ID
    pub volume_id: i64,
    /// 线索名称（卷内唯一）
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 状态
    pub status: ThreadStatus,
    /// 创建时间（ISO 8601 格式）
    pub created_at: String,
    /// 更新时间（ISO 8601 格式）
    pub updated_at: String,
}

impl Thread {
    /// 创建新的 Thread 实例
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        volume_id: i64,
        name: String,
        description: Option<String>,
        status: ThreadStatus,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            volume_id,
            name,
            description,
            status,
            created_at,
            updated_at,
        }
    }
}

/// 线索绑定实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadBinding {
    /// 线索 ID
    pub thread_id: String,
    /// 章节 ID
    pub chapter_id: String,
    /// 绑定类型
    pub binding_type: BindingType,
    /// 备注
    pub note: Option<String>,
    /// 创建时间（ISO 8601 格式）
    pub created_at: String,
}

impl ThreadBinding {
    /// 创建新的 ThreadBinding 实例
    pub fn new(
        thread_id: String,
        chapter_id: String,
        binding_type: BindingType,
        note: Option<String>,
        created_at: String,
    ) -> Self {
        Self {
            thread_id,
            chapter_id,
            binding_type,
            note,
            created_at,
        }
    }
}
