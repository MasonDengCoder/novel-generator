// init 用例：创建新世界。
//
// 职责（CLAUDE.md §6、§7、§10、design D4、OQ2、OQ3）：
// - 检测 `.novel/` 已存在则返回 ResourceConflict（退出码 4），不覆盖。
// - 创建目录结构 + SQLite，事务内写项目元数据与 schemaVersion。
// - 基础校验 name 非空/无路径分隔符（委托 validators）。
// - 目标路径若疑似 novel-generator 仓库根则警告（不硬拦，guard 已守 Write 侧）。

use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::errors::CliError;
use crate::storage::connection::{open, run_migrations};
use crate::storage::metadata::insert_project_meta;
use crate::validators;

/// init 用例的输出对象（渲染层消费）。
pub struct InitOutput {
    pub project_name: String,
    /// 世界根目录的绝对路径。
    pub world_root: PathBuf,
    pub db_path: String,
    pub created_at: String,
    pub schema_version: String,
    /// 是否在目标路径检测到疑似 novel-generator 仓库根（仅警告，未阻断）。
    pub repo_warning: bool,
}

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

/// 目标路径是否疑似 novel-generator 仓库根（路径片段含 novel-generator）。
fn looks_like_repo_root(target: &Path) -> bool {
    target
        .components()
        .any(|c| c.as_os_str().to_string_lossy() == "novel-generator")
}

/// 在 `<dir>/.novel/` 下创建世界结构、SQLite 与元数据。`.novel/` 已存在则拒绝。
pub fn create_world(name: &str, target: &Path) -> Result<InitOutput, CliError> {
    // 1. 基础校验 name（非空、无路径分隔符）。
    validators::validate_project_name(name)
        .map_err(CliError::usage)?;

    let world_root = target.join(".novel");

    // 2. 冲突检测：已存在则拒绝，不覆盖。
    if world_root.exists() {
        return Err(CliError::resource_conflict(format!(
            "目标目录下已存在 .novel/（{}），拒绝覆盖。",
            world_root.display()
        )));
    }

    let repo_warning = looks_like_repo_root(target);
    let db_rel = ".novel/novel.db";
    let db_path = world_root.join("novel.db");

    // 3. 创建目录结构（CLAUDE.md §7）。
    let dirs = [
        world_root.join("content/chapters"),
        world_root.join("content/residues"),
        world_root.join("content/reports"),
        world_root.join("tmp"),
        world_root.join("locks"),
    ];
    for d in &dirs {
        std::fs::create_dir_all(d)
            .map_err(|e| CliError::storage(format!("创建目录 {} 失败: {e}", d.display())))?;
    }

    let created_at = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间失败: {e}")))?
        .to_string();

    // 4. 打开 SQLite（Connection::open 会创建文件）、运行 migration、写元数据。
    let conn = open(&db_path)?;
    run_migrations(&conn, &created_at)?;
    insert_project_meta(&conn, name, db_rel, &created_at)?;

    let schema_version =
        crate::storage::connection::read_schema_version(&conn)?;

    // 连接 drop 时关闭；SQLite 文件已落盘。
    drop(conn);

    Ok(InitOutput {
        project_name: name.to_string(),
        world_root,
        db_path: db_rel.to_string(),
        created_at,
        schema_version,
        repo_warning,
    })
}
