// chapter_service 模块：章节业务逻辑与状态机检查（task 7.1-7.6）。

use crate::errors::CliError;
use crate::repositories::chapter_repository::{self, Chapter, ChapterListItem};
use crate::repositories::volume_repository;
use crate::storage::connection;
use crate::storage::file_ops::ProjectWriteLock;
use crate::validators::chapter_validator;
use rusqlite::TransactionBehavior;
use std::path::Path;

/// 创建新章节（检查项目已初始化，校验卷归属，调用 validator 和 repository）。
///
/// `volume_id` 为 `Some` 时校验卷存在且状态为 `active`，通过后写入 `chapters.volume_id`；
/// 为 `None` 时 `volume_id` 留空（沿用既有不归属行为）。卷不存在返回 `volume_not_found`（码6），
/// 卷非 active（sealing/sealed/archived）返回 `volume_already_sealed`/`volume_already_archived`（码3）。
pub fn create_chapter(target_dir: &Path, title: &str, volume_id: Option<i64>) -> Result<String, CliError> {
    // 校验标题
    chapter_validator::validate_chapter_title(title)?;

    // 获取项目目录
    let novel_dir = target_dir.join(".novel");

    // 先获取写锁，再打开数据库（避免锁冲突）
    let _lock = ProjectWriteLock::acquire(&novel_dir)?;

    // 打开数据库（自动检查项目已初始化）
    let conn = connection::open_existing_at(target_dir)?;

    // 卷归属校验：--volume 传入时校验卷存在且状态==active
    if let Some(vid) = volume_id {
        // get_volume_by_id 卷不存在时返回通用 not_found，转成语义化 volume_not_found（对齐 spec）
        let volume = volume_repository::get_volume_by_id(&conn, vid).map_err(|e| match e {
            CliError::NotFound { message, .. } => CliError::volume_not_found(message),
            other => other,
        })?;
        match volume.status.as_str() {
            "active" => { /* 允许加章 */ }
            "sealing" | "sealed" => {
                return Err(CliError::volume_already_sealed(format!(
                    "卷 {} 当前状态为 {}，无法在其下新建章节（仅 active 卷允许加章）。",
                    vid, volume.status
                )));
            }
            "archived" => {
                return Err(CliError::volume_already_archived(format!(
                    "卷 {} 已归档，无法在其下新建章节。",
                    vid
                )));
            }
            other => {
                return Err(CliError::volume_already_sealed(format!(
                    "卷 {} 当前状态为 {}，无法在其下新建章节。",
                    vid, other
                )));
            }
        }
    }

    // 创建章节（volume_id 透传给 repository/storage）
    let chapter_id = chapter_repository::create_chapter(&conn, &novel_dir, title, volume_id)?;

    Ok(chapter_id)
}

