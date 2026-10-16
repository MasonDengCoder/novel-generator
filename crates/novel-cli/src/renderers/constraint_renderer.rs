// renderers/constraint_renderer.rs
// s01-project-bootstrap：constraint 输出渲染器。
//
// 职责（CLAUDE.md §6）：人读格式与 JSON 格式输出；JSON 输出包含 schemaVersion；
// 不含 ANSI 颜色或解释性旁白。

use serde::Serialize;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让本 renderer 的 Serialize
// struct 成为 json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
// struct 改 pub + 内嵌 struct 提升为模块级，仅为让 spec-gen 跨 crate 调 schema_for!。
use schemars::JsonSchema;

use crate::domain::Constraint;
use crate::services::constraint_service::ConstraintViewResult;
use crate::storage::connection::CURRENT_SCHEMA_VERSION;

/// JSON 输出结构（单个 constraint，包含完整 content）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintJson {
    schema_version: String,
    constraint_type: String,
    scope: String,
    name: String,
    content: String,
    severity: String,
    created_at: String,
}

/// JSON 输出结构（constraint 列表，不含 content）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintListJson {
    schema_version: String,
    constraints: Vec<ConstraintItemJson>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintItemJson {
    constraint_type: String,
    scope: String,
    name: String,
    severity: String,
    created_at: String,
}

/// JSON 输出结构（constraint render --view/--scope 受控视图）。
///
/// 引用 contracts/cli/views-scopes.md §A 组 2/组 3。每个 constraint 元素含完整 content
/// 与 sourceRefs（`constraint:<type>:<scope>:<name>` 格式，按复合键字符串引用形态）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintViewJson {
    schema_version: String,
    view: String,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    chapter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume_id: Option<i64>,
    constraints: Vec<ConstraintViewItemJson>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintViewItemJson {
    constraint_type: String,
    scope: String,
    name: String,
    content: String,
    severity: String,
    created_at: String,
    source_refs: Vec<String>,
}

/// JSON 输出结构（单条规则检查结果）。
///
/// contract-governance-03-struct-schema-gen：从 render_check_results_json 函数内嵌提升为
/// 模块级 pub struct，供 spec-gen 跨 crate 派生 schema。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckResultJson {
    pub id: i64,
    pub constraint_type: String,
    pub constraint_scope: String,
    pub constraint_name: String,
    pub content_hash: String,
    pub severity: String,
    pub passed: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub checked_at: String,
    pub stale: bool,
}

/// JSON 输出结构（规则检查结果列表）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckResultListJson {
    pub schema_version: String,
    pub chapter_id: String,
    pub current_content_hash: String,
    pub checks: Vec<CheckResultJson>,
}

/// JSON 输出结构（批量写入检查结果成功）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckSuccessJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub success: bool,
    pub message: String,
}

/// 人读格式输出单个 constraint。
pub fn render_constraint(constraint: &Constraint) {
    println!("Type: {}", constraint.constraint_type);
    println!("Scope: {}", constraint.scope);
    println!("Name: {}", constraint.name);
    println!("Severity: {}", constraint.severity);
    println!("Created At: {}", constraint.created_at);
    println!("\nContent:");
    println!("{}", constraint.content);
}

