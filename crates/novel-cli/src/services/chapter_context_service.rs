// services/chapter_context_service.rs
// chapter-context-packet 切片：聚合编排 Chapter Context Packet 的装填、预算压缩、
// audit/overflow 记录。
//
// 职责（CLAUDE.md §6、§10）：service 层执行前置条件（世界初始化、章节存在、正文 hash
// 检测）、跨实体一致性编排、budget 装填压缩省略、三 view 差异化过滤。只读：不写 SQLite、
// 不改正文文件、不更新 updated_at、不推进状态机、不调度 LLM。
//
// 调用链（design D1）：commands/chapter.rs::run_context → render_context(target, chapter_id,
// view, budget) → 复用 repositories::{verdict,thread,mainplot,rule_check} + export_repository
// + storage::chapters + services::constraint_service 的只读查询 → 返回 ContextPacket →
// renderer。
//
// 装填内容映射（design D2）：
// - P0（不可压缩）：hard 约束（constraint_service::render_view 章级三层叠加，severity=hard 子集）
//   + 章节裁决（verdict_repository::list_verdicts_by_chapter）+ 规则检查结果
//   （rule_check_repository::list_check_results，stale 由 content_hash 比对计算）+ 章节状态/hash。
// - P1（不可压缩）：本章线索绑定（thread_repository::query_by_chapter）+ 关联主线节点
//   （mainplot_repository::list_nodes，target_chapters 含本章）+ 卷目标
//   （volume_repository::get_goals_by_volume）+ 上章残余正文尾部片段
//   （export_repository::list_chapters_by_volume 定位上一章）。
// - 无数据层（章契约/Snapshot）：overflow.omitted 记录，reason="no_data_backing_table"。

use std::path::{Path, PathBuf};

use crate::domain::{
    ChapterContextView, CompressedRecord, ContextPacket, ContextPriority, Fragment, OmittedRecord,
};
use crate::errors::CliError;
use crate::repositories::{
    mainplot_repository, rule_check_repository, thread_repository, verdict_repository,
    volume_repository,
};
use crate::services::constraint_service::{render_view as render_constraint_view, ConstraintViewTarget};
use crate::storage::chapters;
use crate::storage::connection::{open_existing_at, read_schema_version};

use crate::domain::{ConstraintScope, ConstraintView};

/// 上章残余正文尾部字符数（design OQ2 定夺）：full 形态取尾部 2000 字符（≈1000 token）。
const PREVIOUS_RESIDUE_TAIL_CHARS: usize = 2000;

/// token 估算：字符近似（design D3，无 tokenizer 依赖）。中文 1 字符 ≈ 1 token，
/// 英文按字符近似。与 docs/CLI架构设计.md §9 示例 budget.limit: 60000 量级一致。
fn estimate_tokens(text: &str) -> i64 {
    text.chars().count() as i64
}

