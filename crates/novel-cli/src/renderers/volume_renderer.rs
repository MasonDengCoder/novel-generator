// volume_renderer 模块：卷级输出渲染。

use crate::repositories::volume_repository::{Volume, VolumeGoal};
use crate::storage::connection::CURRENT_SCHEMA_VERSION;
use crate::validators::volume_seal_validator::SealCheckResult;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::Serialize;

/// 卷信息 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct VolumeInfoJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "volumeId")]
    pub volume_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub order: i64,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "sealedAt")]
    pub sealed_at: Option<String>,
    #[serde(rename = "archivedAt")]
    pub archived_at: Option<String>,
}

/// 卷列表 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct VolumeListJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub volumes: Vec<VolumeInfoJson>,
}

/// 卷目标 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct VolumeGoalsJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "volumeId")]
    pub volume_id: i64,
    pub goals: Vec<VolumeGoalJson>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct VolumeGoalJson {
    pub id: i64,
    #[serde(rename = "mainplotNodeId")]
    pub mainplot_node_id: String,
    pub description: Option<String>,
    #[serde(rename = "isMandatory")]
    pub is_mandatory: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// 封卷检查结果 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct SealCheckResultsJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "volumeId")]
    pub volume_id: i64,
    #[serde(rename = "dryRun")]
    pub dry_run: bool,
    #[serde(rename = "canSeal")]
    pub can_seal: bool,
    #[serde(rename = "checkResults")]
    pub check_results: Vec<SealCheckJson>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SealCheckJson {
    #[serde(rename = "checkType")]
    pub check_type: String,
    pub passed: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// 渲染卷信息（人读格式）。
pub fn render_volume_info(volume: &Volume) -> String {
    let mut output = String::new();
    output.push_str(&format!("卷 ID: {}\n", volume.id));
    output.push_str(&format!("名称: {}\n", volume.name));
    if let Some(desc) = &volume.description {
        output.push_str(&format!("描述: {}\n", desc));
    }
    output.push_str(&format!("序号: {}\n", volume.volume_order));
    output.push_str(&format!("状态: {}\n", volume.status));
    output.push_str(&format!("创建时间: {}\n", volume.created_at));
    output.push_str(&format!("更新时间: {}\n", volume.updated_at));
    if let Some(sealed) = &volume.sealed_at {
        output.push_str(&format!("封卷时间: {}\n", sealed));
    }
    if let Some(archived) = &volume.archived_at {
        output.push_str(&format!("归档时间: {}\n", archived));
    }
    output
}

/// 渲染卷信息（JSON 格式）。
pub fn render_volume_info_json(volume: &Volume) -> Result<String, serde_json::Error> {
    let json_output = VolumeInfoJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        volume_id: volume.id,
        name: volume.name.clone(),
        description: volume.description.clone(),
        order: volume.volume_order,
        status: volume.status.clone(),
        created_at: volume.created_at.clone(),
        updated_at: volume.updated_at.clone(),
        sealed_at: volume.sealed_at.clone(),
        archived_at: volume.archived_at.clone(),
    };
    serde_json::to_string_pretty(&json_output)
}

/// 渲染卷列表（人读格式）。
pub fn render_volume_list(volumes: &[Volume]) -> String {
    if volumes.is_empty() {
        return "无卷记录\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{:<6} {:<20} {:<8} {:<12} {:<24}\n", "ID", "名称", "序号", "状态", "创建时间"));
    output.push_str(&"-".repeat(80));
    output.push('\n');

    for volume in volumes {
        output.push_str(&format!(
            "{:<6} {:<20} {:<8} {:<12} {:<24}\n",
            volume.id,
            volume.name,
            volume.volume_order,
            volume.status,
            volume.created_at
        ));
    }

    output
}

/// 渲染卷列表（JSON 格式）。
pub fn render_volume_list_json(volumes: &[Volume]) -> Result<String, serde_json::Error> {
    let volume_infos: Vec<VolumeInfoJson> = volumes
        .iter()
        .map(|v| VolumeInfoJson {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            volume_id: v.id,
            name: v.name.clone(),
            description: v.description.clone(),
            order: v.volume_order,
            status: v.status.clone(),
            created_at: v.created_at.clone(),
            updated_at: v.updated_at.clone(),
            sealed_at: v.sealed_at.clone(),
            archived_at: v.archived_at.clone(),
        })
        .collect();

    let json_output = VolumeListJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        volumes: volume_infos,
    };

    serde_json::to_string_pretty(&json_output)
}

