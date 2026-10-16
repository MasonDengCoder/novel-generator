// thread_service 模块：线索业务逻辑。
//
// 职责（CLAUDE.md §6 / 不变量 12）：service 持有 DB-open，command 层仅传 `target: &Path`。
// 公开函数接收 `target: &Path`，内部首行 `open_existing_at(target)?`，再把 `&Connection` 传给 repository / validator。

use crate::domain::thread::{BindingType, Thread, ThreadBinding, ThreadStatus};
use crate::errors::CliError;
use crate::repositories::thread_repository;
use crate::storage::connection::open_existing_at;
use crate::validators::thread_validator;
use std::path::Path;

/// 创建线索。
pub fn create_thread(
    target: &Path,
    volume_id: i64,
    name: &str,
    description: Option<&str>,
) -> Result<Thread, CliError> {
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

    // 验证线索名称唯一性
    thread_validator::validate_thread_name_uniqueness(&conn, volume_id, name)?;

    // 调用 repository 创建线索
    thread_repository::create_thread(&conn, volume_id, name, description)
}

/// 绑定线索到章节。
pub fn bind_chapter(
    target: &Path,
    thread_id: &str,
    chapter_id: &str,
    binding_type: Option<&str>,
    note: Option<&str>,
) -> Result<(), CliError> {
    let conn = open_existing_at(target)?;

    // 验证绑定类型
    let binding_type_value = binding_type.unwrap_or("mention");
    thread_validator::validate_binding_type(binding_type_value)?;

    // 转换为枚举（design D6：expect → CliError::storage，validator 已校验属不变量破裂兜底）
    let binding_type_enum = BindingType::from_str(binding_type_value).ok_or_else(|| {
        CliError::storage(
            "绑定类型转换失败：validate_binding_type 已验证但枚举转换失败".to_string(),
        )
    })?;

    // 验证线索是否存在
    let thread_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?1)",
            [thread_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查线索是否存在失败：{}", e)))?;

    if !thread_exists {
        return Err(CliError::thread_not_found(format!(
            "线索不存在：{}",
            thread_id
        )));
    }

    // 验证章节是否存在
    let chapter_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)",
            [chapter_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查章节是否存在失败：{}", e)))?;

    if !chapter_exists {
        return Err(CliError::chapter_not_found(
            format!("章节不存在：{}", chapter_id),
            None,
        ));
    }

    // 检查绑定是否已存在
    let binding_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM thread_bindings WHERE thread_id = ?1 AND chapter_id = ?2)",
            rusqlite::params![thread_id, chapter_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查绑定是否存在失败：{}", e)))?;

    if binding_exists {
        return Err(CliError::binding_already_exists(format!(
            "线索 {} 已绑定到章节 {}",
            thread_id, chapter_id
        )));
    }

    // 调用 repository 创建绑定
    thread_repository::bind_chapter(&conn, thread_id, chapter_id, binding_type_enum, note)
}

/// 列举卷内所有线索。
pub fn list_threads(target: &Path, volume_id: i64) -> Result<Vec<Thread>, CliError> {
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

    thread_repository::list_threads(&conn, volume_id)
}

/// 查询章节的所有线索绑定。
pub fn query_by_chapter(
    target: &Path,
    chapter_id: &str,
) -> Result<Vec<ThreadBinding>, CliError> {
    let conn = open_existing_at(target)?;

    // 验证章节是否存在
    let chapter_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)",
            [chapter_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查章节是否存在失败：{}", e)))?;

    if !chapter_exists {
        return Err(CliError::chapter_not_found(
            format!("章节不存在：{}", chapter_id),
            None,
        ));
    }

    thread_repository::query_by_chapter(&conn, chapter_id)
}

/// 更新线索状态。
pub fn update_status(
    target: &Path,
    thread_id: &str,
    status: &str,
) -> Result<(), CliError> {
    let conn = open_existing_at(target)?;

    // 验证状态值并转换为枚举
    let status_enum = ThreadStatus::from_str(status).ok_or_else(|| {
        CliError::usage(format!(
            "线索状态无效：{}，必须为 active、paused 或 resolved",
            status
        ))
    })?;

    // 验证线索是否存在
    let thread_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM threads WHERE id = ?1)",
            [thread_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查线索是否存在失败：{}", e)))?;

    if !thread_exists {
        return Err(CliError::thread_not_found(format!(
            "线索不存在：{}",
            thread_id
        )));
    }

    thread_repository::update_status(&conn, thread_id, status_enum)
}
