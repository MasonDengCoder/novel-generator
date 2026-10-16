//! SSOT 解析：从类型化 enum 源构建 outcome→数值 / outcome→错误码 映射。
//!
//! contract-governance-04-specgen-rebuild：本模块的正则解析路径（原 `re_exit_row` /
//! `re_error_row` / `re_subcode_row` 解析手写 `exit-codes.md` §A / `error-codes.md` §A）
//! 已退役。`load_ssot()` 改为包装 `enum_source::load_enum_source()` 迭代器构建 `SsotMaps`：
//! - 退出码数值唯一源：`ExitCode::iter()` 判别式（`*variant as i32`）。
//! - 错误码字符串唯一源：`ErrorCode::iter()` strum serialize 串。
//! - 错误码→退出码映射：spec-gen 侧 `enum_source` 的 aliases（errors 层 `exit_code()` 仍唯一权威）。
//! - 退出码「类别」措辞：来自 `md_render::exit_category()` 叙述映射（`registry_source` 校准用）。
//!
//! outcome 词汇表（@codegen assert.outcome 取值）归一层 `outcome_to_code` 保留不变——它把稳定
//! outcome 词汇映射到错误码，退出码再经 `ErrorCatalog` 类别派生（05 治理 V2 默认）。

use std::collections::HashMap;

use novel_cli::errors::ErrorCatalog;

use crate::enum_source;
use crate::md_render;

/// 退出码数值（类别措辞即 map 键，见 `exit_codes`）。
#[derive(Debug, Clone)]
pub struct ExitEntry {
    pub code: i64,
}

/// 错误码声称对应的退出码（来自 spec-gen aliases，仅交叉校验用；错误码串即 map 键）。
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub stated_exit: Option<i64>,
}

#[derive(Debug, Default)]
pub struct SsotMaps {
    /// outcome 词汇（归一）→ ExitEntry。键为退出码「类别」列措辞（经 `exit_category`）。
    pub exit_codes: HashMap<String, ExitEntry>,
    /// 错误码字符串 → ErrorEntry。来源 `ErrorCode::iter()`。
    pub error_codes: HashMap<String, ErrorEntry>,
    /// 错误码 → 退出码数值（来源 `enum_source` aliases）。
    pub error_to_exit: HashMap<String, i64>,
}

/// 从类型化 enum 源构建 `SsotMaps`（不读手写 md）。
pub fn load_ssot() -> anyhow::Result<SsotMaps> {
    let enums = enum_source::load_enum_source();

    let mut exit_codes: HashMap<String, ExitEntry> = HashMap::new();
    for e in &enums.exit_codes {
        let category = md_render::exit_category(e.code).ok_or_else(|| {
            anyhow::anyhow!("退出码 {} 无叙述类别（spec-gen 叙述映射缺失）", e.code)
        })?;
        exit_codes.insert(category.to_string(), ExitEntry { code: e.code as i64 });
    }

    let mut error_codes: HashMap<String, ErrorEntry> = HashMap::new();
    let mut error_to_exit: HashMap<String, i64> = HashMap::new();
    for e in &enums.error_codes {
        error_codes.insert(e.code.to_string(), ErrorEntry { stated_exit: Some(e.stated_exit as i64) });
        error_to_exit.insert(e.code.to_string(), e.stated_exit as i64);
    }

    Ok(SsotMaps { exit_codes, error_codes, error_to_exit })
}

/// 把错误码解析为退出码数值（来源 `enum_source` aliases）。
pub fn resolve_error_exit(maps: &SsotMaps, code: &str) -> anyhow::Result<i64> {
    if let Some(&n) = maps.error_to_exit.get(code) {
        return Ok(n);
    }
    // 兜底：用 aliases 声称的退出码。
    if let Some(e) = maps.error_codes.get(code) {
        if let Some(n) = e.stated_exit {
            return Ok(n);
        }
    }
    anyhow::bail!("错误码 '{code}' 在 enum 源与 aliases 中均无退出码映射（SSOT 对齐闸失败）")
}

/// @codegen assert.outcome 词汇 → 退出码数值。
///
/// 05 治理默认化（V2 默认）：退出码由错误类别派生。先经 `outcome_to_code` 解析出错误码，
/// 再经 `ErrorCatalog` 类别派生退出码（错误 ID → 类别 → 固定类别码）。
/// 例：outcome: external_modification → 错误码 `content_modified_externally`（integrity）→ 5。
pub fn resolve_outcome_exit(maps: &SsotMaps, outcome: &str) -> anyhow::Result<i64> {
    let code = outcome_to_code(outcome)
        .ok_or_else(|| anyhow::anyhow!("outcome '{outcome}' 未登记错误码别名（spec-gen 需扩别名表）"))?;
    match ErrorCatalog::exit_code(&code) {
        Some(n) => Ok(n as i64),
        None => anyhow::bail!(
            "outcome '{outcome}' → 错误码 '{code}' 在 ErrorCatalog 无类别绑定（V2 目录派生失败）"
        ),
    }
}

/// @codegen assert.outcome 词汇 → 错误码字符串。
/// 例：outcome: external_modification → "content_modified_externally"（细分码优先）。
pub fn resolve_outcome_code(maps: &SsotMaps, outcome: &str) -> anyhow::Result<String> {
    let code = outcome_to_code(outcome)
        .ok_or_else(|| anyhow::anyhow!("outcome '{outcome}' 未登记错误码别名（spec-gen 需扩别名表）"))?;
    if maps.error_codes.contains_key(&code) {
        Ok(code)
    } else {
        anyhow::bail!("outcome '{outcome}' → 错误码 '{code}' 不在 enum 源（SSOT 对齐闸失败）")
    }
}

/// outcome 词汇 → 错误码字符串（细分码优先，便于 Spec 区分场景）。
fn outcome_to_code(outcome: &str) -> Option<String> {
    let s = outcome.trim();
    Some(match s {
        "success" => return None, // 成功无错误码
        "usage" => "usage",
        "precondition_blocked" => "precondition_blocked",
        "resource_conflict" => "resource_conflict",
        "external_modification" => "content_modified_externally", // 细分码
        "state_conflict" => "state_conflict",
        "not_found" => "not_found",
        "file_not_found" => "file_not_found",
        "chapter_not_found" => "chapter_not_found",
        "illegal_state_transition" => "precondition_blocked", // §A 无独立码，归 precondition
        "thread_name_conflict" => "precondition_blocked",
        "mainplot_node_not_found" => "precondition_blocked",
        "thread_not_found" => "precondition_blocked",
        "emergence_not_found" => "precondition_blocked",
        "binding_already_exists" => "precondition_blocked",
        "emergence_archived" => "precondition_blocked",
        "storage" => "storage",
        _ => return None,
    }.to_string())
}
