// emergence_service 模块：涌现生命周期管理业务逻辑。
//
// 职责（CLAUDE.md §6 / 不变量 12）：service 持有 DB-open，command 层仅传 `target: &Path`。
// 公开函数接收 `target: &Path`，内部首行 `open_existing_at(target)?`，再把 `&Connection` 传给 repository / validator / guard。

use crate::domain::emergence::{Decision, Emergence, EmergenceStatus};
use crate::errors::CliError;
use crate::repositories::emergence_repository;
use crate::storage::connection::open_existing_at;
use crate::validators::emergence_state_validator;
use std::path::Path;

/// 创建涌现。
pub fn create(
    target: &Path,
    volume_id: i64,
    title: &str,
    description: &str,
    chapter_id: Option<&str>,
) -> Result<Emergence, CliError> {
    let conn = open_existing_at(target)?;

    // 验证卷是否存在
    let volume_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM volumes WHERE id = ?1)",
            [volume_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查卷是否存在失败：{}", e)))?;

    if !volume_exists {
        return Err(CliError::volume_not_found(format!(
            "卷不存在：{}",
            volume_id
        )));
    }

    // 如果指定了章节，验证章节是否存在
    if let Some(ch_id) = chapter_id {
        let chapter_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)",
                [ch_id],
                |row| row.get(0),
            )
            .map_err(|e| CliError::storage(format!("检查章节是否存在失败：{}", e)))?;

        if !chapter_exists {
            return Err(CliError::chapter_not_found(
                format!("章节不存在：{}", ch_id),
                None,
            ));
        }
    }

    // 调用 repository 创建涌现（注意参数顺序：chapter_id 在 title 之前）
    emergence_repository::create(&conn, volume_id, chapter_id, title, description)
}

/// 评分涌现（pending → rated）。
pub fn rate(
    target: &Path,
    emergence_id: &str,
    rating: i64,
    reason: &str,
) -> Result<Emergence, CliError> {
    let conn = open_existing_at(target)?;

    // s07 归档冻结门禁：涌现所属卷 archived 时拒绝修改。
    crate::validators::guards::volume_freeze_guard::ensure_emergence_volume_not_archived(
        &conn, emergence_id,
    )?;

    // 获取涌现当前状态
    let emergence = emergence_repository::get(&conn, emergence_id)?;

    // 验证评分范围
    emergence_state_validator::validate_rating(rating)?;

    // 验证未归档
    emergence_state_validator::validate_not_archived(emergence.status)?;

    // 验证状态转换
    emergence_state_validator::validate_state_transition(
        emergence.status,
        EmergenceStatus::Rated,
    )?;

    // 调用 repository 执行评分
    emergence_repository::rate(&conn, emergence_id, rating, reason)?;

    // 重新查询返回更新后的涌现
    emergence_repository::get(&conn, emergence_id)
}

/// 决策涌现（rated → decided）。
pub fn decide(
    target: &Path,
    emergence_id: &str,
    decision: &str,
    reason: &str,
) -> Result<Emergence, CliError> {
    let conn = open_existing_at(target)?;

    // s07 归档冻结门禁：涌现所属卷 archived 时拒绝修改。
    crate::validators::guards::volume_freeze_guard::ensure_emergence_volume_not_archived(
        &conn, emergence_id,
    )?;

    // 解析决策
    let decision_enum = Decision::from_str(decision).ok_or_else(|| {
        CliError::usage(format!(
            "决策无效：{}，必须为 accept、defer 或 reject",
            decision
        ))
    })?;

    // 获取涌现当前状态
    let emergence = emergence_repository::get(&conn, emergence_id)?;

    // 验证未归档
    emergence_state_validator::validate_not_archived(emergence.status)?;

    // 验证状态转换
    emergence_state_validator::validate_state_transition(
        emergence.status,
        EmergenceStatus::Decided,
    )?;

    // 调用 repository 执行决策（传入 Decision 枚举）
    emergence_repository::decide(&conn, emergence_id, decision_enum, reason)?;

    // 重新查询返回更新后的涌现
    emergence_repository::get(&conn, emergence_id)
}

/// 执行涌现（decided → executed）。
pub fn execute(
    target: &Path,
    emergence_id: &str,
    target_chapter: &str,
    note: Option<&str>,
) -> Result<Emergence, CliError> {
    let conn = open_existing_at(target)?;

    // s07 归档冻结门禁：涌现所属卷 archived 时拒绝修改。
    crate::validators::guards::volume_freeze_guard::ensure_emergence_volume_not_archived(
        &conn, emergence_id,
    )?;

    // 获取涌现当前状态
    let emergence = emergence_repository::get(&conn, emergence_id)?;

    // 验证未归档
    emergence_state_validator::validate_not_archived(emergence.status)?;

    // 验证状态转换
    emergence_state_validator::validate_state_transition(
        emergence.status,
        EmergenceStatus::Executed,
    )?;

    // 验证决策允许执行（仅 accept 可执行）
    emergence_state_validator::validate_decision_allows_execution(emergence.decision)?;

    // 验证执行目标章节是否存在
    let chapter_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)",
            [target_chapter],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查章节是否存在失败：{}", e)))?;

    if !chapter_exists {
        return Err(CliError::chapter_not_found(
            format!("执行目标章节不存在：{}", target_chapter),
            None,
        ));
    }

    // 调用 repository 执行
    emergence_repository::execute(&conn, emergence_id, target_chapter, note)?;

    // 重新查询返回更新后的涌现
    emergence_repository::get(&conn, emergence_id)
}

/// 归档涌现（executed → archived）。
pub fn archive(target: &Path, emergence_id: &str) -> Result<Emergence, CliError> {
    let conn = open_existing_at(target)?;

    // 获取涌现当前状态
    let emergence = emergence_repository::get(&conn, emergence_id)?;

    // 验证状态转换
    emergence_state_validator::validate_state_transition(
        emergence.status,
        EmergenceStatus::Archived,
    )?;

    // 调用 repository 归档
    emergence_repository::archive(&conn, emergence_id)?;

    // 重新查询返回更新后的涌现
    emergence_repository::get(&conn, emergence_id)
}

/// 列举卷内涌现（支持状态过滤）。
pub fn list(
    target: &Path,
    volume_id: i64,
    status_filter: Option<&str>,
) -> Result<Vec<Emergence>, CliError> {
    let conn = open_existing_at(target)?;

    // 验证卷是否存在
    let volume_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM volumes WHERE id = ?1)",
            [volume_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查卷是否存在失败：{}", e)))?;

    if !volume_exists {
        return Err(CliError::volume_not_found(format!(
            "卷不存在：{}",
            volume_id
        )));
    }

    // 如果指定了状态过滤，验证状态值并转换为枚举
    let status_enum = if let Some(status) = status_filter {
        let parsed = EmergenceStatus::from_str(status).ok_or_else(|| {
            CliError::usage(format!(
                "状态无效：{}，必须为 pending、rated、decided、executed 或 archived",
                status
            ))
        })?;
        Some(parsed)
    } else {
        None
    };

    emergence_repository::list(&conn, volume_id, status_enum)
}

/// 获取单个涌现详情。
pub fn get(target: &Path, emergence_id: &str) -> Result<Emergence, CliError> {
    let conn = open_existing_at(target)?;
    emergence_repository::get(&conn, emergence_id)
}