/// JSON 格式输出单个 constraint。
pub fn render_constraint_json(constraint: &Constraint, schema_version: &str) {
    let output = ConstraintJson {
        schema_version: schema_version.to_string(),
        constraint_type: constraint.constraint_type.to_string(),
        scope: constraint.scope.clone(),
        name: constraint.name.clone(),
        content: constraint.content.clone(),
        severity: constraint.severity.to_string(),
        created_at: constraint.created_at.clone(),
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 人读格式输出 constraint 列表（简单表格，不含 content）。
pub fn render_constraint_list(constraints: &[Constraint]) {
    if constraints.is_empty() {
        println!("No constraints found.");
        return;
    }

    // 简单表格输出
    println!("{:<15} {:<25} {:<30} {:<10} {}", "Type", "Scope", "Name", "Severity", "Created At");
    println!("{}", "-".repeat(110));
    for c in constraints {
        let name_display = if c.name.len() > 27 {
            format!("{}...", &c.name[..27])
        } else {
            c.name.clone()
        };
        let scope_display = if c.scope.len() > 22 {
            format!("{}...", &c.scope[..22])
        } else {
            c.scope.clone()
        };
        println!(
            "{:<15} {:<25} {:<30} {:<10} {}",
            c.constraint_type.to_string(),
            scope_display,
            name_display,
            c.severity.to_string(),
            c.created_at
        );
    }
}

/// JSON 格式输出 constraint 列表（不含 content）。
pub fn render_constraint_list_json(constraints: &[Constraint], schema_version: &str) {
    let items: Vec<ConstraintItemJson> = constraints
        .iter()
        .map(|c| ConstraintItemJson {
            constraint_type: c.constraint_type.to_string(),
            scope: c.scope.clone(),
            name: c.name.clone(),
            severity: c.severity.to_string(),
            created_at: c.created_at.clone(),
        })
        .collect();

    let output = ConstraintListJson {
        schema_version: schema_version.to_string(),
        constraints: items,
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 派生 constraint 的 sourceRefs：`constraint:<type>:<scope>:<name>`（按复合键字符串引用形态）。
fn constraint_source_refs(c: &Constraint) -> Vec<String> {
    vec![format!(
        "constraint:{}:{}:{}",
        c.constraint_type, c.scope, c.name
    )]
}

/// JSON 格式输出 constraint render --view/--scope（受控视图）。
pub fn render_constraint_view_json(result: &ConstraintViewResult) {
    let items: Vec<ConstraintViewItemJson> = result
        .constraints
        .iter()
        .map(|c| ConstraintViewItemJson {
            constraint_type: c.constraint_type.to_string(),
            scope: c.scope.clone(),
            name: c.name.clone(),
            content: c.content.clone(),
            severity: c.severity.to_string(),
            created_at: c.created_at.clone(),
            source_refs: constraint_source_refs(c),
        })
        .collect();

    let output = ConstraintViewJson {
        schema_version: result.schema_version.clone(),
        view: result.view.to_string(),
        scope: result.scope.to_string(),
        chapter_id: result.chapter_id.clone(),
        volume_id: result.volume_id,
        constraints: items,
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 人读格式输出 constraint render --view/--scope（受控视图）。
pub fn render_constraint_view(result: &ConstraintViewResult) {
    println!("View: {}", result.view);
    println!("Scope: {}", result.scope);
    if let Some(cid) = &result.chapter_id {
        println!("Chapter: {}", cid);
    }
    if let Some(vid) = result.volume_id {
        println!("Volume: {}", vid);
    }
    if result.constraints.is_empty() {
        println!("No constraints for this scope.");
        return;
    }
    println!("{}", "-".repeat(60));
    for c in &result.constraints {
        println!("[{}] {}/{}/{} (severity: {})", c.severity, c.constraint_type, c.scope, c.name, c.severity);
        println!("  {}", c.content);
        println!("  created: {}", c.created_at);
    }
}

/// render_check_results：渲染规则检查结果（s03-rule-checking-engine）。
///
/// 参数：
/// - checks: 检查结果数组
/// - current_hash: 当前章节正文 hash
/// - schema_version: schema 版本
/// - json_mode: 是否 JSON 格式输出
pub fn render_check_results(
    checks: &[crate::repositories::rule_check_repository::RuleCheck],
    current_hash: &str,
    schema_version: &str,
    json_mode: bool,
) {
    if json_mode {
        render_check_results_json(checks, current_hash, schema_version);
    } else {
        render_check_results_human(checks, current_hash);
    }
}

/// JSON 格式输出检查结果。
fn render_check_results_json(
    checks: &[crate::repositories::rule_check_repository::RuleCheck],
    current_hash: &str,
    schema_version: &str,
) {
    let chapter_id = if let Some(first) = checks.first() {
        first.chapter_id.clone()
    } else {
        String::new()
    };

    let items: Vec<CheckResultJson> = checks
        .iter()
        .map(|c| CheckResultJson {
            id: c.id,
            constraint_type: c.constraint_type.clone(),
            constraint_scope: c.constraint_scope.clone(),
            constraint_name: c.constraint_name.clone(),
            content_hash: c.content_hash.clone(),
            severity: c.severity.clone(),
            passed: c.passed,
            description: c.description.clone(),
            location: c.location.clone(),
            checked_at: c.checked_at.clone(),
            stale: c.content_hash != current_hash,
        })
        .collect();

    let output = CheckResultListJson {
        schema_version: schema_version.to_string(),
        chapter_id,
        current_content_hash: current_hash.to_string(),
        checks: items,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 人读格式输出检查结果（简单表格）。
fn render_check_results_human(
    checks: &[crate::repositories::rule_check_repository::RuleCheck],
    current_hash: &str,
) {
    if checks.is_empty() {
        println!("No check results found.");
        return;
    }

    println!(
        "{:<40} {:<10} {:<8} {:<8}",
        "Constraint", "Severity", "Passed", "Stale"
    );
    println!("{}", "-".repeat(70));

    for c in checks {
        let constraint_display = format!("{}/{}/{}", c.constraint_type, c.constraint_scope, c.constraint_name);
        let constraint_display = if constraint_display.chars().count() > 37 {
            let truncated: String = constraint_display.chars().take(37).collect();
            format!("{}...", truncated)
        } else {
            constraint_display
        };

        let passed_display = if c.passed { "✓" } else { "✗" };
        let stale_display = if c.content_hash != current_hash {
            "stale"
        } else {
            "-"
        };

        println!(
            "{:<40} {:<10} {:<8} {:<8}",
            constraint_display, c.severity, passed_display, stale_display
        );
    }
}

/// render_check_success：渲染批量写入成功消息（s03-rule-checking-engine）。
///
/// 参数：
/// - json_mode: 是否 JSON 格式输出
pub fn render_check_success(json_mode: bool) {
    if json_mode {
        let output = CheckSuccessJson {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            success: true,
            message: "检查结果已写入。".to_string(),
        };

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
    // 人读模式静默成功（无输出）
}
