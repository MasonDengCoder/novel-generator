// renderers/chapter_context_renderer.rs
// chapter-context-packet 切片：Chapter Context Packet 输出渲染器。
//
// 职责（CLAUDE.md §6）：JSON 与人读格式输出；JSON 输出顶层含 schemaVersion/
// chapterId/view/budget/fragments/audit/overflow；人读输出 view/budget 摘要 +
// fragment 列表 + audit/overflow 摘要。不含 ANSI 颜色或解释性旁白。renderer 不感知
// view 差异（过滤在 service 层，见 design D5）。

use crate::domain::ContextPacket;

/// JSON 格式输出 Chapter Context Packet（稳定结构化，顶层含 schemaVersion/
/// chapterId/view/budget/fragments/audit/overflow，见 json-schemas/chapter-context.schema.json）。
pub fn render_context_json(packet: &ContextPacket) -> String {
    serde_json::to_string_pretty(packet).unwrap()
}

/// 人读格式输出 Chapter Context Packet（多行文本，view/budget 摘要 + fragment 列表 +
/// audit/overflow 摘要）。不含 ANSI/旁白。
pub fn render_context_human(packet: &ContextPacket) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Chapter: {}  View: {}  Budget: {}/{}",
        packet.chapter_id, packet.view, packet.budget.used, packet.budget.limit
    ));
    out.push('\n');

    out.push_str(&format!("Fragments ({}):", packet.fragments.len()));
    out.push('\n');
    for f in &packet.fragments {
        out.push_str(&format!(
            "  [{}] {} ({}) — sourceRefs: {}",
            f.priority,
            f.id,
            f.r#type,
            f.source_refs.join(", ")
        ));
        out.push('\n');
        // 截断过长的 content（人读摘要，不全文展示）
        let preview = if f.content.chars().count() > 120 {
            let head: String = f.content.chars().take(117).collect();
            format!("{}...", head)
        } else {
            f.content.clone()
        };
        for line in preview.lines() {
            out.push_str(&format!("    {}", line));
            out.push('\n');
        }
    }

    if !packet.audit.compressed.is_empty() {
        out.push_str(&format!("Audit compressed ({}):", packet.audit.compressed.len()));
        out.push('\n');
        for c in &packet.audit.compressed {
            out.push_str(&format!(
                "  {} {}→{}: {}",
                c.id, c.from, c.to, c.reason
            ));
            out.push('\n');
        }
    }
    if !packet.audit.failures.is_empty() {
        out.push_str(&format!("Audit failures ({}):", packet.audit.failures.len()));
        out.push('\n');
        for fail in &packet.audit.failures {
            out.push_str(&format!(
                "  [{}] used={} : {}",
                fail.layer, fail.estimated_used, fail.reason
            ));
            out.push('\n');
        }
    }
    if !packet.overflow.omitted.is_empty() {
        out.push_str(&format!("Overflow omitted ({}):", packet.overflow.omitted.len()));
        out.push('\n');
        for o in &packet.overflow.omitted {
            out.push_str(&format!("  {}: {}", o.id, o.reason));
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Audit, BudgetAccount, BudgetFailure, CompressedRecord, ContextPacket, ContextPriority,
        Fragment, OmittedRecord, Overflow,
    };

    fn sample_packet() -> ContextPacket {
        ContextPacket {
            schema_version: "8".to_string(),
            chapter_id: "chapter-001".to_string(),
            view: "writer".to_string(),
            budget: BudgetAccount { limit: 60000, used: 10 },
            fragments: vec![Fragment {
                id: "p0-chapter-state".to_string(),
                r#type: "chapter-state".to_string(),
                priority: ContextPriority::P0,
                source_refs: vec!["chapter:chapter-001".to_string()],
                content: "章节状态：drafting".to_string(),
            }],
            audit: Audit {
                compressed: vec![CompressedRecord {
                    id: "p1-previous-residue".to_string(),
                    from: "full".to_string(),
                    to: "summary".to_string(),
                    reason: "预算不足".to_string(),
                    source_refs: vec!["chapter:chapter-000".to_string()],
                }],
                failures: vec![BudgetFailure {
                    layer: "P0".to_string(),
                    estimated_used: 10,
                    reason: "超限".to_string(),
                }],
            },
            overflow: Overflow {
                omitted: vec![OmittedRecord {
                    id: "chapter-contract".to_string(),
                    reason: "no_data_backing_table".to_string(),
                    source_refs: vec![],
                }],
            },
        }
    }

    #[test]
    fn json_output_has_required_keys() {
        let json = render_context_json(&sample_packet());
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["schemaVersion"].as_str().unwrap(), "8");
        assert_eq!(v["chapterId"].as_str().unwrap(), "chapter-001");
        assert_eq!(v["view"].as_str().unwrap(), "writer");
        assert_eq!(v["budget"]["limit"].as_i64().unwrap(), 60000);
        assert!(v["fragments"].is_array());
        assert!(v["audit"]["compressed"].is_array());
        assert!(v["audit"]["failures"].is_array());
        assert!(v["overflow"]["omitted"].is_array());
    }

    #[test]
    fn human_output_contains_view_and_fragments() {
        let human = render_context_human(&sample_packet());
        assert!(human.contains("View: writer"));
        assert!(human.contains("Fragments (1):"));
        assert!(human.contains("[P0] p0-chapter-state"));
        assert!(human.contains("Overflow omitted (1):"));
        assert!(human.contains("chapter-contract"));
    }
}
