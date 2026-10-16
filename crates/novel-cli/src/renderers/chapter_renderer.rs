// chapter_renderer 模块：章节人读与 JSON 输出（task 8.1-8.5）。

use crate::repositories::chapter_repository::{Chapter, ChapterListItem};
use crate::storage::connection::CURRENT_SCHEMA_VERSION;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

/// 章节正文写入成功后的最小机器回执。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChapterUpdateJson {
    pub schema_version: String,
    pub id: String,
    pub content_hash: String,
    pub word_count: i64,
    pub updated_at: String,
}

/// 渲染章节正文写入成功后的稳定 JSON 回执。
pub fn render_chapter_update_json(chapter: &Chapter) -> String {
    let output = ChapterUpdateJson {
        schema_version: CURRENT_SCHEMA_VERSION.to_string(),
        id: chapter.id.clone(),
        content_hash: chapter.content_hash.clone(),
        word_count: chapter.word_count,
        updated_at: chapter.updated_at.clone(),
    };

    serde_json::to_string_pretty(&output).unwrap()
}

/// 渲染章节 JSON 输出（包含 schemaVersion 字段）。
///
/// `volume_id` 为 `Some` 时输出 `volumeId` 字段（字符串化整数），为 `None` 时不出现该字段。
pub fn render_chapter_json(chapter: &Chapter) -> String {
    let output = match chapter.volume_id {
        Some(vid) => json!({
            "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
            "id": chapter.id,
            "title": chapter.title,
            "content": chapter.content,
            "contentHash": chapter.content_hash,
            "wordCount": chapter.word_count,
            "status": chapter.status,
            "volumeId": vid.to_string(),
            "createdAt": chapter.created_at,
            "updatedAt": chapter.updated_at,
        }),
        None => json!({
            "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
            "id": chapter.id,
            "title": chapter.title,
            "content": chapter.content,
            "contentHash": chapter.content_hash,
            "wordCount": chapter.word_count,
            "status": chapter.status,
            "createdAt": chapter.created_at,
            "updatedAt": chapter.updated_at,
        }),
    };

    serde_json::to_string_pretty(&output).unwrap()
}

/// 渲染章节人读输出（格式化文本输出）。
pub fn render_chapter_human(chapter: &Chapter) -> String {
    let mut output = String::new();
    output.push_str(&format!("章节 ID: {}\n", chapter.id));
    output.push_str(&format!("标题: {}\n", chapter.title));
    output.push_str(&format!("状态: {}\n", chapter.status));
    output.push_str(&format!("字数: {}\n", chapter.word_count));
    output.push_str(&format!("创建时间: {}\n", chapter.created_at));
    output.push_str(&format!("更新时间: {}\n", chapter.updated_at));
    output.push_str("\n--- 正文 ---\n\n");
    output.push_str(&chapter.content);
    output
}

/// 渲染章节列表 JSON 输出（数组格式，包含 schemaVersion）。
pub fn render_chapter_list_json(chapters: &[ChapterListItem], verify_hash: bool) -> String {
    let chapters_json: Vec<_> = chapters
        .iter()
        .map(|ch| {
            let mut obj = json!({
                "id": ch.id,
                "title": ch.title,
                "wordCount": ch.word_count,
                "createdAt": ch.created_at,
                "updatedAt": ch.updated_at,
            });

            // 如果启用了 hash 验证，添加验证字段
            if verify_hash {
                obj["hashVerified"] = json!(ch.hash_verified);
                if ch.hash_mismatch {
                    obj["hashMismatch"] = json!(true);
                }
            }

            obj
        })
        .collect();

    let output = json!({
        "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
        "chapters": chapters_json,
    });

    serde_json::to_string_pretty(&output).unwrap()
}

/// 渲染章节列表人读输出（表格输出，使用 comfy-table）。
pub fn render_chapter_list_human(chapters: &[ChapterListItem]) -> String {
    use comfy_table::{Table, presets::UTF8_FULL};

    if chapters.is_empty() {
        return "暂无章节。\n".to_string();
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "标题", "字数", "创建时间", "更新时间"]);

    for ch in chapters {
        table.add_row(vec![
            &ch.id,
            &ch.title,
            &ch.word_count.to_string(),
            &ch.created_at,
            &ch.updated_at,
        ]);
    }

    format!("{}\n", table)
}

/// 渲染章节创建成功的 JSON 输出。
///
/// `volume_id` 为 `Some` 时输出 `volumeId` 字段（字符串化整数），为 `None` 时不出现该字段
/// （对齐契约：不传 --volume 时 JSON 不含 volumeId，避免 null 语义）。
pub fn render_chapter_create_json(
    id: &str,
    title: &str,
    created_at: &str,
    volume_id: Option<i64>,
) -> String {
    let output = match volume_id {
        Some(vid) => json!({
            "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
            "id": id,
            "title": title,
            "volumeId": vid.to_string(),
            "createdAt": created_at,
        }),
        None => json!({
            "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
            "id": id,
            "title": title,
            "createdAt": created_at,
        }),
    };

    serde_json::to_string_pretty(&output).unwrap()
}

/// 渲染章节创建成功的人读输出。
pub fn render_chapter_create_human(id: &str, title: &str) -> String {
    format!("✓ 章节创建成功\n\nID: {}\n标题: {}\n", id, title)
}

