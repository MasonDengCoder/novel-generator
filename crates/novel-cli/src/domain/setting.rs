// domain/setting.rs
// s01-project-bootstrap：Setting 实体。
//
// Setting 是项目级配置键值对，存储世界观设定、创作风格、技术约束等。
// 采用 UPSERT 语义，覆盖更新，不保留历史版本。

use serde::{Deserialize, Serialize};

/// 项目配置实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    /// 配置项名称（主键）
    pub name: String,
    /// 配置项值
    pub value: String,
    /// 创建时间（ISO 8601 格式）
    pub created_at: String,
}

impl Setting {
    /// 创建新的 Setting 实例
    pub fn new(name: String, value: String, created_at: String) -> Self {
        Self {
            name,
            value,
            created_at,
        }
    }
}