/// 渲染卷目标列表（人读格式）。
pub fn render_volume_goals(goals: &[VolumeGoal]) -> String {
    if goals.is_empty() {
        return "无卷目标\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!("{:<6} {:<20} {:<10} {:<30} {:<24}\n", "ID", "主线节点 ID", "必达", "描述", "创建时间"));
    output.push_str(&"-".repeat(100));
    output.push('\n');

    for goal in goals {
        let mandatory = if goal.is_mandatory { "是" } else { "否" };
        let desc = goal.description.as_deref().unwrap_or("");
        output.push_str(&format!(
            "{:<6} {:<20} {:<10} {:<30} {:<24}\n",
            goal.id,
            goal.mainplot_node_id,
            mandatory,
            desc,
            goal.created_at
        ));
    }

    output
}

/// 渲染卷目标列表（JSON 格式）。
pub fn render_volume_goals_json(volume_id: i64, goals: &[VolumeGoal]) -> Result<String, serde_json::Error> {
    let goal_jsons: Vec<VolumeGoalJson> = goals
        .iter()
        .map(|g| VolumeGoalJson {
            id: g.id,
            mainplot_node_id: g.mainplot_node_id.clone(),
            description: g.description.clone(),
            is_mandatory: g.is_mandatory,
            created_at: g.created_at.clone(),
        })
        .collect();

    let json_output = VolumeGoalsJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        volume_id,
        goals: goal_jsons,
    };

    serde_json::to_string_pretty(&json_output)
}

/// 渲染封卷检查结果（人读格式）。
pub fn render_seal_check_results(volume_id: i64, dry_run: bool, results: &[SealCheckResult]) -> String {
    let mut output = String::new();

    if dry_run {
        output.push_str(&format!("=== 封卷干运行检查：卷 {} ===\n\n", volume_id));
    } else {
        output.push_str(&format!("=== 封卷检查：卷 {} ===\n\n", volume_id));
    }

    let all_passed = results.iter().all(|r| r.passed);

    for result in results {
        let status = if result.passed { "✓" } else { "✗" };
        output.push_str(&format!("{} 【{}】{}\n", status, result.check_type, result.message));
        
        if !result.details.is_empty() {
            for detail in &result.details {
                output.push_str(&format!("  - {}\n", detail));
            }
        }
    }

    output.push('\n');

    if all_passed {
        if dry_run {
            output.push_str("✓ 所有封卷前置条件满足，可执行正式封卷。\n");
        } else {
            output.push_str("✓ 封卷成功，卷状态已更新为 sealing。\n");
        }
    } else {
        output.push_str("✗ 封卷前置条件未满足，请修复上述问题后重试。\n");
    }

    output
}

/// 渲染封卷检查结果（JSON 格式）。
pub fn render_seal_check_results_json(
    volume_id: i64,
    dry_run: bool,
    results: &[SealCheckResult],
) -> Result<String, serde_json::Error> {
    let all_passed = results.iter().all(|r| r.passed);

    let check_jsons: Vec<SealCheckJson> = results
        .iter()
        .map(|r| SealCheckJson {
            check_type: r.check_type.clone(),
            passed: r.passed,
            message: r.message.clone(),
            details: r.details.clone(),
        })
        .collect();

    let json_output = SealCheckResultsJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        volume_id,
        dry_run,
        can_seal: all_passed,
        check_results: check_jsons,
    };

    serde_json::to_string_pretty(&json_output)
}

/// 渲染「创建卷成功」人读消息（design D4：成功消息收归 renderer）。
pub fn render_create_success(volume_id: i64) {
    println!("✓ 创建卷成功：ID = {}", volume_id);
}

/// 渲染「归档卷成功」人读消息。
pub fn render_archive_success(volume_id: i64) {
    println!("✓ 归档卷成功：ID = {}", volume_id);
}

/// 渲染「设置卷目标成功」人读消息。
pub fn render_goal_set_success(goal_id: i64) {
    println!("✓ 设置卷目标成功：目标 ID = {}", goal_id);
}

/// 渲染「删除卷目标成功」人读消息。
pub fn render_goal_remove_success(goal_id: i64) {
    println!("✓ 删除卷目标成功：目标 ID = {}", goal_id);
}
