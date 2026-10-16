// storage/constraints.rs
// s01-project-bootstrap：constraints 表的数据访问层。
//
// 职责：封装 constraints 表的 SQL 读写，支持插入（检查复合主键冲突）、查询、列表（支持 scope 前缀与 severity 过滤）。
// 不做业务判断，业务规则由 service 层负责。

use rusqlite::{params, Connection};

use crate::domain::{Constraint, ConstraintSeverity, ConstraintType};
use crate::errors::CliError;

/// 插入新 constraint。如果复合主键 (constraint_type, scope, name) 冲突则返回错误。
pub fn insert_constraint(
    conn: &Connection,
    constraint_type: ConstraintType,
    scope: &str,
    name: &str,
    content: &str,
    severity: ConstraintSeverity,
    created_at: &str,
) -> Result<(), CliError> {
    let result = conn.execute(
        "INSERT INTO constraints (constraint_type, scope, name, content, severity, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            constraint_type.to_string(),
            scope,
            name,
            content,
            severity.to_string(),
            created_at
        ],
    );

    match result {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Err(CliError::resource_conflict(format!(
                "约束已存在: type='{}', scope='{}', name='{}'",
                constraint_type, scope, name
            )))
        }
        Err(e) => Err(CliError::storage(format!("插入 constraint 失败: {}", e))),
    }
}

