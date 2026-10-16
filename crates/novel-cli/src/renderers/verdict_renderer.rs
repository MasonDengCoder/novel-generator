// verdict_renderer 模块：主线落地裁决的人读与 JSON 输出（s04-state-machine-and-verdicts）。
//
// 职责（CLAUDE.md §6）：renderer 只做格式化，不决定业务规则。
// --json 输出结构稳定、含 schemaVersion、不含 ANSI/旁白（contracts/cli/json-schemas/verdict.schema.json）。

use crate::services::verdict_service::VerdictWritten;
use crate::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::json;

/// 渲染裁决写入成功的 JSON 输出。
pub fn render_verdict_json(verdict: &VerdictWritten) -> String {
    let output = json!({
        "schemaVersion": CURRENT_SCHEMA_VERSION.to_string(),
        "chapterId": verdict.chapter_id,
        "nodeId": verdict.node_id,
        "achieved": if verdict.achieved { 1 } else { 0 },
        "contentHash": verdict.content_hash,
        "createdAt": verdict.created_at,
    });

    serde_json::to_string_pretty(&output).unwrap()
}

/// 渲染裁决写入成功的人读输出。
pub fn render_verdict_human(verdict: &VerdictWritten) -> String {
    format!(
        "✓ 裁决已记录\n\n章节: {}\n主线节点: {}\n达成: {}\n绑定正文 hash: {}\n写入时间: {}\n",
        verdict.chapter_id,
        verdict.node_id,
        if verdict.achieved { "已达成 (1)" } else { "未达成 (0)" },
        verdict.content_hash,
        verdict.created_at,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VerdictWritten {
        VerdictWritten {
            chapter_id: "chapter-001".to_string(),
            node_id: "mp-1-001".to_string(),
            achieved: true,
            content_hash: "hash-001".to_string(),
            created_at: "2026-07-01T01:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_render_verdict_json_achieved() {
        let v = sample();
        let json = render_verdict_json(&v);
        assert!(json.contains("\"schemaVersion\""));
        assert!(json.contains("\"chapterId\": \"chapter-001\""));
        assert!(json.contains("\"nodeId\": \"mp-1-001\""));
        assert!(json.contains("\"achieved\": 1"));
        assert!(json.contains("\"contentHash\": \"hash-001\""));
        assert!(json.contains("\"createdAt\": \"2026-07-01T01:00:00Z\""));
    }

    #[test]
    fn test_render_verdict_json_not_achieved() {
        let mut v = sample();
        v.achieved = false;
        let json = render_verdict_json(&v);
        assert!(json.contains("\"achieved\": 0"));
    }

    #[test]
    fn test_render_verdict_human() {
        let v = sample();
        let output = render_verdict_human(&v);
        assert!(output.contains("裁决已记录"));
        assert!(output.contains("chapter-001"));
        assert!(output.contains("mp-1-001"));
        assert!(output.contains("已达成"));
    }
}