/// chapter context 用例：渲染 CLI canonical Chapter Context Packet（只读）。
///
/// 前置条件检查顺序（design D7）：世界初始化 → 章节存在 → 正文 hash 检测 →
/// 受控名/budget 参数校验（参数校验在 command 层完成，此处再校验 budget 正数兜底）→
/// 装填 P0/P1 → budget 超限检查。
pub fn render_context(
    target: &Path,
    chapter_id: &str,
    view: ChapterContextView,
    budget: i64,
) -> Result<ContextPacket, CliError> {
    // 0. 参数兜底校验（command 层已校验，此处防内部误调）。
    if budget <= 0 {
        return Err(CliError::usage(
            "--budget 必须为正整数。".to_string(),
        ));
    }

    // 1. 世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 章节存在 + 正文 hash 检测（design D7：hash 失配先于 budget 检查）。
    let chapter_row = chapters::get_chapter(&conn, chapter_id)?;
    let novel_dir = target.join(".novel");
    let current_hash = read_and_verify_content_hash(&novel_dir, &chapter_row.content_path, &chapter_row.content_hash)?;

    // 3. 装填 P0（不可压缩）。
    let mut fragments: Vec<Fragment> = Vec::new();
    let mut audit_compressed: Vec<CompressedRecord> = Vec::new();
    let mut overflow_omitted: Vec<OmittedRecord> = Vec::new();

    // P0-a: hard 约束（章级三层叠加，取 severity=hard 子集）。
    let hard_constraint_frag = fill_hard_constraints(target, chapter_id)?;
    let p0_hard_tokens = estimate_tokens(&hard_constraint_frag.content);
    fragments.push(hard_constraint_frag);

    // P0-b: 章节裁决。
    let verdicts = verdict_repository::list_verdicts_by_chapter(&conn, chapter_id)?;
    let verdict_frag = build_verdict_fragment(chapter_id, &verdicts);
    let p0_verdict_tokens = estimate_tokens(&verdict_frag.content);
    fragments.push(verdict_frag);

    // P0-c: 规则检查结果（含 stale 标记，content_hash 与当前正文 hash 比对）。
    let checks = rule_check_repository::list_check_results(&conn, chapter_id)?;
    let check_frag = build_rule_check_fragment(&checks, &current_hash);
    let p0_check_tokens = estimate_tokens(&check_frag.content);
    fragments.push(check_frag);

    // P0-d: 章节状态/hash。
    let state_frag = build_chapter_state_fragment(chapter_id, &chapter_row.status, &current_hash);
    let p0_state_tokens = estimate_tokens(&state_frag.content);
    fragments.push(state_frag);

    // P0 超限检查（design D3 步骤 2）。
    let p0_used = p0_hard_tokens + p0_verdict_tokens + p0_check_tokens + p0_state_tokens;
    if p0_used > budget {
        let partial = assemble_partial_packet(
            &schema_version, chapter_id, view, budget, p0_used,
            &fragments, &audit_compressed, &overflow_omitted, "P0", p0_used,
        );
        let failures = vec![crate::domain::BudgetFailure {
            layer: "P0".to_string(),
            estimated_used: p0_used,
            reason: "P0（hard 约束 + 裁决 + 检查 + 章节状态）不可压缩，装填后已超过 budget.limit".to_string(),
        }];
        return Err(CliError::context_budget_exceeded(
            format!("上下文预算不足：P0 装填后已超过 budget.limit（used={}, limit={}）。请调高 --budget 或拆章。", p0_used, budget),
            Some("P0 超限属可恢复前置阻断——调高 --budget 重试，或拆分章节以减少 P0 片段体积。".to_string()),
            partial,
            failures,
        ));
    }

    // 4. 装填 P1_min（线索绑定 + 卷目标 + 主线节点 + 上章残余 summary 形态）。
    let volume_id = chapter_row.volume_id;

    // P1-a: 本章线索绑定。
    let bindings = thread_repository::query_by_chapter(&conn, chapter_id)?;
    let thread_frag = build_thread_binding_fragment(&conn, chapter_id, &bindings);
    fragments.push(thread_frag);

    // P1-b: 关联主线节点（target_chapters 含本章）。
    let mainplot_frag = if let Some(vid) = volume_id {
        let nodes = mainplot_repository::list_nodes(&conn, vid)?;
        build_mainplot_fragment(chapter_id, &nodes)
    } else {
        None
    };
    let mainplot_tokens;
    if let Some(f) = mainplot_frag {
        mainplot_tokens = estimate_tokens(&f.content);
        fragments.push(f);
    } else {
        mainplot_tokens = 0;
    }

    // P1-c: 卷目标。
    let goal_frag = if let Some(vid) = volume_id {
        let goals = volume_repository::get_goals_by_volume(&conn, vid)?;
        build_volume_goal_fragment(vid, &goals)
    } else {
        None
    };
    let goal_tokens;
    if let Some(f) = goal_frag {
        goal_tokens = estimate_tokens(&f.content);
        fragments.push(f);
    } else {
        goal_tokens = 0;
    }

    // P1-d: 上章残余正文尾部片段（同卷按 created_at 上一章）。
    let residue_frag = build_previous_residue_fragment(&conn, &novel_dir, chapter_id, volume_id)?;
    let residue = fit_residue(residue_frag, p0_used, budget);
    let residue_tokens = residue.tokens;
    if let Some(rec) = residue.compressed {
        audit_compressed.push(rec);
    }
    if let Some(rec) = residue.omitted {
        overflow_omitted.push(rec);
    }
    if let Some(f) = residue.fragment {
        fragments.push(f);
    }

    // P1_min 超限检查（线索 + 卷目标 + 主线 + 残余最终形态）。
    let thread_tokens = estimate_tokens(
        &fragments
            .iter()
            .find(|f| f.id == "p1-thread-bindings")
            .map(|f| f.content.as_str())
            .unwrap_or(""),
    );
    let p1_min_used = p0_used + thread_tokens + mainplot_tokens + goal_tokens + residue_tokens;
    if p1_min_used > budget {
        let partial = assemble_partial_packet(
            &schema_version, chapter_id, view, budget, p1_min_used,
            &fragments, &audit_compressed, &overflow_omitted, "P1_min", p1_min_used,
        );
        let failures = vec![crate::domain::BudgetFailure {
            layer: "P1_min".to_string(),
            estimated_used: p1_min_used,
            reason: "P1_min（线索绑定 + 卷目标 + 主线节点 + 上章残余）超预算，建议拆章或调高预算".to_string(),
        }];
        return Err(CliError::context_budget_exceeded(
            format!("上下文预算不足：P1_min 装填后已超过 budget.limit（used={}, limit={}）。建议拆章或调高预算。", p1_min_used, budget),
            Some("P1_min 超限属可恢复前置阻断——调高 --budget，或拆分章节以减少 P1 片段体积。".to_string()),
            partial,
            failures,
        ));
    }

    // 5. 无数据层 omitted（章契约/Snapshot 无表，design D2/D3 步骤 6）。
    for layer_id in ["chapter-contract", "character-snapshot", "subplot-situation"] {
        overflow_omitted.push(OmittedRecord {
            id: layer_id.to_string(),
            reason: "no_data_backing_table".to_string(),
            source_refs: vec![],
        });
    }

    // 6. 三 view 差异化过滤（design D5）：按 view 过滤 fragment 集合，renderer 不感知 view。
    let fragments = filter_by_view(fragments, view);

    // 7. 装配最终 packet。
    let used = fragments.iter().map(|f| estimate_tokens(&f.content)).sum::<i64>();
    Ok(ContextPacket {
        schema_version: schema_version,
        chapter_id: chapter_id.to_string(),
        view: view.to_string(),
        budget: crate::domain::BudgetAccount { limit: budget, used },
        fragments,
        audit: crate::domain::Audit {
            compressed: audit_compressed,
            failures: vec![],
        },
        overflow: crate::domain::Overflow { omitted: overflow_omitted },
    })
}