/// 更新章节正文（检查章节存在、未归档，执行写入 transaction）。
pub fn update_chapter_content(
    target_dir: &Path,
    chapter_id: &str,
    content_file: &Path,
) -> Result<Chapter, CliError> {
    // 校验并读取正文文件
    let content = chapter_validator::validate_content_file(content_file)?;

    // 获取项目目录
    let novel_dir = target_dir.join(".novel");

    // 先获取写锁，再打开数据库（避免锁冲突）
    let _lock = ProjectWriteLock::acquire(&novel_dir)?;

    // 打开数据库
    let mut conn = connection::open_existing_at(target_dir)?;

    // s07 归档冻结门禁：章节所属卷 archived 时拒绝修改正文。
    crate::validators::guards::volume_freeze_guard::ensure_chapter_volume_not_archived(
        &conn, chapter_id,
    )?;

    // approved 终态冻结：approved 章节拒绝修改正文（CLAUDE.md §7、state-machine.md:41）。
    // 与 verdict_service 同模式：退出码 3 + precondition_blocked。
    let chapter = crate::storage::chapters::get_chapter(&conn, chapter_id)?;
    if chapter.status == "approved" {
        return Err(CliError::precondition_blocked(format!(
            "章节 {} 已 approved，正文不可修改（终态冻结）。",
            chapter_id
        )));
    }

    // 开启事务（IMMEDIATE：写事务开始即持 SQLite 写锁，配合 ProjectWriteLock 的 flock
    // 形成「CLI 进程间 flock 互斥 + 持锁期间裸 SQLite 写被 DB 级锁阻塞」双重关键段，
    // 防止不经 novel 命令的裸 SQLite 写入绕过 CLI 的状态机/guard/hash 一致性保障。）
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| CliError::storage(format!("开启事务失败：{}", e)))?;

    // 准备正文更新（写 temp + 在事务内更新元数据）。commit-then-rename（design D1）：
    // 元数据在事务内更新，事务提交成功后才执行 temp→正式正文的原子替换，使元数据更新
    // 失败时事务回滚、正文未替换，不留半提交不一致状态（F-02，不变量 9）。
    let pending = chapter_repository::prepare_content_update(&tx, &novel_dir, chapter_id, &content)?;

    // 提交事务。失败时 `pending` drop 清理 temp，正文未替换 → 一致。
    tx.commit()
        .map_err(|e| CliError::storage(format!("提交事务失败：{}", e)))?;

    // 事务提交成功后执行原子替换。失败时清理 temp 并返回错误（元数据新/正文旧的
    // mismatch 由现有 hash 检测捕获，可经 `chapter reimport` 修复）。
    chapter_repository::finalize_content_replace(pending)?;

    // 在仍持有项目写锁时读取刚提交的权威版本，供机器回执使用。
    chapter_repository::get_chapter(&conn, &novel_dir, chapter_id)
}

/// 读取章节（检查章节存在，验证 hash，返回完整数据）。
pub fn get_chapter(target_dir: &Path, chapter_id: &str) -> Result<Chapter, CliError> {
    // 打开数据库
    let conn = connection::open_existing_at(target_dir)?;

    // 获取项目目录
    let novel_dir = target_dir.join(".novel");

    // 读取章节
    chapter_repository::get_chapter(&conn, &novel_dir, chapter_id)
}

/// 列出所有章节（可选 hash 验证，返回元数据列表）。
pub fn list_chapters(
    target_dir: &Path,
    verify_hash: bool,
) -> Result<Vec<ChapterListItem>, CliError> {
    // 打开数据库
    let conn = connection::open_existing_at(target_dir)?;

    // 获取项目目录
    let novel_dir = target_dir.join(".novel");

    // 列出章节
    chapter_repository::list_chapters(&conn, &novel_dir, verify_hash)
}

