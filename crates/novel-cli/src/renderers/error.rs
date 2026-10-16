// 错误渲染：根据是否 `--json` 输出结构化错误对象或人读错误。
//
// 约定（contracts/cli/json-output.md）：`--json` 模式输出纯 JSON 错误对象到 stdout，
// 不混入 ANSI/旁白；非 JSON 模式输出人读错误到 stderr（含错误码、原因、修复建议）。

use clap::ArgMatches;
use crate::errors::{
    ErrorCatalog, ErrorProtocolVersion, V2ErrorEnvelope, CliError, ErrorObject,
};

fn is_json(matches: &ArgMatches) -> bool {
    if matches.try_contains_id("json").unwrap_or(false) && matches.get_flag("json") {
        return true;
    }

    if let Some((_, sub_matches)) = matches.subcommand() {
        return is_json(sub_matches);
    }

    false
}

/// 渲染错误。缺省协议版本为 **V2**（`ErrorProtocolVersion::default()`，05 治理切片设为
/// 默认）——未显式选择 V1 的命令走 V2 `V2ErrorEnvelope`（类别推导退出码，字段经目录校验）。
pub fn render_error(e: &CliError, matches: &ArgMatches) {
    render_error_inner(e, matches, ErrorProtocolVersion::default())
}

/// 供显式选择 V1 的兼容期路径使用：输出逐字保持 V1 `ErrorObject` 与 per-entity 退出码
/// （V1 回归基线）。迁移动入口见 `contracts/cli/` SSOT 的弃用兼容标注。
pub fn render_error_v1(e: &CliError, matches: &ArgMatches) {
    render_error_inner(e, matches, ErrorProtocolVersion::V1)
}

/// 供显式选择 V2 的命令使用（与缺省 `render_error` 同义的显式入口）。
pub fn render_error_v2(e: &CliError, matches: &ArgMatches) {
    render_error_inner(e, matches, ErrorProtocolVersion::V2)
}

fn render_error_inner(e: &CliError, matches: &ArgMatches, version: ErrorProtocolVersion) {
    if !is_json(matches) {
        eprintln!("❌ [{}] {}", e.code().as_str(), e.message());
        if let Some(s) = e.suggestion() {
            eprintln!("💡 {s}");
        }
        let exit_code = if version == ErrorProtocolVersion::V2 {
            // V2：退出码由目录类别派生；错误 ID 未登记时回退 V1 退出码。
            ErrorCatalog::exit_code(e.error_id().as_str()).unwrap_or_else(|| i32::from(e.exit_code()))
        } else {
            i32::from(e.exit_code())
        };
        eprintln!("（退出码 {exit_code}）");
        return;
    }

    match version {
        // V1：输出结构化 ErrorObject（逐字不变）。
        ErrorProtocolVersion::V1 => {
            let obj = ErrorObject::from_error(e);
            let json = serde_json::to_string_pretty(&obj).unwrap_or_else(|_| {
                "{\"code\":\"STORAGE\",\"message\":\"渲染错误对象失败\",\"exitCode\":70}"
                    .to_string()
            });
            println!("{json}");
        }
        // V2：输出版本化错误信封。类别与退出码由目录派生；错误 ID 未登记时退化为 V1 输出。
        ErrorProtocolVersion::V2 => {
            let id = e.error_id().as_str();
            let Some(entry) = ErrorCatalog::get(id) else {
                // 目录未登记该错误 ID：无法表达 V2 信封，保持 V1 输出（不损坏机器契约）。
                let obj = ErrorObject::from_error(e);
                let json = serde_json::to_string_pretty(&obj).unwrap_or_else(|_| {
                    "{\"code\":\"STORAGE\",\"message\":\"渲染错误对象失败\",\"exitCode\":70}"
                        .to_string()
                });
                println!("{json}");
                return;
            };
            // V2 信封承载与 V1 `ErrorObject` 等价的结构化详情（budget/failures/partialPacket
            // 或 verify details），保证 V2 默认不丢失机器可读信息。recovery 来自目录恢复语义
            // （命令按需附上），此处不把 V1 suggestion 直接当 recovery 语义复用。
            let envelope = V2ErrorEnvelope::new(
                id,
                entry.category,
                e.message().to_string(),
                None,
                v2_details(e),
            );
            let json = serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| {
                "{\"schemaVersion\":\"error-protocol/v2\",\"status\":\"failed\",\"errorId\":\"storage\",\"category\":\"internal\",\"exitCode\":70,\"message\":\"渲染错误信封失败\"}".to_string()
            });
            println!("{json}");
        }
    }
}

/// V2 信封的 `details`：把 V1 `ErrorObject` 承载的结构化详情等价映射到 envelope，使 V2 默认
/// 不丢失机器可读信息。目前两类错误携带结构详情：
/// - `ContextBudgetExceeded`：`budget`/`failures`/`partialPacket`（Spec 需看到已装填内容）；
/// - `VerifyIntegrity`：`details` 承载完整 `IntegrityReport`（与成功路径同构）。
/// 其余错误无结构详情，返回 `None`（envelope 省略 `details`）。
fn v2_details(e: &CliError) -> Option<serde_json::Value> {
    match e {
        CliError::ContextBudgetExceeded {
            partial_packet,
            budget_limit,
            budget_used,
            failures,
            ..
        } => Some(serde_json::json!({
            "budget": { "limit": budget_limit, "used": budget_used },
            "failures": failures,
            "partialPacket": partial_packet,
        })),
        CliError::VerifyIntegrity { details, .. } => Some(details.clone()),
        _ => None,
    }
}
