// emergence_renderer 模块：涌现输出渲染。

use crate::domain::emergence::Emergence;
use crate::storage::connection::CURRENT_SCHEMA_VERSION;
use comfy_table::{presets::UTF8_FULL, Table};
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::Serialize;

/// 涌现 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct EmergenceJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub id: String,
    #[serde(rename = "volumeId")]
    pub volume_id: i64,
    #[serde(rename = "chapterId")]
    pub chapter_id: Option<String>,
    pub title: String,
    pub description: String,
    #[serde(rename = "discoveredAt")]
    pub discovered_at: String,
    pub status: String,
    pub rating: Option<i64>,
    #[serde(rename = "ratingReason")]
    pub rating_reason: Option<String>,
    #[serde(rename = "ratedAt")]
    pub rated_at: Option<String>,
    pub decision: Option<String>,
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    #[serde(rename = "decidedAt")]
    pub decided_at: Option<String>,
    #[serde(rename = "executionTarget")]
    pub execution_target: Option<String>,
    #[serde(rename = "executionNote")]
    pub execution_note: Option<String>,
    #[serde(rename = "executedAt")]
    pub executed_at: Option<String>,
    #[serde(rename = "archivedAt")]
    pub archived_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// 涌现列表 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct EmergenceListJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub emergences: Vec<EmergenceJson>,
}

/// 渲染单个涌现（人读格式）。
pub fn render_emergence(emergence: &Emergence) {
    println!("涌现：{}", emergence.id);
    println!("  卷：{}", emergence.volume_id);
    if let Some(ref ch_id) = emergence.chapter_id {
        println!("  章节：{}", ch_id);
    }
    println!("  标题：{}", emergence.title);
    println!("  描述：{}", emergence.description);
    println!("  状态：{}", emergence.status.as_str());
    println!("  发现时间：{}", emergence.discovered_at);

    if let Some(rating) = emergence.rating {
        println!("  评分：{}/10", rating);
        if let Some(ref reason) = emergence.rating_reason {
            println!("  评分理由：{}", reason);
        }
    }

    if let Some(ref decision) = emergence.decision {
        println!("  决策：{}", decision.as_str());
        if let Some(ref reason) = emergence.decision_reason {
            println!("  决策理由：{}", reason);
        }
    }

    if let Some(ref target) = emergence.execution_target {
        println!("  执行目标：{}", target);
        if let Some(ref note) = emergence.execution_note {
            println!("  执行备注：{}", note);
        }
    }

    if let Some(ref archived_at) = emergence.archived_at {
        println!("  归档时间：{}", archived_at);
    }
}

/// 渲染涌现列表（人读表格格式）。
pub fn render_emergence_list(emergences: &[Emergence]) {
    if emergences.is_empty() {
        println!("无涌现。");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "标题", "状态", "评分", "决策", "发现时间"]);

    for emergence in emergences {
        let rating = emergence
            .rating
            .map(|r| format!("{}/10", r))
            .unwrap_or_else(|| "-".to_string());

        let decision = emergence
            .decision
            .as_ref()
            .map(|d| d.as_str().to_string())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            &emergence.id,
            &emergence.title,
            emergence.status.as_str(),
            &rating,
            &decision,
            &emergence.discovered_at,
        ]);
    }

    println!("{}", table);
}

/// 渲染单个涌现（JSON 格式）。
pub fn render_emergence_json(emergence: &Emergence) -> Result<(), serde_json::Error> {
    let json = EmergenceJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        id: emergence.id.clone(),
        volume_id: emergence.volume_id,
        chapter_id: emergence.chapter_id.clone(),
        title: emergence.title.clone(),
        description: emergence.description.clone(),
        discovered_at: emergence.discovered_at.clone(),
        status: emergence.status.as_str().to_string(),
        rating: emergence.rating,
        rating_reason: emergence.rating_reason.clone(),
        rated_at: emergence.rated_at.clone(),
        decision: emergence.decision.as_ref().map(|d| d.as_str().to_string()),
        decision_reason: emergence.decision_reason.clone(),
        decided_at: emergence.decided_at.clone(),
        execution_target: emergence.execution_target.clone(),
        execution_note: emergence.execution_note.clone(),
        executed_at: emergence.executed_at.clone(),
        archived_at: emergence.archived_at.clone(),
        created_at: emergence.created_at.clone(),
        updated_at: emergence.updated_at.clone(),
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// 渲染涌现列表（JSON 格式）。
pub fn render_emergence_list_json(emergences: &[Emergence]) -> Result<(), serde_json::Error> {
    let json_emergences: Vec<EmergenceJson> = emergences
        .iter()
        .map(|emergence| EmergenceJson {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            id: emergence.id.clone(),
            volume_id: emergence.volume_id,
            chapter_id: emergence.chapter_id.clone(),
            title: emergence.title.clone(),
            description: emergence.description.clone(),
            discovered_at: emergence.discovered_at.clone(),
            status: emergence.status.as_str().to_string(),
            rating: emergence.rating,
            rating_reason: emergence.rating_reason.clone(),
            rated_at: emergence.rated_at.clone(),
            decision: emergence.decision.as_ref().map(|d| d.as_str().to_string()),
            decision_reason: emergence.decision_reason.clone(),
            decided_at: emergence.decided_at.clone(),
            execution_target: emergence.execution_target.clone(),
            execution_note: emergence.execution_note.clone(),
            executed_at: emergence.executed_at.clone(),
            archived_at: emergence.archived_at.clone(),
            created_at: emergence.created_at.clone(),
            updated_at: emergence.updated_at.clone(),
        })
        .collect();

    let json = EmergenceListJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        emergences: json_emergences,
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
