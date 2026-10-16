// mainplot_renderer 模块：主线节点输出渲染。

use crate::domain::mainplot_node::MainplotNode;
use crate::storage::connection::CURRENT_SCHEMA_VERSION;
use comfy_table::{presets::UTF8_FULL, Table};
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::Serialize;

/// 主线节点 JSON 输出结构。
///
/// 注意（contracts/cli/json-schemas/mainplot-node.schema.json）：`achieved` 为
/// integer enum [0,1]（0=未达成, 1=已达成），与 domain MainplotNode.achieved (i64) 一致。
/// 不输出 bool，避免稳定 JSON 契约内部矛盾。
#[derive(Debug, Serialize, JsonSchema)]
pub struct MainplotNodeJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub id: String,
    #[serde(rename = "volumeId")]
    pub volume_id: i64,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: i64,
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "targetChapters")]
    pub target_chapters: Option<Vec<String>>,
    pub achieved: i64,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// 主线节点列表 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct MainplotNodeListJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub nodes: Vec<MainplotNodeJson>,
}

/// 渲染单个主线节点（人读格式）。
pub fn render_node(node: &MainplotNode) {
    println!("主线节点：{}", node.id);
    println!("  卷：{}", node.volume_id);
    println!("  序号：{}", node.sequence_number);
    println!("  标题：{}", node.title);
    if let Some(ref desc) = node.description {
        println!("  描述：{}", desc);
    }
    if let Some(ref chapters) = node.target_chapters {
        println!("  目标章节：{}", chapters.join(", "));
    }
    println!(
        "  达成状态：{}",
        if node.achieved == 1 { "已达成" } else { "未达成" }
    );
    println!("  创建时间：{}", node.created_at);
    println!("  更新时间：{}", node.updated_at);
}

/// 渲染主线节点列表（人读表格格式）。
pub fn render_node_list(nodes: &[MainplotNode]) {
    if nodes.is_empty() {
        println!("无主线节点。");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "序号", "标题", "达成", "目标章节"]);

    for node in nodes {
        let achieved = if node.achieved == 1 { "✓" } else { "" };
        let chapters = node
            .target_chapters
            .as_ref()
            .map(|chs| chs.join(", "))
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            &node.id,
            &node.sequence_number.to_string(),
            &node.title,
            achieved,
            &chapters,
        ]);
    }

    println!("{}", table);
}

/// 渲染单个主线节点（JSON 格式）。
pub fn render_node_json(node: &MainplotNode) -> Result<(), serde_json::Error> {
    let json = MainplotNodeJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        id: node.id.clone(),
        volume_id: node.volume_id,
        sequence_number: node.sequence_number,
        title: node.title.clone(),
        description: node.description.clone(),
        target_chapters: node.target_chapters.clone(),
        achieved: node.achieved,
        created_at: node.created_at.clone(),
        updated_at: node.updated_at.clone(),
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// 渲染主线节点列表（JSON 格式）。
pub fn render_node_list_json(nodes: &[MainplotNode]) -> Result<(), serde_json::Error> {
    let json_nodes: Vec<MainplotNodeJson> = nodes
        .iter()
        .map(|node| MainplotNodeJson {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            id: node.id.clone(),
            volume_id: node.volume_id,
            sequence_number: node.sequence_number,
            title: node.title.clone(),
            description: node.description.clone(),
            target_chapters: node.target_chapters.clone(),
            achieved: node.achieved,
            created_at: node.created_at.clone(),
            updated_at: node.updated_at.clone(),
        })
        .collect();

    let json = MainplotNodeListJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        nodes: json_nodes,
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// 渲染「主线节点达成状态已更新」人读消息（design D4：成功消息收归 renderer）。
pub fn render_node_update_success(node_id: &str) {
    println!("主线节点 {} 达成状态已更新。", node_id);
}
