// mainplot_validator 模块：主线节点引用完整性校验。

use crate::errors::CliError;
use rusqlite::Connection;

/// 校验 target_chapters 引用完整性：检查章节 ID 是否存在。
pub fn validate_target_chapters(
    conn: &Connection,
    target_chapters: &[String],
) -> Result<(), CliError> {
    if target_chapters.is_empty() {
        return Ok(());
    }

    for chapter_id in target_chapters {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM chapters WHERE id = ?1)",
                [chapter_id],
                |row| row.get(0),
            )
            .map_err(|e| CliError::storage(format!("检查章节引用失败：{}", e)))?;

        if !exists {
            return Err(CliError::not_found(
                format!("目标章节不存在：{}", chapter_id),
                None,
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // 创建必要的表
        conn.execute_batch(
            r#"
            CREATE TABLE chapters (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL
            );

            INSERT INTO chapters (id, title) VALUES ('ch-001', 'Chapter 1');
            INSERT INTO chapters (id, title) VALUES ('ch-002', 'Chapter 2');
            "#,
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_validate_target_chapters_all_exist() {
        let conn = setup_test_db();
        let chapters = vec!["ch-001".to_string(), "ch-002".to_string()];
        assert!(validate_target_chapters(&conn, &chapters).is_ok());
    }

    #[test]
    fn test_validate_target_chapters_empty() {
        let conn = setup_test_db();
        let chapters = vec![];
        assert!(validate_target_chapters(&conn, &chapters).is_ok());
    }

    #[test]
    fn test_validate_target_chapters_not_found() {
        let conn = setup_test_db();
        let chapters = vec!["ch-001".to_string(), "ch-999".to_string()];
        let result = validate_target_chapters(&conn, &chapters);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ch-999"));
    }

    #[test]
    fn test_validate_target_chapters_single_not_found() {
        let conn = setup_test_db();
        let chapters = vec!["ch-999".to_string()];
        let result = validate_target_chapters(&conn, &chapters);
        assert!(result.is_err());
    }
}
