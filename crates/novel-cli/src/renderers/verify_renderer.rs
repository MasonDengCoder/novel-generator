// verify_renderer 模块：完整性验证 JSON 渲染（s07-archive-and-export）。
//
// 职责（CLAUDE.md §6）：格式化验证报告为单一 JSON 对象（含 schemaVersion/verifiedAt），
// 三分项结构。只做格式化，不决定业务规则。schemaVersion 来自 PRAGMA user_version。

use crate::services::verify_service::IntegrityReport;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReportJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    #[serde(rename = "verifiedAt")]
    pub verified_at: String,
    #[serde(rename = "hashIntegrity")]
    pub hash_integrity: HashIntegrityJson,
    #[serde(rename = "referenceIntegrity")]
    pub reference_integrity: ReferenceIntegrityJson,
    #[serde(rename = "stateConsistency")]
    pub state_consistency: StateConsistencyJson,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HashIntegrityJson {
    pub passed: bool,
    pub mismatches: Vec<crate::services::verify_service::HashMismatch>,
    #[serde(rename = "missingFiles")]
    pub missing_files: Vec<crate::services::verify_service::MissingFile>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceIntegrityJson {
    pub passed: bool,
    #[serde(rename = "danglingReferences")]
    pub dangling_references: Vec<crate::services::verify_service::DanglingReference>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StateConsistencyJson {
    pub passed: bool,
    pub inconsistencies: Vec<crate::services::verify_service::Inconsistency>,
}

/// 渲染 `novel verify integrity` JSON。
pub fn render_verify_integrity_json(report: &IntegrityReport) -> Result<String, serde_json::Error> {
    let json = VerifyReportJson {
        schema_version: report.schema_version.clone(),
        verified_at: report.verified_at.clone(),
        hash_integrity: HashIntegrityJson {
            passed: report.hash_integrity.passed,
            mismatches: report.hash_integrity.mismatches.clone(),
            missing_files: report.hash_integrity.missing_files.clone(),
        },
        reference_integrity: ReferenceIntegrityJson {
            passed: report.reference_integrity.passed,
            dangling_references: report.reference_integrity.dangling_references.clone(),
        },
        state_consistency: StateConsistencyJson {
            passed: report.state_consistency.passed,
            inconsistencies: report.state_consistency.inconsistencies.clone(),
        },
    };
    serde_json::to_string_pretty(&json)
}

/// 渲染 `novel verify integrity` 人读摘要。
pub fn render_verify_integrity_human(report: &IntegrityReport) -> String {
    let mut output = String::new();
    output.push_str("=== 完整性验证报告 ===\n\n");

    output.push_str(&format!(
        "[{}] hash 完整性：{} 失配，{} 文件缺失\n",
        if report.hash_integrity.passed { "✓" } else { "✗" },
        report.hash_integrity.mismatches.len(),
        report.hash_integrity.missing_files.len()
    ));
    output.push_str(&format!(
        "[{}] 引用完整性：{} 悬空引用\n",
        if report.reference_integrity.passed { "✓" } else { "✗" },
        report.reference_integrity.dangling_references.len()
    ));
    output.push_str(&format!(
        "[{}] 状态一致性：{} 不一致项\n",
        if report.state_consistency.passed { "✓" } else { "✗" },
        report.state_consistency.inconsistencies.len()
    ));

    output
}
