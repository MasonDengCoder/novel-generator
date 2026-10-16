// verdict_service 模块：主线落地裁决写入用例（s04-state-machine-and-verdicts）。
//
// 职责（CLAUDE.md §6、§7、§10）：编排写锁/事务/外部修改检测/裁决 UPSERT，
// 组装错误。approved 章节拒绝写入（CLAUDE.md §7）。hard 阻断非零退出码，不静默降级。

use crate::errors::CliError;
use crate::repositories::verdict_repository;
use crate::storage::connection;
use crate::storage::file_ops::ProjectWriteLock;
use crate::validators::chapter_validator;
use crate::validators::guards::state_transition_guard::VERDICT_TYPE_MAINPLOT_LANDING;
use std::path::Path;

/// 裁决写入结果（用于 renderer）。
#[derive(Debug)]
pub struct VerdictWritten {
    pub chapter_id: String,
    pub node_id: String,
    pub achieved: bool,
    pub content_hash: String,
    pub created_at: String,
}

/// 写入主线落地裁决。
///
/// 调用链（design D5）：
///   写锁 → open_existing_at → 读取 ChapterRow（chapter_not_found）
///   → approved 章节拒绝（precondition_blocked）
///   → 外部修改检测（content_modified_externally，退出码 21）
///   → 事务 upsert_verdict 绑定 current_hash → commit
pub fn set_mainplot_landing(
    target_dir: &Path,
    chapter_id: &str,
    node_id: &str,
    achieved: bool,
) -> Result<VerdictWritten, CliError> {
    let novel_dir = target_dir.join(".novel");

    // 1. 获取写锁（复用 s02 ProjectWriteLock）。
    let _lock = ProjectWriteLock::acquire(&novel_dir)?;

    // 2. 打开数据库（自动检查 world_not_initialized）。
    let mut conn = connection::open_existing_at(target_dir)?;

    // 2.5 s07 归档冻结门禁：章节所属卷 archived 时拒绝写入裁决。
    crate::validators::guards::volume_freeze_guard::ensure_chapter_volume_not_archived(
        &conn, chapter_id,
    )?;

    // 3. 读取章节元数据（chapter_not_found 检测）。
    let chapter = crate::storage::chapters::get_chapter(&conn, chapter_id)?;

    // 3.5 校验主线节点存在（防止悬空裁决；对比 thread bind 的 thread_not_found 校验）。
    //     节点不存在返回 mainplot_node_not_found（退出码 43），而非静默建出悬空裁决。
    let node_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM mainplot_nodes WHERE id = ?1)",
            [node_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查主线节点是否存在失败：{}", e)))?;
    if !node_exists {
        return Err(CliError::mainplot_node_not_found(format!(
            "主线节点 '{}' 不存在，无法记录裁决（请先用 `novel mainplot node create` 创建该节点）",
            node_id
        )));
    }

    // 4. approved 章节拒绝写入（CLAUDE.md §7 approved 后不得修改裁决）。
    if chapter.status == "approved" {
        return Err(CliError::precondition_blocked(format!(
            "章节 {} 已 approved，裁决不可修改（终态冻结）。",
            chapter_id
        )));
    }

    // 5. 外部修改检测：重算正文文件 hash 与 chapters.content_hash 比较。
    let content_path = novel_dir.join(&chapter.content_path);
    let current_hash = match std::fs::read_to_string(&content_path) {
        Ok(content) => chapter_validator::calculate_content_hash(&content),
        Err(_) => {
            // 正文文件缺失或不可读：无法验证正文未被外部修改，视为外部修改（退出码 21）。
            return Err(CliError::content_modified_externally(format!(
                "章节 {} 的正文文件无法读取，无法验证完整性：{}",
                chapter_id,
                content_path.display()
            )));
        }
    };
    if current_hash != chapter.content_hash {
        return Err(CliError::content_modified_externally(format!(
            "章节 {} 的正文已被外部修改，请运行 `novel chapter reimport {}` 重新导入。",
            chapter_id, chapter_id
        )));
    }

    // 6. 生成时间戳。
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 7. 事务内 UPSERT 裁决，绑定 current_hash。
    let tx = conn
        .transaction()
        .map_err(|e| CliError::storage(format!("开启事务失败：{}", e)))?;

    verdict_repository::upsert_verdict(
        &tx,
        chapter_id,
        VERDICT_TYPE_MAINPLOT_LANDING,
        node_id,
        achieved,
        &current_hash,
        &timestamp,
    )?;

    tx.commit()
        .map_err(|e| CliError::storage(format!("提交事务失败：{}", e)))?;

    Ok(VerdictWritten {
        chapter_id: chapter_id.to_string(),
        node_id: node_id.to_string(),
        achieved,
        content_hash: current_hash,
        created_at: timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::chapter_repository;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().to_path_buf();
        let novel_dir = project_dir.join(".novel");
        fs::create_dir_all(&novel_dir).unwrap();

        let db_path = novel_dir.join("novel.db");
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
            run_migrations(&conn, "2026-07-01T00:00:00Z").unwrap();
            // 缺陷修复后裁决须引用真实存在的节点：预置卷 1 与主线节点 mp-1-001。
            conn.execute(
                "INSERT INTO volumes (id, name, volume_order, status, created_at, updated_at) VALUES (1,'测试卷',1,'active','2026-07-01T00:00:00Z','2026-07-01T00:00:00Z')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) VALUES ('mp-1-001',1,1,'节点A',0,'2026-07-01T00:00:00Z','2026-07-01T00:00:00Z')",
                [],
            )
            .unwrap();
        }

        (dir, project_dir)
    }

    fn create_and_write_chapter(project_dir: &std::path::PathBuf, title: &str, content: &str) -> String {
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let chapter_id = chapter_repository::create_chapter(&conn, &novel_dir, title, None).unwrap();
        chapter_repository::update_chapter_content(&conn, &novel_dir, &chapter_id, content).unwrap();
        chapter_id
    }

    #[test]
    fn test_set_mainplot_landing_ok() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");

        let result = set_mainplot_landing(&project_dir, &chapter_id, "mp-1-001", true).unwrap();
        assert!(result.achieved);
        assert!(!result.content_hash.is_empty());
        assert_eq!(result.node_id, "mp-1-001");
    }

    #[test]
    fn test_set_mainplot_landing_overwrite_same_node() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");

        // 第一次 achieved=0
        set_mainplot_landing(&project_dir, &chapter_id, "mp-1-001", false).unwrap();
        // 同节点覆盖为 achieved=1
        let result = set_mainplot_landing(&project_dir, &chapter_id, "mp-1-001", true).unwrap();
        assert!(result.achieved);
    }

    #[test]
    fn test_set_mainplot_landing_external_modification_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "原始正文");

        // 外部修改正文文件
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, &chapter_id).unwrap();
        let content_path = novel_dir.join(&row.content_path);
        fs::write(&content_path, "外部修改").unwrap();

        let result = set_mainplot_landing(&project_dir, &chapter_id, "mp-1-001", true);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::ContentModifiedExternally { .. }));
    }

    #[test]
    fn test_set_mainplot_landing_chapter_not_found() {
        let (_dir, project_dir) = setup();

        let result = set_mainplot_landing(&project_dir, "chapter-999", "mp-1-001", true);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::ChapterNotFound { .. }));
    }

    #[test]
    fn test_set_mainplot_landing_approved_blocked() {
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");

        // 直接把章节状态设为 approved
        let novel_dir = project_dir.join(".novel");
        let conn = Connection::open(novel_dir.join("novel.db")).unwrap();
        crate::storage::chapters::update_status(&conn, &chapter_id, "approved", "2026-07-01T01:00:00Z").unwrap();

        let result = set_mainplot_landing(&project_dir, &chapter_id, "mp-1-001", true);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_set_mainplot_landing_world_not_initialized() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().to_path_buf();

        let result = set_mainplot_landing(&project_dir, "chapter-001", "mp-1-001", true);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::PreconditionBlocked { .. }));
    }

    #[test]
    fn test_set_mainplot_landing_node_not_found() {
        // 缺陷修复：引用不存在的主线节点应返回 mainplot_node_not_found（退出码 43），
        // 而非静默写入悬空裁决。
        let (_dir, project_dir) = setup();
        let chapter_id = create_and_write_chapter(&project_dir, "测试", "正文内容");

        let result = set_mainplot_landing(&project_dir, &chapter_id, "mp-1-999", true);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::MainplotNodeNotFound { .. }));
    }
}
