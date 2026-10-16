// renderers 层：人读输出与 JSON 输出格式化。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §10）：默认输出适合真人阅读；`--json` 输出结构
// 稳定、含 schemaVersion、不含 ANSI/旁白；诊断信息进 stderr 或独立字段；renderer 只做
// 格式化，不决定业务规则。

pub mod chapter_context_renderer;
pub mod chapter_renderer;
pub mod constraint_renderer;
pub mod emergence_renderer;
pub mod error;
pub mod export_renderer;
pub mod info;
pub mod init;
pub mod mainplot_renderer;
pub mod setting_renderer;
pub mod thread_renderer;
pub mod verify_renderer;
pub mod verdict_renderer;
pub mod volume_renderer;

use clap::ArgMatches;
use crate::errors::CliError;

/// 根据是否 `--json` 渲染错误：JSON 模式输出结构化错误对象（stdout，纯 JSON），
/// 否则人读错误（stderr）。
pub fn render_error(e: &CliError, matches: &ArgMatches) {
    error::render_error(e, matches);
}
