// mainplot_service 模块：主线节点业务逻辑。
//
// 职责（CLAUDE.md §6 / 不变量 12）：service 持有 DB-open，command 层仅传 `target: &Path`。
// 公开函数接收 `target: &Path`，内部首行 `open_existing_at(target)?`，再把 `&Connection` 传给 repository / validator。

use crate::domain::mainplot_node::MainplotNode;
use crate::errors::CliError;
use crate::repositories::mainplot_repository;
use crate::storage::connection::open_existing_at;
use crate::validators::mainplot_validator;
use std::path::Path;

/// 创建主线节点。
pub fn create_node(
    target: &Path,
    volume_id: i64,
    title: &str,
    description: Option<&str>,
    target_chapters: Option<Vec<String>>,
) -> Result<MainplotNode, CliError> {
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

    // 验证目标章节引用完整性
    if let Some(ref chapters) = target_chapters {
        mainplot_validator::validate_target_chapters(&conn, chapters)?;
    }

    // 将 Vec<String> 转换为 JSON 字符串（design D6：expect → CliError::storage）
    let target_chapters_json = target_chapters
        .as_ref()
        .map(|chapters| serde_json::to_string(chapters))
        .transpose()
        .map_err(|e| CliError::storage(format!("序列化章节 ID 列表失败: {e}")))?;

    // 调用 repository 创建节点
    mainplot_repository::create_node(
        &conn,
        volume_id,
        title,
        description,
        target_chapters_json.as_deref(),
    )
}

/// 列举卷内所有主线节点。
pub fn list_nodes(target: &Path, volume_id: i64) -> Result<Vec<MainplotNode>, CliError> {
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

    mainplot_repository::list_nodes(&conn, volume_id)
}

/// 获取单个主线节点详情。
pub fn get_node(target: &Path, node_id: &str) -> Result<MainplotNode, CliError> {
    let conn = open_existing_at(target)?;
    mainplot_repository::get_node(&conn, node_id)
}

/// 更新主线节点的达成状态。
pub fn update_node_achieved(
    target: &Path,
    node_id: &str,
    achieved: i64,
) -> Result<(), CliError> {
    let conn = open_existing_at(target)?;

    // 验证 achieved 参数合法性
    if achieved != 0 && achieved != 1 {
        return Err(CliError::usage(format!(
            "achieved 参数无效：{}，必须为 0 或 1",
            achieved
        )));
    }

    mainplot_repository::update_node_achieved(&conn, node_id, achieved)
}
