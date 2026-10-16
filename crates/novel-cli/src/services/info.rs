// info 用例：查询世界基本信息。
//
// 职责（CLAUDE.md §6、§10、design D5）：读取项目元数据与 schemaVersion，组装 WorldInfo；
// `.novel/` 不存在返回 PreconditionBlocked（退出码 3）。

use std::path::PathBuf;

use crate::domain::info::WorldInfo;
use crate::errors::CliError;
use crate::storage::connection::open;
use crate::storage::metadata::read_world_info;

/// 解析 `--path`：默认当前工作目录；目录必须存在否则退出码 2。
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

/// 读取世界概况。`.novel/` 不存在 → PreconditionBlocked(3)。
pub fn read_info(target: &std::path::Path) -> Result<WorldInfo, CliError> {
    let world_root = target.join(".novel");
    if !world_root.is_dir() {
        return Err(CliError::world_not_initialized(format!(
            "当前目录未初始化为 novel 世界（未找到 {}）。请先运行 `novel init`。",
            world_root.display()
        )));
    }
    let db_path = world_root.join("novel.db");
    if !db_path.is_file() {
        return Err(CliError::world_not_initialized(format!(
            "世界目录存在但缺少 novel.db（{}）。数据库可能未正确初始化。",
            db_path.display()
        )));
    }

    let conn = open(&db_path)?;
    let info = read_world_info(&conn)?;
    Ok(info)
}