/// 按一组精确 scope 值查询 constraints（受控 `constraint render --scope` 范围查询用）。
///
/// 与 `list_constraints` 的 scope 前缀过滤（LIKE）不同，本函数按 scope 值**精确匹配**
/// 集合中的任意一个。受控 scope 范围解析约定（setting-constraint-render design D2）：
/// - `world` scope 值匹配 `scope = "world"`
/// - 卷级匹配 `scope = "volume:<整数id>"`
/// - 章级匹配 `scope = "chapter:<chapter-id字符串>"`
///
/// scope 值列表可为空（返回空 Vec）。结果按 constraint_type, scope, name 排序，与
/// `list_constraints` 一致。
pub fn list_constraints_by_scopes(
    conn: &Connection,
    scopes: &[String],
) -> Result<Vec<Constraint>, CliError> {
    if scopes.is_empty() {
        return Ok(Vec::new());
    }

    // 构建 IN (?, ?, ...) 占位符
    let placeholders = (0..scopes.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT constraint_type, scope, name, content, severity, created_at \
         FROM constraints WHERE scope IN ({}) ORDER BY constraint_type, scope, name",
        placeholders
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CliError::storage(format!("准备 constraints 范围查询失败: {}", e)))?;

    let params: Vec<&dyn rusqlite::ToSql> = scopes
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    let constraints = stmt
        .query_map(params.as_slice(), |row| {
            let ct_str: String = row.get(0)?;
            let constraint_type = ct_str
                .parse::<ConstraintType>()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

            let sev_str: String = row.get(4)?;
            let severity = sev_str
                .parse::<ConstraintSeverity>()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

            Ok(Constraint::new(
                constraint_type,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                severity,
                row.get(5)?,
            ))
        })
        .map_err(|e| CliError::storage(format!("查询 constraints 失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("读取 constraints 行失败: {}", e)))?;

    Ok(constraints)
}


pub fn get_constraint(
    conn: &Connection,
    constraint_type: ConstraintType,
    scope: &str,
    name: &str,
) -> Result<Option<Constraint>, CliError> {
    let result = conn.query_row(
        "SELECT constraint_type, scope, name, content, severity, created_at
         FROM constraints
         WHERE constraint_type = ?1 AND scope = ?2 AND name = ?3",
        params![constraint_type.to_string(), scope, name],
        |row| {
            let ct_str: String = row.get(0)?;
            let constraint_type = ct_str
                .parse::<ConstraintType>()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

            let sev_str: String = row.get(4)?;
            let severity = sev_str
                .parse::<ConstraintSeverity>()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

            Ok(Constraint::new(
                constraint_type,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                severity,
                row.get(5)?,
            ))
        },
    );

    match result {
        Ok(constraint) => Ok(Some(constraint)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(CliError::storage(format!("查询 constraint 失败: {}", e))),
    }
}

/// 列出 constraints，支持按 scope 前缀和 severity 过滤。
///
/// - scope_filter: 如果提供，则只返回 scope 以此前缀开头的约束（如 "volume:" 匹配 "volume:v1", "volume:v2"）
/// - severity_filter: 如果提供，则只返回匹配此 severity 的约束
pub fn list_constraints(
    conn: &Connection,
    scope_filter: Option<&str>,
    severity_filter: Option<ConstraintSeverity>,
) -> Result<Vec<Constraint>, CliError> {
    let mut sql = String::from(
        "SELECT constraint_type, scope, name, content, severity, created_at FROM constraints WHERE 1=1"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(scope_prefix) = scope_filter {
        sql.push_str(" AND scope LIKE ?");
        params.push(Box::new(format!("{}%", scope_prefix)));
    }

    if let Some(severity) = severity_filter {
        sql.push_str(" AND severity = ?");
        params.push(Box::new(severity.to_string()));
    }

    sql.push_str(" ORDER BY constraint_type, scope, name");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CliError::storage(format!("准备 constraints 查询失败: {}", e)))?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let constraints = stmt
        .query_map(param_refs.as_slice(), |row| {
            let ct_str: String = row.get(0)?;
            let constraint_type = ct_str
                .parse::<ConstraintType>()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

            let sev_str: String = row.get(4)?;
            let severity = sev_str
                .parse::<ConstraintSeverity>()
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

            Ok(Constraint::new(
                constraint_type,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                severity,
                row.get(5)?,
            ))
        })
        .map_err(|e| CliError::storage(format!("查询 constraints 失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("读取 constraints 行失败: {}", e)))?;

    Ok(constraints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
        conn
    }

    #[test]
    fn insert_duplicate_constraint_returns_conflict() {
        let conn = setup_conn();

        insert_constraint(
            &conn,
            ConstraintType::Plot,
            "world",
            "时间线一致性",
            "规则1",
            ConstraintSeverity::Hard,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();

        let err = insert_constraint(
            &conn,
            ConstraintType::Plot,
            "world",
            "时间线一致性",
            "规则2",
            ConstraintSeverity::Warning,
            "2026-06-29T01:00:00Z",
        )
        .unwrap_err();

        assert!(matches!(err, CliError::ResourceConflict { .. }));
    }

    #[test]
    fn list_constraints_filters_by_scope_prefix() {
        let conn = setup_conn();

        insert_constraint(
            &conn,
            ConstraintType::Plot,
            "world",
            "规则1",
            "内容1",
            ConstraintSeverity::Hard,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();
        insert_constraint(
            &conn,
            ConstraintType::Plot,
            "volume:v1",
            "规则2",
            "内容2",
            ConstraintSeverity::Warning,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();
        insert_constraint(
            &conn,
            ConstraintType::Character,
            "volume:v2",
            "规则3",
            "内容3",
            ConstraintSeverity::Hard,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();

        let constraints = list_constraints(&conn, Some("volume:"), None).unwrap();
        assert_eq!(constraints.len(), 2);
        assert!(constraints.iter().all(|c| c.scope.starts_with("volume:")));
    }

    #[test]
    fn list_constraints_filters_by_severity() {
        let conn = setup_conn();

        insert_constraint(
            &conn,
            ConstraintType::Plot,
            "world",
            "规则1",
            "内容1",
            ConstraintSeverity::Hard,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();
        insert_constraint(
            &conn,
            ConstraintType::Plot,
            "world",
            "规则2",
            "内容2",
            ConstraintSeverity::Warning,
            "2026-06-29T00:00:00Z",
        )
        .unwrap();

        let constraints = list_constraints(&conn, None, Some(ConstraintSeverity::Hard)).unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0].severity, ConstraintSeverity::Hard);
    }
}
