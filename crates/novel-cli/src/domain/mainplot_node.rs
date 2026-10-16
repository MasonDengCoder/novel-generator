// domain/mainplot_node.rs
// s06-destiny-weaving：MainplotNode 实体。
//
// MainplotNode 是卷内关键情节锚点，用于追踪主线推进。

use serde::{Deserialize, Serialize};

/// 主线节点实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MainplotNode {
    /// 节点 ID（格式：mp-<volume_id>-<seq>）
    pub id: String,
    /// 卷 ID
    pub volume_id: i64,
    /// 序号（卷内唯一）
    pub sequence_number: i64,
    /// 标题
    pub title: String,
    /// 描述
    pub description: Option<String>,
    /// 目标章节 ID 列表（JSON 数组）
    pub target_chapters: Option<Vec<String>>,
    /// 达成状态：0=未达成, 1=已达成
    pub achieved: i64,
    /// 创建时间（ISO 8601 格式）
    pub created_at: String,
    /// 更新时间（ISO 8601 格式）
    pub updated_at: String,
}

impl MainplotNode {
    /// 创建新的 MainplotNode 实例
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        volume_id: i64,
        sequence_number: i64,
        title: String,
        description: Option<String>,
        target_chapters: Option<Vec<String>>,
        achieved: i64,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            volume_id,
            sequence_number,
            title,
            description,
            target_chapters,
            achieved,
            created_at,
            updated_at,
        }
    }
}