/// 重新导入章节（检查章节存在，重新计算 hash）。
pub fn reimport_chapter(target_dir: &Path, chapter_id: &str) -> Result<(), CliError> {
    // 获取项目目录
    let novel_dir = target_dir.join(".novel");

    // 先获取写锁，再打开数据库（避免锁冲突）
    let _lock = ProjectWriteLock::acquire(&novel_dir)?;

    // 打开数据库
    let mut conn = connection::open_existing_at(target_dir)?;

    // 开启事务（IMMEDIATE：见 update_chapter_content 注释，正文写路径统一防绕过双锁。）
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| CliError::storage(format!("开启事务失败：{}", e)))?;

    // 重新导入章节
    chapter_repository::reimport_chapter(&tx, &novel_dir, chapter_id)?;

    // 提交事务
    tx.commit()
        .map_err(|e| CliError::storage(format!("提交事务失败：{}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
            run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
        } // Connection 在此处 drop，释放锁

        (dir, project_dir)
    }

    #[test]
    fn test_create_chapter_ok() {
        let (_dir, project_dir) = setup();

        let chapter_id = create_chapter(&project_dir, "第一章", None).unwrap();
        assert_eq!(chapter_id, "chapter-001");
    }

    #[test]
    fn test_create_chapter_invalid_title() {
        let (_dir, project_dir) = setup();

        let result = create_chapter(&project_dir, "", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("标题不能为空"));
    }

    #[test]
    fn test_create_chapter_project_not_initialized() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().to_path_buf();

        let result = create_chapter(&project_dir, "标题", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未初始化"));
    }

    #[test]
    fn test_update_chapter_content_ok() {
        let (_dir, project_dir) = setup();

        let chapter_id = create_chapter(&project_dir, "测试", None).unwrap();

        // 创建临时正文文件
        let content_file = _dir.path().join("content.md");
        fs::write(&content_file, "章节正文内容").unwrap();

        update_chapter_content(&project_dir, &chapter_id, &content_file).unwrap();

        // 验证更新成功
        let chapter = get_chapter(&project_dir, &chapter_id).unwrap();
        assert_eq!(chapter.content, "章节正文内容");
    }

    #[test]
    fn test_update_chapter_not_found() {
        let (_dir, project_dir) = setup();

        let content_file = _dir.path().join("content.md");
        fs::write(&content_file, "内容").unwrap();

        let result = update_chapter_content(&project_dir, "chapter-999", &content_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_chapter_file_not_found() {
        let (_dir, project_dir) = setup();

        let chapter_id = create_chapter(&project_dir, "测试", None).unwrap();
        let nonexistent = _dir.path().join("nonexistent.md");

        let result = update_chapter_content(&project_dir, &chapter_id, &nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_chapter_ok() {
        let (_dir, project_dir) = setup();

        let chapter_id = create_chapter(&project_dir, "标题", None).unwrap();

        let content_file = _dir.path().join("content.md");
        fs::write(&content_file, "正文").unwrap();
        update_chapter_content(&project_dir, &chapter_id, &content_file).unwrap();

        let chapter = get_chapter(&project_dir, &chapter_id).unwrap();
        assert_eq!(chapter.id, chapter_id);
        assert_eq!(chapter.title, "标题");
        assert_eq!(chapter.content, "正文");
    }

    #[test]
    fn test_get_chapter_not_found() {
        let (_dir, project_dir) = setup();

        let result = get_chapter(&project_dir, "chapter-999");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_chapters_ok() {
        let (_dir, project_dir) = setup();

        create_chapter(&project_dir, "第一章", None).unwrap();
        create_chapter(&project_dir, "第二章", None).unwrap();

        let chapters = list_chapters(&project_dir, false).unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章");
        assert_eq!(chapters[1].title, "第二章");
    }

    #[test]
    fn test_list_chapters_empty() {
        let (_dir, project_dir) = setup();

        let chapters = list_chapters(&project_dir, false).unwrap();
        assert_eq!(chapters.len(), 0);
    }

    #[test]
    fn test_reimport_chapter_ok() {
        let (_dir, project_dir) = setup();

        let chapter_id = create_chapter(&project_dir, "测试", None).unwrap();

        let content_file = _dir.path().join("content.md");
        fs::write(&content_file, "原始内容").unwrap();
        update_chapter_content(&project_dir, &chapter_id, &content_file).unwrap();

        // 读取章节以获取实际的正文文件路径
        let chapter = get_chapter(&project_dir, &chapter_id).unwrap();
        assert_eq!(chapter.content, "原始内容");

        // 通过 repository 层获取正文路径，然后外部修改
        let novel_dir = project_dir.join(".novel");
        let conn = crate::storage::connection::open_existing_at(&project_dir).unwrap();
        let row = crate::storage::chapters::get_chapter(&conn, &chapter_id).unwrap();
        let content_path = novel_dir.join(&row.content_path);

        // 外部修改正文文件
        fs::write(&content_path, "外部修改").unwrap();

        // 重新导入
        reimport_chapter(&project_dir, &chapter_id).unwrap();

        // 现在读取应成功
        let chapter = get_chapter(&project_dir, &chapter_id).unwrap();
        assert_eq!(chapter.content, "外部修改");
    }

    #[test]
    fn test_reimport_chapter_not_found() {
        let (_dir, project_dir) = setup();

        let result = reimport_chapter(&project_dir, "chapter-999");
        assert!(result.is_err());
    }
}
