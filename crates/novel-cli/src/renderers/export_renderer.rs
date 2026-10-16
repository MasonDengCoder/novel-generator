// export_renderer 模块：导出 JSON 渲染（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6）：格式化导出为单一 JSON 对象（含 schemaVersion/scope/exportedAt），
// 正文以字段嵌入。只做格式化，不决定业务规则。schemaVersion 由 export_service 从
// PRAGMA user_version 读取后传入（design D2：renderer 不再接收 `&Connection`，不持有 DB-open）。

use crate::repositories::volume_repository::Volume;
use crate::repositories::{rule_check_repository, verdict_repository};
use crate::services::export_service::{
    ChapterExportData, ExportChapterBody, FullExportData, HashWarning, VolumeExportData, VolumeExportSummary,
};
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。export 的 3 个 Out
// struct 从函数内嵌提升为模块级 pub，供 spec-gen 跨 crate 派生 schema（对应
// export-volume/export-chapter/export-full 三份 schema）。
use schemars::JsonSchema;
use serde::Serialize;

fn exported_at() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VolumeJson {
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

/// `novel export volume` JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportVolumeJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub scope: String,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    pub volume: VolumeJson,
    pub chapters: Vec<ExportChapterBody>,
    pub emergences: Vec<serde_json::Value>,
    pub mainplot_nodes: Vec<serde_json::Value>,
    pub thread_bindings: Vec<serde_json::Value>,
    pub verdicts: Vec<verdict_repository::Verdict>,
    pub rule_checks: Vec<rule_check_repository::RuleCheck>,
    pub warnings: Vec<String>,
}

/// `novel export chapter` JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportChapterJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub scope: String,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    pub chapter: ExportChapterBody,
    pub verdicts: Vec<verdict_repository::Verdict>,
    pub rule_checks: Vec<rule_check_repository::RuleCheck>,
}

/// `novel export full` 单卷结构。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportFullVolumeJson {
    pub volume: VolumeJson,
    pub chapters: Vec<ExportChapterBody>,
}

/// `novel export full` JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportFullJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub scope: String,
    #[serde(rename = "exportedAt")]
    pub exported_at: String,
    pub volumes: Vec<ExportFullVolumeJson>,
    pub warnings: Vec<HashWarning>,
}

impl From<&Volume> for VolumeJson {
    fn from(v: &Volume) -> Self {
        VolumeJson {
            volume_id: v.id,
            name: v.name.clone(),
            description: v.description.clone(),
            order: v.volume_order,
            status: v.status.clone(),
            created_at: v.created_at.clone(),
            updated_at: v.updated_at.clone(),
            sealed_at: v.sealed_at.clone(),
            archived_at: v.archived_at.clone(),
        }
    }
}

/// 渲染 `novel export volume` JSON。
pub fn render_export_volume_json(schema_version: &str, data: &VolumeExportData) -> Result<String, serde_json::Error> {
    let out = ExportVolumeJson {
        schema_version: schema_version.to_string(),
        scope: "volume".to_string(),
        exported_at: exported_at(),
        volume: VolumeJson::from(&data.volume),
        chapters: data.chapters.clone(),
        emergences: data.emergences.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect(),
        mainplot_nodes: data.mainplot_nodes.iter().map(|n| serde_json::to_value(n).unwrap_or_default()).collect(),
        thread_bindings: data.threads.iter().map(|t| serde_json::to_value(t).unwrap_or_default()).collect(),
        verdicts: data.verdicts.clone(),
        rule_checks: data.rule_checks.clone(),
        warnings: data.warnings.clone(),
    };
    serde_json::to_string_pretty(&out)
}

/// 渲染 `novel export chapter` JSON。
pub fn render_export_chapter_json(schema_version: &str, data: &ChapterExportData) -> Result<String, serde_json::Error> {
    let out = ExportChapterJson {
        schema_version: schema_version.to_string(),
        scope: "chapter".to_string(),
        exported_at: exported_at(),
        chapter: data.chapter.clone(),
        verdicts: data.verdicts.clone(),
        rule_checks: data.rule_checks.clone(),
    };
    serde_json::to_string_pretty(&out)
}

/// 渲染 `novel export full` JSON。
pub fn render_export_full_json(schema_version: &str, data: &FullExportData) -> Result<String, serde_json::Error> {
    let volumes: Vec<ExportFullVolumeJson> = data
        .volumes
        .iter()
        .map(|s: &VolumeExportSummary| ExportFullVolumeJson {
            volume: VolumeJson::from(&s.volume),
            chapters: s.chapters.clone(),
        })
        .collect();
    let out = ExportFullJson {
        schema_version: schema_version.to_string(),
        scope: "full".to_string(),
        exported_at: exported_at(),
        volumes,
        warnings: data.warnings.clone(),
    };
    serde_json::to_string_pretty(&out)
}
