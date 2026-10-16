// renderers/setting_renderer.rs
// s01-project-bootstrap：setting 输出渲染器。
//
// 职责（CLAUDE.md §6）：人读格式与 JSON 格式输出；JSON 输出包含 schemaVersion；
// 不含 ANSI 颜色或解释性旁白。

use serde::Serialize;

// contract-governance-03-struct-schema-gen：JsonSchema derive 让本 renderer 的 Serialize
// struct 成为 contracts/cli/json-schemas/*.json 的结构契约源，供 dev-only spec-gen 遍历派生
// schema。struct 改 pub 仅为让 spec-gen 跨 crate 调用 schema_for!（与 02 切片 errors enum
// pub 对称），不改变 `novel` CLI 对外稳定承诺（对外只承诺 bin + CLI Contract）。
use schemars::JsonSchema;

use crate::domain::{Setting, SettingView};

/// JSON 输出结构（单个 setting）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingJson {
    schema_version: String,
    name: String,
    value: String,
    created_at: String,
}

/// JSON 输出结构（setting 列表）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingListJson {
    schema_version: String,
    settings: Vec<SettingItemJson>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingItemJson {
    name: String,
    value: String,
    created_at: String,
}

/// JSON 输出结构（setting render --view，受控视图）。
///
/// 引用 contracts/cli/views-scopes.md §A 组 1。每个 setting 元素带 sourceRefs
/// （`setting:<name>` 格式，按复合键字符串引用形态，见 entity-ids-sourcerefs.md）。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingViewJson {
    schema_version: String,
    view: String,
    settings: Vec<SettingViewItemJson>,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingViewItemJson {
    name: String,
    value: String,
    created_at: String,
    source_refs: Vec<String>,
}

/// 人读格式输出单个 setting。
pub fn render_setting(setting: &Setting) {
    println!("{}: {}", setting.name, setting.value);
}

/// JSON 格式输出单个 setting。
pub fn render_setting_json(setting: &Setting, schema_version: &str) {
    let output = SettingJson {
        schema_version: schema_version.to_string(),
        name: setting.name.clone(),
        value: setting.value.clone(),
        created_at: setting.created_at.clone(),
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 人读格式输出 setting 列表（简单表格）。
pub fn render_setting_list(settings: &[Setting]) {
    if settings.is_empty() {
        println!("No settings defined.");
        return;
    }

    // 简单表格输出
    println!("{:<30} {:<50} {}", "Name", "Value", "Created At");
    println!("{}", "-".repeat(100));
    for s in settings {
        // 截断过长的 value
        let value_display = if s.value.len() > 47 {
            format!("{}...", &s.value[..47])
        } else {
            s.value.clone()
        };
        println!("{:<30} {:<50} {}", s.name, value_display, s.created_at);
    }
}

/// JSON 格式输出 setting 列表。
pub fn render_setting_list_json(settings: &[Setting], schema_version: &str) {
    let items: Vec<SettingItemJson> = settings
        .iter()
        .map(|s| SettingItemJson {
            name: s.name.clone(),
            value: s.value.clone(),
            created_at: s.created_at.clone(),
        })
        .collect();

    let output = SettingListJson {
        schema_version: schema_version.to_string(),
        settings: items,
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 派生 setting 的 sourceRefs：`setting:<name>`（按复合键字符串引用形态）。
fn setting_source_refs(name: &str) -> Vec<String> {
    vec![format!("setting:{}", name)]
}

/// JSON 格式输出 setting render --view（受控视图）。
pub fn render_setting_view_json(settings: &[Setting], view: SettingView, schema_version: &str) {
    let items: Vec<SettingViewItemJson> = settings
        .iter()
        .map(|s| SettingViewItemJson {
            name: s.name.clone(),
            value: s.value.clone(),
            created_at: s.created_at.clone(),
            source_refs: setting_source_refs(&s.name),
        })
        .collect();

    let output = SettingViewJson {
        schema_version: schema_version.to_string(),
        view: view.to_string(),
        settings: items,
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// 人读格式输出 setting render --view（受控视图）。
pub fn render_setting_view(settings: &[Setting], view: SettingView) {
    println!("View: {}", view);
    if settings.is_empty() {
        println!("No settings for this view.");
        return;
    }
    println!("{:<30} {:<50} {}", "Name", "Value", "Created At");
    println!("{}", "-".repeat(100));
    for s in settings {
        let value_display = if s.value.len() > 47 {
            format!("{}...", &s.value[..47])
        } else {
            s.value.clone()
        };
        println!("{:<30} {:<50} {}", s.name, value_display, s.created_at);
    }
}
