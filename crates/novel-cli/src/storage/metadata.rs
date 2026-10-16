// 项目级元数据读写（项目名、创建时间、dbPath、schemaVersion）。
//
// 职责（CLAUDE.md §7）：SQLite 是世界状态权威索引，记录项目级元数据。
// project_meta 表单行约束（id=1）。读写均通过 SQL，不做业务判断（属 service 层）。

use rusqlite::{params, Connection};

use crate::domain::info::{Counts, WorldInfo};
use crate::errors::CliError;
use crate::storage::connection::CURRENT_SCHEMA_VERSION;

/// 写入初始项目元数据（单行）。由 init service 在 migration 后调用。
pub fn insert_project_meta(
    conn: &Connection,
    project_name: &str,
    db_path: &str,
    created_at: &str,
) -> Result<(), CliError> {
    conn.execute(
        "INSERT INTO project_meta (id, project_name, db_path, created_at, schema_version) \
         VALUES (1, ?1, ?2, ?3, ?4)",
        params![project_name, db_path, created_at, CURRENT_SCHEMA_VERSION.to_string()],
    )
    .map_err(|e| CliError::storage(format!("写入 project_meta 失败: {e}")))?;
    Ok(())
}

/// 读取项目元数据，组装为 WorldInfo。counts 反映各实体表的真实行数。
pub fn read_world_info(conn: &Connection) -> Result<WorldInfo, CliError> {
    let (project_name, db_path, created_at): (String, String, String) = conn
        .query_row(
            "SELECT project_name, db_path, created_at FROM project_meta WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| CliError::storage(format!("读取 project_meta 失败: {e}")))?;

    let schema_version = crate::storage::connection::read_schema_version(conn)?;

    // 各实体表真实行数。表自 V002 起随迁移建立，初始化后的世界均存在这些表。
    let count = |table: &str| -> Result<u64, CliError> {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        conn.query_row(&sql, [], |r| r.get::<_, i64>(0))
            .map(|n| n.max(0) as u64)
            .map_err(|e| CliError::storage(format!("读取 {table} 计数失败: {e}")))
    };

    Ok(WorldInfo {
        schema_version,
        project_name,
        db_path,
        created_at,
        counts: Counts {
            chapters: count("chapters")?,
            volumes: count("volumes")?,
            settings: count("settings")?,
            constraints: count("constraints")?,
            mainplot_nodes: count("mainplot_nodes")?,
        },
    })
}
