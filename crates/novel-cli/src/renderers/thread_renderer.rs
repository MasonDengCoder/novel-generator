// thread_renderer 模块：线索输出渲染。

use crate::domain::thread::{Thread, ThreadBinding};
use crate::storage::connection::CURRENT_SCHEMA_VERSION;
use comfy_table::{presets::UTF8_FULL, Table};
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;
use serde::Serialize;

/// 线索 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct ThreadJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub id: String,
    #[serde(rename = "volumeId")]
    pub volume_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// 线索列表 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct ThreadListJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub threads: Vec<ThreadJson>,
}

/// 线索绑定 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct ThreadBindingJson {
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "chapterId")]
    pub chapter_id: String,
    #[serde(rename = "bindingType")]
    pub binding_type: String,
    pub note: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// 线索绑定列表 JSON 输出结构。
#[derive(Debug, Serialize, JsonSchema)]
pub struct ThreadBindingListJson {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub bindings: Vec<ThreadBindingJson>,
}

/// 渲染单个线索（人读格式）。
pub fn render_thread(thread: &Thread) {
    println!("线索：{}", thread.id);
    println!("  卷：{}", thread.volume_id);
    println!("  名称：{}", thread.name);
    if let Some(ref desc) = thread.description {
        println!("  描述：{}", desc);
    }
    println!("  状态：{}", thread.status.as_str());
    println!("  创建时间：{}", thread.created_at);
    println!("  更新时间：{}", thread.updated_at);
}

/// 渲染线索列表（人读表格格式）。
pub fn render_thread_list(threads: &[Thread]) {
    if threads.is_empty() {
        println!("无线索。");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "名称", "状态", "描述", "创建时间"]);

    for thread in threads {
        let desc = thread
            .description
            .as_ref()
            .map(|d| {
                if d.len() > 30 {
                    format!("{}...", &d[..30])
                } else {
                    d.clone()
                }
            })
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            &thread.id,
            &thread.name,
            thread.status.as_str(),
            &desc,
            &thread.created_at,
        ]);
    }

    println!("{}", table);
}

/// 渲染线索绑定列表（人读表格格式）。
pub fn render_binding_list(bindings: &[ThreadBinding]) {
    if bindings.is_empty() {
        println!("无线索绑定。");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["线索ID", "绑定类型", "备注", "创建时间"]);

    for binding in bindings {
        let note = binding
            .note
            .as_ref()
            .map(|n| {
                if n.len() > 30 {
                    format!("{}...", &n[..30])
                } else {
                    n.clone()
                }
            })
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            &binding.thread_id,
            binding.binding_type.as_str(),
            &note,
            &binding.created_at,
        ]);
    }

    println!("{}", table);
}

/// 渲染单个线索（JSON 格式）。
pub fn render_thread_json(thread: &Thread) -> Result<(), serde_json::Error> {
    let json = ThreadJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        id: thread.id.clone(),
        volume_id: thread.volume_id,
        name: thread.name.clone(),
        description: thread.description.clone(),
        status: thread.status.as_str().to_string(),
        created_at: thread.created_at.clone(),
        updated_at: thread.updated_at.clone(),
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// 渲染线索列表（JSON 格式）。
pub fn render_thread_list_json(threads: &[Thread]) -> Result<(), serde_json::Error> {
    let json_threads: Vec<ThreadJson> = threads
        .iter()
        .map(|thread| ThreadJson {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            id: thread.id.clone(),
            volume_id: thread.volume_id,
            name: thread.name.clone(),
            description: thread.description.clone(),
            status: thread.status.as_str().to_string(),
            created_at: thread.created_at.clone(),
            updated_at: thread.updated_at.clone(),
        })
        .collect();

    let json = ThreadListJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        threads: json_threads,
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// 渲染线索绑定列表（JSON 格式）。
pub fn render_binding_list_json(bindings: &[ThreadBinding]) -> Result<(), serde_json::Error> {
    let json_bindings: Vec<ThreadBindingJson> = bindings
        .iter()
        .map(|binding| ThreadBindingJson {
            thread_id: binding.thread_id.clone(),
            chapter_id: binding.chapter_id.clone(),
            binding_type: binding.binding_type.as_str().to_string(),
            note: binding.note.clone(),
            created_at: binding.created_at.clone(),
        })
        .collect();

    let json = ThreadBindingListJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        bindings: json_bindings,
    };
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// 渲染「线索绑定到章节成功」人读消息（design D4：成功消息收归 renderer）。
pub fn render_bind_success(thread_id: &str, chapter_id: &str) {
    println!("线索 {} 已绑定到章节 {}。", thread_id, chapter_id);
}

/// 渲染「线索状态已更新成功」人读消息。
pub fn render_update_success(thread_id: &str, status: &str) {
    println!("线索 {} 状态已更新为 {}。", thread_id, status);
}
