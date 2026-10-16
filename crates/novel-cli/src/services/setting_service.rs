// services/setting_service.rs
// s01-project-bootstrap：setting 相关业务用例。
//
// 职责（CLAUDE.md §6、§10）：执行状态机、权限矩阵、前置条件校验（如世界已初始化）；
// 编排 storage 层调用；hard 阻断返回非零退出码，不静默降级。

use std::path::{Path, PathBuf};
use time::OffsetDateTime;

use crate::domain::{Setting, SettingView};
use crate::errors::CliError;
use crate::storage::connection::{open_existing_at, read_schema_version};
use crate::storage::settings::{get_setting_by_name, insert_or_update_setting, list_all_settings};

/// 解析 setting 相关命令的目标目录。
pub fn resolve_target(path: Option<&str>) -> Result<PathBuf, CliError> {
    let target = match path {
        Some(p) => std::path::absolute(p)
            .map_err(|e| CliError::usage(format!("解析 --path 失败: {e}")))?,
        None => std::env::current_dir()
            .map_err(|e| CliError::usage(format!("获取当前目录失败: {e}")))?,
    };
    if !target.is_dir() {
        return Err(CliError::usage(format!(
            "`--path` 指向的目录不存在: {}",
            target.display()
        )));
    }
    Ok(target)
}

/// set_setting 用例：设置配置项（UPSERT 语义）。
///
/// 前置条件：世界已初始化（通过 open_existing_at 检查）。
pub fn set_setting(target: &Path, name: &str, value: &str) -> Result<(), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;

    // 2. 生成时间戳
    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间失败: {}", e)))?
        .to_string();

    // 3. UPSERT setting
    insert_or_update_setting(&conn, name, value, &created_at)?;

    Ok(())
}

/// get_setting 用例：查询单个配置项。
///
/// 前置条件：世界已初始化。
/// 不存在则返回 SettingNotFound 错误（退出码 6）。
pub fn get_setting(target: &Path, name: &str) -> Result<(Setting, String), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 查询 setting
    let setting = get_setting_by_name(&conn, name)?;

    // 3. 不存在则返回 NotFound 错误
    let setting = setting.ok_or_else(|| {
        CliError::setting_not_found(
            format!("配置项 '{}' 不存在。", name),
            Some("使用 `novel setting list` 查看所有已定义的配置项。".to_string()),
        )
    })?;

    Ok((setting, schema_version))
}

/// list_settings 用例：列出所有配置项。
///
/// 前置条件：世界已初始化。
pub fn list_settings(target: &Path) -> Result<(Vec<Setting>, String), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 查询所有 settings
    let settings = list_all_settings(&conn)?;
    Ok((settings, schema_version))
}

/// render_view 用例：按受控 view 渲染设定注入视图（只读）。
///
/// 引用 contracts/cli/views-scopes.md §A 组 1 冻结枚举。只读：不写 SQLite、
/// 不改正文文件、不更新 updated_at、不推进状态机。`llm` 为预算内固定组合
/// （intro∪four-poles∪world∪style），不引入 `--budget`（预算化属
/// chapter-context-packet 切片）。
///
/// 前置条件：世界已初始化。
pub fn render_view(target: &Path, view: SettingView) -> Result<(Vec<Setting>, String), CliError> {
    // 1. 检查世界已初始化
    let conn = open_existing_at(target)?;
    let schema_version = read_schema_version(&conn)?;

    // 2. 查询所有 settings（按 name 排序，与 list_settings 一致）
    let all = list_all_settings(&conn)?;

    // 3. 按 view 组合子集
    let settings = select_for_view(&all, view);
    Ok((settings, schema_version))
}

/// 按 view 名组合 settings 子集。
///
/// 映射策略（design D3，半稳定，非契约——契约只冻结 view 名与"组合后输出
/// settings 数组 + sourceRefs"的结构，不钉死 key 名映射）：
/// - `full`：全部 settings。
/// - `intro`/`four-poles`/`world`/`style`：按 name 关键字归类，含该类关键字的 setting 入视图。
/// - `summary`：兜底视图——优先返回归类为 intro 的项；若无则返回全部（保证非空兜底）。
/// - `llm`：预算内固定组合 = intro∪four-poles∪world∪style 的并集（Spec 写前默认入口所需最小设定子集）。
///
/// 任何 setting 若都不匹配，归入 `full` 与 `summary` 兜底，不归入细分 view。
fn select_for_view(all: &[Setting], view: SettingView) -> Vec<Setting> {
    match view {
        SettingView::Full => all.to_vec(),
        SettingView::Summary => {
            let intros: Vec<Setting> = all
                .iter()
                .filter(|s| matches_class(s.name.as_str(), SettingView::Intro))
                .cloned()
                .collect();
            if intros.is_empty() {
                // 兜底：无 intro 类时返回全部，保证 Spec 不拿到空注入
                all.to_vec()
            } else {
                intros
            }
        }
        SettingView::Llm => {
            // 预算内固定组合：intro∪four-poles∪world∪style
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut out: Vec<Setting> = Vec::new();
            for s in all {
                let classes = classify(s.name.as_str());
                let in_llm = classes.iter().any(|c| matches!(
                    c,
                    SettingView::Intro | SettingView::FourPoles | SettingView::World | SettingView::Style
                ));
                if in_llm && seen.insert(s.name.clone()) {
                    out.push(s.clone());
                }
            }
            out
        }
        // 细分 view：按 name 关键字归类
        SettingView::Intro | SettingView::FourPoles | SettingView::World | SettingView::Style => {
            all.iter()
                .filter(|s| matches_class(s.name.as_str(), view))
                .cloned()
                .collect()
        }
    }
}

/// 判定单个 setting name 是否属于指定 view 类。
fn matches_class(name: &str, view: SettingView) -> bool {
    classify(name).iter().any(|c| *c == view)
}

/// 按 name 关键字归类一个 setting 可能所属的全部 view 类（一个 name 可多类）。
fn classify(name: &str) -> Vec<SettingView> {
    let lower = name.to_lowercase();
    let mut classes = Vec::new();
    if lower.contains("intro")
        || lower.contains("summary")
        || lower.contains("简介")
        || lower.contains("摘要")
    {
        classes.push(SettingView::Intro);
    }
    if lower.contains("pole")
        || lower.contains("四极")
        || lower.contains("边界")
        || lower.contains("limit")
        || lower.contains("constraint")
    {
        classes.push(SettingView::FourPoles);
    }
    if lower.contains("style")
        || lower.contains("风格")
        || lower.contains("文风")
        || lower.contains("叙事")
        || lower.contains("tone")
    {
        classes.push(SettingView::Style);
    }
    // world 类：含 world/世界/设定/setting 但不含 style 关键字（避免 style 类被误归 world）
    if (lower.contains("world")
        || lower.contains("世界")
        || lower.contains("设定")
        || lower.contains("setting"))
        && !(lower.contains("style")
            || lower.contains("风格")
            || lower.contains("文风")
            || lower.contains("叙事")
            || lower.contains("tone"))
    {
        classes.push(SettingView::World);
    }
    classes
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn setting_service_blocks_when_world_not_initialized() {
        let dir = tempdir().unwrap();

        let err = set_setting(dir.path(), "world-style", "奇幻史诗").unwrap_err();
        assert!(matches!(err, CliError::PreconditionBlocked { .. }));
        assert_eq!(i32::from(err.exit_code()), 3);
    }
}
