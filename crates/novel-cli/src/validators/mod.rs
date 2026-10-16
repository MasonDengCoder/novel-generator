// validators 层：可复用校验能力。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §7）：参数校验、引用完整性校验、状态机校验、
// 权限矩阵校验。领域校验由 service/use-case 调用，不只挂在 command 层。
//
// 本切片为占位：仅提供 init/info 用到的参数级校验（name 非空/无路径分隔符、path 存在），
// 状态机/引用完整性校验随各自切片补入。

pub mod chapter_validator;
pub mod emergence_state_validator;
pub mod guards;
pub mod mainplot_validator;
pub mod thread_validator;
pub mod volume_validator;
pub mod volume_seal_validator;

/// 校验项目名：非空且不含路径分隔符。
pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("项目名称 <name> 不得为空。".to_string());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("项目名称 <name> 不得包含路径分隔符（/ 或 \\）。".to_string());
    }
    Ok(())
}