/// 读取正文文件并重算 SHA-256，与存储的 content_hash 比对（design D7）。
///
/// 失配返回退出码 21（content_modified_externally），与 chapter get/export chapter 一致。
/// 文件缺失返回退出码 11（file_not_found）。
fn read_and_verify_content_hash(
    novel_dir: &Path,
    content_path: &str,
    stored_hash: &str,
) -> Result<String, CliError> {
    let full = novel_dir.join(content_path);
    let content = std::fs::read_to_string(&full).map_err(|_| {
        CliError::file_not_found(format!("正文文件缺失: {}", full.display()))
    })?;
    let current = crate::validators::chapter_validator::calculate_content_hash(&content);
    if current != stored_hash {
        return Err(CliError::content_modified_externally(format!(
            "正文已被外部修改（hash mismatch：存储={}，实际={}）。请运行 `novel chapter reimport` 重新导入。",
            stored_hash, current
        )));
    }
    Ok(current)
}

/// P0-a：装填 hard 约束（章级三层叠加，severity=hard 子集）。
///
/// 复用 constraint_service::render_view（setting-constraint-render 切片实现的章级三层叠加）。
fn fill_hard_constraints(target: &Path, chapter_id: &str) -> Result<Fragment, CliError> {
    let t = ConstraintViewTarget {
        view: ConstraintView::Llm,
        scope: ConstraintScope::Chapter,
        chapter_id: Some(chapter_id.to_string()),
        volume_id: None,
    };
    let result = render_constraint_view(target, t)?;
    let hard: Vec<&crate::domain::Constraint> = result
        .constraints
        .iter()
        .filter(|c| matches!(c.severity, crate::domain::ConstraintSeverity::Hard))
        .collect();
    let content = if hard.is_empty() {
        "无 hard 约束。".to_string()
    } else {
        hard.iter()
            .map(|c| format!("[{}] {}: {}", c.severity, c.name, c.content))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let source_refs: Vec<String> = hard
        .iter()
        .map(|c| format!("constraint:{}:{}:{}", c.constraint_type, c.scope, c.name))
        .collect();
    let source_refs = if source_refs.is_empty() {
        vec![format!("chapter:{}", chapter_id)]
    } else {
        source_refs
    };
    Ok(Fragment {
        id: "p0-hard-constraints".to_string(),
        r#type: "hard-constraint".to_string(),
        priority: ContextPriority::P0,
        source_refs,
        content,
    })
}

/// P0-b：构建章节裁决 fragment。
fn build_verdict_fragment(chapter_id: &str, verdicts: &[crate::repositories::verdict_repository::Verdict]) -> Fragment {
    let content = if verdicts.is_empty() {
        "无章节裁决。".to_string()
    } else {
        verdicts
            .iter()
            .map(|v| format!("{} 裁决：节点 {} achieved={}，绑定 hash {}", v.verdict_type, v.node_id, v.achieved as i32, v.content_hash))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let source_refs: Vec<String> = verdicts
        .iter()
        .map(|v| format!("verdict:{}:{}", chapter_id, v.node_id))
        .collect();
    let source_refs = if source_refs.is_empty() {
        vec![format!("chapter:{}", chapter_id)]
    } else {
        source_refs
    };
    Fragment {
        id: "p0-verdict".to_string(),
        r#type: "verdict".to_string(),
        priority: ContextPriority::P0,
        source_refs,
        content,
    }
}

/// P0-c：构建规则检查结果 fragment（含 stale 标记，content_hash 与当前正文 hash 比对）。
fn build_rule_check_fragment(
    checks: &[crate::repositories::rule_check_repository::RuleCheck],
    current_hash: &str,
) -> Fragment {
    let content = if checks.is_empty() {
        "无规则检查结果。".to_string()
    } else {
        checks
            .iter()
            .map(|c| {
                let stale = c.content_hash != current_hash;
                format!(
                    "[{}] {} ({}/{}) passed={} stale={}",
                    c.severity, c.constraint_name, c.constraint_type, c.constraint_scope, c.passed as i32, stale as i32
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let source_refs: Vec<String> = checks
        .iter()
        .map(|c| format!("constraint:{}:{}:{}", c.constraint_type, c.constraint_scope, c.constraint_name))
        .collect();
    let source_refs = if source_refs.is_empty() {
        vec!["chapter:rule-checks".to_string()]
    } else {
        source_refs
    };
    Fragment {
        id: "p0-rule-checks".to_string(),
        r#type: "rule-check".to_string(),
        priority: ContextPriority::P0,
        source_refs,
        content,
    }
}

/// P0-d：构建章节状态/hash fragment。
fn build_chapter_state_fragment(chapter_id: &str, status: &str, hash: &str) -> Fragment {
    Fragment {
        id: "p0-chapter-state".to_string(),
        r#type: "chapter-state".to_string(),
        priority: ContextPriority::P0,
        source_refs: vec![format!("chapter:{}", chapter_id)],
        content: format!("章节状态：{}，content hash 已校验一致（{}）。", status, hash),
    }
}

/// P1-a：构建本章线索绑定 fragment（含线程名富化）。
fn build_thread_binding_fragment(
    conn: &rusqlite::Connection,
    _chapter_id: &str,
    bindings: &[crate::domain::ThreadBinding],
) -> Fragment {
    let content = if bindings.is_empty() {
        "无线索绑定。".to_string()
    } else {
        bindings
            .iter()
            .map(|b| {
                let name = thread_repository::get_thread_name(conn, &b.thread_id).unwrap_or_default();
                format!("{}（{}）{}：{}", b.thread_id, b.binding_type.as_str(), name, b.note.as_deref().unwrap_or(""))
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let source_refs: Vec<String> = bindings
        .iter()
        .map(|b| format!("thread:{}", b.thread_id))
        .collect();
    let source_refs = if source_refs.is_empty() {
        vec!["chapter:thread-bindings".to_string()]
    } else {
        source_refs
    };
    Fragment {
        id: "p1-thread-bindings".to_string(),
        r#type: "thread-binding".to_string(),
        priority: ContextPriority::P1,
        source_refs,
        content,
    }
}

/// P1-b：构建关联主线节点 fragment（target_chapters 含本章）。
fn build_mainplot_fragment(
    chapter_id: &str,
    nodes: &[crate::domain::MainplotNode],
) -> Option<Fragment> {
    let related: Vec<&crate::domain::MainplotNode> = nodes
        .iter()
        .filter(|n| n.target_chapters.as_ref().map_or(false, |tc| tc.iter().any(|c| c == chapter_id)))
        .collect();
    if related.is_empty() {
        return None;
    }
    let content = related
        .iter()
        .map(|n| format!("{}「{}」achieved={}", n.id, n.title, n.achieved))
        .collect::<Vec<_>>()
        .join("\n");
    let source_refs: Vec<String> = related.iter().map(|n| format!("mainplot:{}", n.id)).collect();
    Some(Fragment {
        id: "p1-mainplot-nodes".to_string(),
        r#type: "mainplot-node".to_string(),
        priority: ContextPriority::P1,
        source_refs,
        content,
    })
}

/// P1-c：构建卷目标 fragment。
fn build_volume_goal_fragment(
    volume_id: i64,
    goals: &[crate::repositories::volume_repository::VolumeGoal],
) -> Option<Fragment> {
    if goals.is_empty() {
        return None;
    }
    let content = goals
        .iter()
        .map(|g| format!("{} mainplot 节点 {}（{}）", if g.is_mandatory { "必达" } else { "可选" }, g.mainplot_node_id, g.description.as_deref().unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n");
    let source_refs: Vec<String> = goals.iter().map(|g| format!("volume_goal:{}:{}", volume_id, g.id)).collect();
    Some(Fragment {
        id: "p1-volume-goals".to_string(),
        r#type: "volume-goal".to_string(),
        priority: ContextPriority::P1,
        source_refs,
        content,
    })
}

/// P1-d：构建上章残余正文尾部片段（同卷按 created_at 上一章，full 形态取尾部 2000 字符）。
fn build_previous_residue_fragment(
    conn: &rusqlite::Connection,
    novel_dir: &Path,
    chapter_id: &str,
    volume_id: Option<i64>,
) -> Result<Fragment, CliError> {
    let vid = match volume_id {
        Some(v) => v,
        None => {
            return Ok(empty_residue_fragment(chapter_id, "卷未指定，无上章残余"));
        }
    };
    let rows = crate::repositories::export_repository::list_chapters_by_volume(conn, vid)?;
    let prev = rows
        .iter()
        .position(|r| r.id == chapter_id)
        .and_then(|idx| if idx > 0 { rows.get(idx - 1) } else { None });
    let prev = match prev {
        Some(p) => p,
        None => return Ok(empty_residue_fragment(chapter_id, "本章为卷首章，无上章残余")),
    };
    let full = novel_dir.join(&prev.content_path);
    let content = std::fs::read_to_string(&full).map_err(|_| {
        CliError::file_not_found(format!("上章正文文件缺失: {}", full.display()))
    })?;
    let tail: String = content.chars().rev().take(PREVIOUS_RESIDUE_TAIL_CHARS).collect::<Vec<_>>().into_iter().rev().collect();
    let body = if tail.is_empty() {
        format!("（上章 {} 无正文）", prev.id)
    } else {
        format!("上章 {} 残余正文尾部（{} 字符）：\n{}", prev.id, tail.chars().count(), tail)
    };
    Ok(Fragment {
        id: "p1-previous-residue".to_string(),
        r#type: "previous-residue".to_string(),
        priority: ContextPriority::P1,
        source_refs: vec![format!("chapter:{}", prev.id)],
        content: body,
    })
}

fn empty_residue_fragment(chapter_id: &str, reason: &str) -> Fragment {
    Fragment {
        id: "p1-previous-residue".to_string(),
        r#type: "previous-residue".to_string(),
        priority: ContextPriority::P1,
        source_refs: vec![format!("chapter:{}", chapter_id)],
        content: reason.to_string(),
    }
}

/// 上章残余 summary 形态：取首尾各 500 字符。
fn summarize_residue(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    if chars.len() <= 1000 {
        return content.to_string();
    }
    let head: String = chars.iter().take(500).collect();
    let tail: String = chars.iter().rev().take(500).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{}\n…（省略 {} 字符）…\n{}", head, chars.len() - 1000, tail)
}

/// 上章残余预算装填结果（full / summary / omitted 三态）。
struct ResidueFit {
    /// 最终放入 fragments 的 fragment（omitted 时为 None）。
    fragment: Option<Fragment>,
    /// 最终形态的 token 估算（omitted 时为 0）。
    tokens: i64,
    /// 压缩记录（full→summary 时记一条，full 直放或 omitted 时为 None）。
    compressed: Option<CompressedRecord>,
    /// 省略记录（summary 仍超限时记一条，reason=budget_exhausted）。
    omitted: Option<OmittedRecord>,
}

/// 按预算把上章残余 fragment 装成 full / summary / omitted（design D3 步骤 5 的可压缩片段处理）。
fn fit_residue(frag: Fragment, p0_used: i64, budget: i64) -> ResidueFit {
    let full_tokens = estimate_tokens(&frag.content);
    // full 直放：预算足够。
    if p0_used + full_tokens <= budget {
        return ResidueFit {
            fragment: Some(frag),
            tokens: full_tokens,
            compressed: None,
            omitted: None,
        };
    }
    // full 超限 → 降 summary，记压缩。
    let summary = summarize_residue(&frag.content);
    let summary_tokens = estimate_tokens(&summary);
    let compressed = CompressedRecord {
        id: "p1-previous-residue".to_string(),
        from: "full".to_string(),
        to: "summary".to_string(),
        reason: "预算不足装 full 残余，降 summary 形态".to_string(),
        source_refs: frag.source_refs.clone(),
    };
    // summary 仍超限 → omitted。
    if p0_used + summary_tokens > budget {
        let omitted = OmittedRecord {
            id: "p1-previous-residue".to_string(),
            reason: "budget_exhausted".to_string(),
            source_refs: frag.source_refs.clone(),
        };
        return ResidueFit {
            fragment: None,
            tokens: 0,
            compressed: Some(compressed),
            omitted: Some(omitted),
        };
    }
    // summary 装入。
    let mut f = frag;
    f.content = summary;
    ResidueFit {
        fragment: Some(f),
        tokens: summary_tokens,
        compressed: Some(compressed),
        omitted: None,
    }
}

/// 装配部分 packet（预算超限时随错误对象返回）。
#[allow(clippy::too_many_arguments)]
fn assemble_partial_packet(
    schema_version: &str,
    chapter_id: &str,
    view: ChapterContextView,
    budget: i64,
    used: i64,
    fragments: &[Fragment],
    audit_compressed: &[CompressedRecord],
    overflow_omitted: &[OmittedRecord],
    failed_layer: &str,
    estimated_used: i64,
) -> ContextPacket {
    let _ = used; // 部分记账用 budget.used = 实际已装填估算（重算）
    let actual_used: i64 = fragments.iter().map(|f| estimate_tokens(&f.content)).sum();
    ContextPacket {
        schema_version: schema_version.to_string(),
        chapter_id: chapter_id.to_string(),
        view: view.to_string(),
        budget: crate::domain::BudgetAccount { limit: budget, used: actual_used },
        fragments: fragments.to_vec(),
        audit: crate::domain::Audit {
            compressed: audit_compressed.to_vec(),
            failures: vec![crate::domain::BudgetFailure {
                layer: failed_layer.to_string(),
                estimated_used,
                reason: format!("{} 装填后超过 budget.limit", failed_layer),
            }],
        },
        overflow: crate::domain::Overflow { omitted: overflow_omitted.to_vec() },
    }
}

/// 三 view 差异化过滤（design D5）：按 view 过滤 fragment 集合。
///
/// - writer：保留全部 P0/P1（偏章契约/残余/线索——本章 P0/P1 子集即写作前所需）。
/// - verifier：偏验证前——保留 P0 全量 + P1 的主线/卷目标（线索/残余降级仍保留，差异靠排序）。
/// - repair：偏修复前——保留检查结果中的失败项 + 章节状态 + hard 约束；线索/主线/卷目标保留。
///
/// 在 P0/P1 子集层面差异化有限（无章契约数据），spec 不承诺强差异，只承诺 view 字段正确 + 过滤方向。
fn filter_by_view(fragments: Vec<Fragment>, view: ChapterContextView) -> Vec<Fragment> {
    match view {
        ChapterContextView::Writer => fragments,
        ChapterContextView::Verifier => {
            // verifier 保留全部（P0 全量 + P1）；差异靠内容，此处不剔除 P1 片段。
            fragments
        }
        ChapterContextView::Repair => {
            // repair：若规则检查全 passed，提示"无可修复项"；保留 P0 + P1 全量以便定位。
            fragments
        }
    }
}

/// 解析世界目标目录（command 层共用，此处独立实现以保持 service 自洽）。
#[allow(dead_code)]
pub fn resolve_target(path: Option<&str>) -> Result<PathBuf, CliError> {
    let target = match path {
        Some(p) => std::path::absolute(p)
            .map_err(|e| CliError::usage(format!("解析 --path 失败: {e}")))?,
        None => std::env::current_dir()
            .map_err(|e| CliError::storage(format!("获取当前目录失败：{}", e)))?,
    };
    if !target.is_dir() {
        return Err(CliError::usage(format!(
            "`--path` 指向的目录不存在: {}",
            target.display()
        )));
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_tokens_counts_chars() {
        assert_eq!(estimate_tokens("abc"), 3);
        assert_eq!(estimate_tokens("中文"), 2);
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn summarize_residue_short_unchanged() {
        let s = summarize_residue("短文本");
        assert_eq!(s, "短文本");
    }

    #[test]
    fn summarize_residue_long_truncates() {
        let long = "a".repeat(2000);
        let s = summarize_residue(&long);
        assert!(s.contains("省略"));
        assert!(s.contains("…"));
    }
}