/// 渲染章节状态推进成功的 JSON 输出。
pub fn render_chapter_status_json(id: &str, status: &str, updated_at: &str) -> String {
    let output = json!({
        "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
        "id": id,
        "status": status,
        "updatedAt": updated_at,
    });

    serde_json::to_string_pretty(&output).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_chapter_json() {
        let chapter = Chapter {
            id: "chapter-001".to_string(),
            title: "第一章".to_string(),
            content: "章节正文内容".to_string(),
            word_count: 100,
            content_hash: "hash123".to_string(),
            status: "drafting".to_string(),
            volume_id: None,
            created_at: "2026-06-29T00:00:00Z".to_string(),
            updated_at: "2026-06-29T01:00:00Z".to_string(),
        };

        let json = render_chapter_json(&chapter);
        assert!(json.contains("\"schemaVersion\""));
        assert!(json.contains("\"id\": \"chapter-001\""));
        assert!(json.contains("\"title\": \"第一章\""));
        assert!(json.contains("\"content\": \"章节正文内容\""));
        assert!(json.contains("\"wordCount\": 100"));
        assert!(json.contains("\"status\": \"drafting\""));
    }

    #[test]
    fn test_render_chapter_human() {
        let chapter = Chapter {
            id: "chapter-001".to_string(),
            title: "第一章".to_string(),
            content: "正文内容".to_string(),
            word_count: 50,
            content_hash: "hash".to_string(),
            status: "drafting".to_string(),
            volume_id: None,
            created_at: "2026-06-29T00:00:00Z".to_string(),
            updated_at: "2026-06-29T01:00:00Z".to_string(),
        };

        let output = render_chapter_human(&chapter);
        assert!(output.contains("章节 ID: chapter-001"));
        assert!(output.contains("标题: 第一章"));
        assert!(output.contains("状态: drafting"));
        assert!(output.contains("字数: 50"));
        assert!(output.contains("正文内容"));
    }

    #[test]
    fn test_render_chapter_list_json() {
        let chapters = vec![
            ChapterListItem {
                id: "chapter-001".to_string(),
                title: "第一章".to_string(),
                word_count: 100,
                created_at: "2026-06-29T00:00:00Z".to_string(),
                updated_at: "2026-06-29T01:00:00Z".to_string(),
                hash_verified: true,
                hash_mismatch: false,
            },
            ChapterListItem {
                id: "chapter-002".to_string(),
                title: "第二章".to_string(),
                word_count: 200,
                created_at: "2026-06-29T02:00:00Z".to_string(),
                updated_at: "2026-06-29T03:00:00Z".to_string(),
                hash_verified: true,
                hash_mismatch: false,
            },
        ];

        let json = render_chapter_list_json(&chapters, false);
        assert!(json.contains("\"schemaVersion\""));
        assert!(json.contains("\"chapters\""));
        assert!(json.contains("\"id\": \"chapter-001\""));
        assert!(json.contains("\"id\": \"chapter-002\""));
        assert!(!json.contains("hashVerified")); // 未启用 verify_hash
    }

    #[test]
    fn test_render_chapter_list_json_with_hash_verification() {
        let chapters = vec![
            ChapterListItem {
                id: "chapter-001".to_string(),
                title: "第一章".to_string(),
                word_count: 100,
                created_at: "2026-06-29T00:00:00Z".to_string(),
                updated_at: "2026-06-29T01:00:00Z".to_string(),
                hash_verified: true,
                hash_mismatch: false,
            },
        ];

        let json = render_chapter_list_json(&chapters, true);
        assert!(json.contains("\"hashVerified\": true"));
        assert!(!json.contains("hashMismatch")); // hash_mismatch 为 false 时不输出
    }

    #[test]
    fn test_render_chapter_list_json_with_hash_mismatch() {
        let chapters = vec![
            ChapterListItem {
                id: "chapter-001".to_string(),
                title: "第一章".to_string(),
                word_count: 100,
                created_at: "2026-06-29T00:00:00Z".to_string(),
                updated_at: "2026-06-29T01:00:00Z".to_string(),
                hash_verified: false,
                hash_mismatch: true,
            },
        ];

        let json = render_chapter_list_json(&chapters, true);
        assert!(json.contains("\"hashVerified\": false"));
        assert!(json.contains("\"hashMismatch\": true"));
    }

    #[test]
    fn test_render_chapter_list_human_empty() {
        let chapters = vec![];
        let output = render_chapter_list_human(&chapters);
        assert!(output.contains("暂无章节"));
    }

    #[test]
    fn test_render_chapter_list_human_with_data() {
        let chapters = vec![
            ChapterListItem {
                id: "chapter-001".to_string(),
                title: "第一章".to_string(),
                word_count: 100,
                created_at: "2026-06-29T00:00:00Z".to_string(),
                updated_at: "2026-06-29T01:00:00Z".to_string(),
                hash_verified: true,
                hash_mismatch: false,
            },
        ];

        let output = render_chapter_list_human(&chapters);
        assert!(output.contains("chapter-001"));
        assert!(output.contains("第一章"));
        assert!(output.contains("100"));
    }

    #[test]
    fn test_render_chapter_create_json() {
        let json = render_chapter_create_json("chapter-001", "第一章", "2026-06-29T00:00:00Z", None);
        assert!(json.contains("\"schemaVersion\""));
        assert!(json.contains("\"id\": \"chapter-001\""));
        assert!(json.contains("\"title\": \"第一章\""));
        assert!(json.contains("\"createdAt\": \"2026-06-29T00:00:00Z\""));
    }

    #[test]
    fn test_render_chapter_create_human() {
        let output = render_chapter_create_human("chapter-001", "第一章");
        assert!(output.contains("创建成功"));
        assert!(output.contains("chapter-001"));
        assert!(output.contains("第一章"));
    }
}
