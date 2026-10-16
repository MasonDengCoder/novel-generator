// novel-cli crate 根。
//
// 对外只承诺 `novel` 可执行文件与 CLI Contract（CLAUDE.md §13、CLI模块对齐清单 §13）。
// crate 内部模块均为实现细节，不作为稳定 library API 对外承诺。
//
// 分层调用链（CLAUDE.md §6）：
//   commands -> services -> validators/guard -> repositories -> storage -> renderers
// errors 统一错误类型与退出码映射。

pub mod commands;
pub mod contracts;
pub mod domain;
pub mod errors;
pub mod renderers;
pub mod repositories;
pub mod services;
pub mod storage;
pub mod validators;

// 命令契约注册表（contract-registry-v2-03）：供 spec-gen 等 dev-only 消费方
// 以 lib 形式读取全命令结构化契约与交叉一致性校验入口。
pub use contracts::{register_contracts, CommandContract};

/// CLI 进程入口：解析参数、分发命令、把领域错误映射为退出码。
pub fn run() -> i32 {
    let app = commands::build_cli();
    let matches = match app.try_get_matches() {
        Ok(m) => m,
        Err(e) => {
            // clap 用法错误：退出码 2
            e.print().ok();
            return errors::ExitCode::Usage.into();
        }
    };

    let result = commands::dispatch(&matches);
    match result {
        Ok(()) => errors::ExitCode::Success.into(),
        Err(e) => {
            // 渲染错误（人读或结构化 JSON）并返回对应退出码。
            renderers::render_error(&e, &matches);
            // 05 治理默认化（V2 默认）：进程退出码由错误类别派生，与 `--json` 信封
            // `exitCode` 及人读退出码行一致（json-output.md「exitCode 与进程退出码一致」）。
            // 未登记错误 ID 时回退 V1 per-entity 码（不伪造类别）。V1 弃用兼容路径退出码
            // 仍由 `verify_exit_mapping` / `CliError::exit_code()` 表达（`render_error_v1`）。
            errors::ErrorCatalog::exit_code(e.error_id().as_str())
                .unwrap_or_else(|| i32::from(e.exit_code()))
        }
    }
}
