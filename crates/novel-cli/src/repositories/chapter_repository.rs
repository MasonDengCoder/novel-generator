// chapter_repository 模块：封装章节 CRUD 操作（task 6.1-6.6）。

use crate::errors::CliError;
use crate::storage::chapters::{self};
use crate::storage::file_ops::{self};
use crate::validators::chapter_validator;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// 章节完整数据（元数据 + 正文内容）。
#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    pub content_hash: String,
    pub status: String,
    pub volume_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建新章节（生成递增 chapter-id，插入 SQLite 记录）。
pub fn create_chapter(
    conn: &Connection,
    novel_dir: &Path,
    title: &str,
    volume_id: Option<i64>,
) -> Result<String, CliError> {
    // 生成下一个章节 ID
    let chapter_id = chapters::generate_next_chapter_id(conn)?;

    // 构造正文文件路径（相对于 .novel 目录，存储在 SQLite 中）
    let content_path = format!("content/chapters/{}.md", chapter_id);

    // 初始化空正文
    let initial_content = "";
    let content_hash = chapter_validator::calculate_content_hash(initial_content);
    let word_count = 0;

    // 获取当前时间
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 插入 SQLite 记录（volume_id 透传，Some 时归属卷，None 时留空）
    chapters::insert_chapter(
        conn,
        &chapter_id,
        title,
        &content_path,
        &content_hash,
        word_count,
        volume_id,
        &timestamp,
    )?;

    // 创建空正文文件（novel_dir 是 .novel 目录，所以只需 join content/chapters/...）
    let full_content_path = novel_dir.join(format!("content/chapters/{}.md", chapter_id));
    if let Some(parent) = full_content_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CliError::storage(format!("创建章节目录失败：{} - {}", parent.display(), e)))?;
    }
    std::fs::write(&full_content_path, initial_content)
        .map_err(|e| CliError::storage(format!("创建空正文文件失败：{} - {}", full_content_path.display(), e)))?;

    Ok(chapter_id)
}

/// 待完成的正文替换：事务提交后由 `finalize_content_replace` 执行原子替换。
///
/// 设计 D1（commit-then-rename）：先在事务内写 temp + 更新 SQLite 元数据，
/// 由 service 层提交事务后，再执行 temp→正式正文的原子替换。
/// 这样 SQLite 元数据更新失败时事务回滚、原子替换未执行，正文与元数据保持一致，
/// 不留「正文已替换但元数据未更新」的半提交状态（F-02，不变量 9）。
pub struct PendingContentReplace {
    pub temp_path: PathBuf,
    pub target_path: PathBuf,
}

/// 准备章节正文更新（写 temp + 更新元数据，在事务内）。
///
/// 在 service 开启的事务句柄内调用：写临时文件、于事务内更新 `content_hash`/`word_count`/
/// `updated_at`，返回 `PendingContentReplace`。调用方（service）提交事务成功后，须调
/// `finalize_content_replace` 执行原子替换；事务回滚时须调 `cleanup_temp_file`（或让
/// `PendingContentReplace` drop 时清理）。
///
/// 失败补偿：
/// - `write_temp_file` 失败：无 temp、无 SQLite 变更，直接返回错误。
/// - `update_chapter_metadata` 失败：清理 temp 后返回错误，事务由 service 回滚（SQLite 未变更）。
pub fn prepare_content_update(
    conn: &Connection,
    novel_dir: &Path,
    chapter_id: &str,
    content: &str,
) -> Result<PendingContentReplace, CliError> {
    // 获取章节元数据
    let chapter_row = chapters::get_chapter(conn, chapter_id)?;

    // 计算新 hash 和 word_count
    let new_hash = chapter_validator::calculate_content_hash(content);
    let new_word_count = chapter_validator::calculate_word_count(content) as i64;

    // 获取当前时间
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 写入临时文件
    let temp_path = file_ops::write_temp_file(novel_dir, content, "md")?;

    // 在事务内更新 SQLite 元数据。失败时清理 temp 并返回错误（事务由 service 回滚，
    // 正文未替换 → 正文与元数据均为旧值 → 一致，不留半提交状态）。
    if let Err(e) =
        chapters::update_chapter_metadata(conn, chapter_id, &new_hash, new_word_count, &timestamp)
    {
        file_ops::cleanup_temp_file(&temp_path);
        return Err(e);
    }

    let target_path = novel_dir.join(&chapter_row.content_path);
    Ok(PendingContentReplace {
        temp_path,
        target_path,
    })
}

/// 执行事务提交后的正文原子替换（commit-then-rename 的 rename 步）。
///
/// 在 SQLite 元数据已提交为新 hash 后，把 temp 原子替换为正式正文。若替换失败（如权限不足），
/// 清理 temp 并返回错误——此时 SQLite 元数据为新 hash、正文为旧内容，产生 mismatch，
/// 由现有 hash 检测捕获并拒绝；用户可运行 `novel chapter reimport <id>` 修复。
pub fn finalize_content_replace(pending: PendingContentReplace) -> Result<(), CliError> {
    match file_ops::atomic_replace(&pending.temp_path, &pending.target_path) {
        Ok(()) => Ok(()),
        Err(e) => {
            file_ops::cleanup_temp_file(&pending.temp_path);
            Err(e)
        }
    }
}

impl Drop for PendingContentReplace {
    /// 若未显式 finalize，drop 时清理残留 temp，避免孤儿临时文件。
    fn drop(&mut self) {
        file_ops::cleanup_temp_file(&self.temp_path);
    }
}

/// 单步正文更新便捷封装（无显式事务）：prepare + finalize。
///
/// 适用于无 service 层事务编排的调用方（如裸 `&Connection` 自动提交路径、测试 fixture）。
/// 元数据更新在自动提交下即时落库，随后执行原子替换——commit-then-rename 语义不变。
/// 正式写路径须经 service 层 `prepare_content_update` + `tx.commit()` + `finalize_content_replace`
/// 编排（见 `services/chapter_service`），不得直接用本封装绕过事务与 guard。
pub fn update_chapter_content(
    conn: &Connection,
    novel_dir: &Path,
    chapter_id: &str,
    content: &str,
) -> Result<(), CliError> {
    let pending = prepare_content_update(conn, novel_dir, chapter_id, content)?;
    finalize_content_replace(pending)
}

/// 读取章节（读取元数据，验证 hash，读取正文）。
pub fn get_chapter(
    conn: &Connection,
    novel_dir: &Path,
    chapter_id: &str,
) -> Result<Chapter, CliError> {
    // 读取元数据
    let row = chapters::get_chapter(conn, chapter_id)?;

    // 读取正文文件
    let content_path = novel_dir.join(&row.content_path);
    let content = std::fs::read_to_string(&content_path).map_err(|e| {
        CliError::file_not_found(format!("正文文件不存在：{} - {}", content_path.display(), e))
    })?;

    // 验证 hash
    let current_hash = chapter_validator::calculate_content_hash(&content);
    if current_hash != row.content_hash {
        return Err(CliError::content_modified_externally(format!(
            "章节 {} 的正文已被外部修改，请运行 `novel chapter reimport {}` 重新导入。",
            chapter_id, chapter_id
        )));
    }

    Ok(Chapter {
        id: row.id,
        title: row.title,
        content,
        word_count: row.word_count,
        content_hash: row.content_hash,
        status: row.status,
        volume_id: row.volume_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

/// 列出所有章节（读取所有章节元数据，可选 hash 验证）。
pub fn list_chapters(
    conn: &Connection,
    novel_dir: &Path,
    verify_hash: bool,
) -> Result<Vec<ChapterListItem>, CliError> {
    // 读取所有章节元数据
    let rows = chapters::list_chapters(conn)?;

    let mut chapters = Vec::new();
    let mut has_mismatch = false;

    for row in rows {
        let mut hash_verified = true;
        let mut hash_mismatch = false;

        if verify_hash {
            // 读取正文文件并验证 hash
            let content_path = novel_dir.join(&row.content_path);
            match std::fs::read_to_string(&content_path) {
                Ok(content) => {
                    let current_hash = chapter_validator::calculate_content_hash(&content);
                    if current_hash != row.content_hash {
                        hash_verified = false;
                        hash_mismatch = true;
                        has_mismatch = true;
                    }
                }
                Err(_) => {
                    hash_verified = false;
                    hash_mismatch = true;
                    has_mismatch = true;
                }
            }
        }

        chapters.push(ChapterListItem {
            id: row.id,
            title: row.title,
            word_count: row.word_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
            hash_verified,
            hash_mismatch,
        });
    }

    // 如果发现 hash 不一致，返回错误
    if has_mismatch {
        return Err(CliError::content_modified_externally(
            "检测到部分章节正文已被外部修改，请检查 hashMismatch 字段。".to_string()
        ));
    }

    Ok(chapters)
}

/// 章节列表项（元数据摘要 + hash 验证状态）。
#[derive(Debug, Clone)]
pub struct ChapterListItem {
    pub id: String,
    pub title: String,
    pub word_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub hash_verified: bool,
    pub hash_mismatch: bool,
}

/// 重新导入章节（重新计算 hash，更新元数据）。
pub fn reimport_chapter(
    conn: &Connection,
    novel_dir: &Path,
    chapter_id: &str,
) -> Result<(), CliError> {
    // 读取章节元数据
    let row = chapters::get_chapter(conn, chapter_id)?;

    // 读取正文文件
    let content_path = novel_dir.join(&row.content_path);
    let content = std::fs::read_to_string(&content_path).map_err(|e| {
        CliError::file_not_found(format!("正文文件不存在：{} - {}", content_path.display(), e))
    })?;

    // 重新计算 hash 和 word_count
    let new_hash = chapter_validator::calculate_content_hash(&content);
    let new_word_count = chapter_validator::calculate_word_count(&content) as i64;

    // 获取当前时间
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    // 更新 SQLite 元数据
    chapters::update_chapter_metadata(conn, chapter_id, &new_hash, new_word_count, &timestamp)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup() -> (Connection, PathBuf, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let novel_dir = dir.path().join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();

        let db_path = novel_dir.join("novel.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();

        (conn, novel_dir, dir)
    }

    #[test]
    fn test_create_chapter() {
        let (conn, novel_dir, _dir) = setup();

        let chapter_id = create_chapter(&conn, &novel_dir, "第一章", None).unwrap();
        assert_eq!(chapter_id, "chapter-001");

        // 验证 SQLite 记录
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        assert_eq!(row.title, "第一章");
        assert_eq!(row.word_count, 0);

        // 验证正文文件存在
        let content_path = novel_dir.join(&row.content_path);
        assert!(content_path.exists());
    }

    #[test]
    fn test_update_chapter_content() {
        let (conn, novel_dir, _dir) = setup();

        let chapter_id = create_chapter(&conn, &novel_dir, "测试章节", None).unwrap();

        // 更新正文
        let content = "这是章节正文内容。\n包含多行。";
        update_chapter_content(&conn, &novel_dir, &chapter_id, content).unwrap();

        // 验证元数据已更新
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        assert!(row.word_count > 0);
        assert_ne!(row.content_hash, chapter_validator::calculate_content_hash(""));

        // 验证正文文件内容
        let content_path = novel_dir.join(&row.content_path);
        let saved_content = std::fs::read_to_string(&content_path).unwrap();
        assert_eq!(saved_content, content);
    }

    #[test]
    fn test_get_chapter() {
        let (conn, novel_dir, _dir) = setup();

        let chapter_id = create_chapter(&conn, &novel_dir, "章节标题", None).unwrap();
        let content = "章节正文";
        update_chapter_content(&conn, &novel_dir, &chapter_id, content).unwrap();

        // 读取章节
        let chapter = get_chapter(&conn, &novel_dir, &chapter_id).unwrap();
        assert_eq!(chapter.id, chapter_id);
        assert_eq!(chapter.title, "章节标题");
        assert_eq!(chapter.content, content);
    }

    #[test]
    fn test_get_chapter_hash_mismatch() {
        let (conn, novel_dir, _dir) = setup();

        let chapter_id = create_chapter(&conn, &novel_dir, "测试", None).unwrap();
        update_chapter_content(&conn, &novel_dir, &chapter_id, "原始内容").unwrap();

        // 外部修改正文文件
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let content_path = novel_dir.join(&row.content_path);
        std::fs::write(&content_path, "外部修改后的内容").unwrap();

        // 读取章节应返回错误
        let result = get_chapter(&conn, &novel_dir, &chapter_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("外部修改"));
    }

    #[test]
    fn test_list_chapters() {
        let (conn, novel_dir, _dir) = setup();

        create_chapter(&conn, &novel_dir, "第一章", None).unwrap();
        create_chapter(&conn, &novel_dir, "第二章", None).unwrap();

        let chapters = list_chapters(&conn, &novel_dir, false).unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].id, "chapter-001");
        assert_eq!(chapters[1].id, "chapter-002");
    }

    #[test]
    fn test_list_chapters_verify_hash() {
        let (conn, novel_dir, _dir) = setup();

        let id1 = create_chapter(&conn, &novel_dir, "第一章", None).unwrap();
        let id2 = create_chapter(&conn, &novel_dir, "第二章", None).unwrap();

        update_chapter_content(&conn, &novel_dir, &id1, "内容1").unwrap();
        update_chapter_content(&conn, &novel_dir, &id2, "内容2").unwrap();

        // 验证 hash 应成功
        let chapters = list_chapters(&conn, &novel_dir, true).unwrap();
        assert_eq!(chapters.len(), 2);
        assert!(chapters[0].hash_verified);
        assert!(chapters[1].hash_verified);
    }

    #[test]
    fn test_list_chapters_hash_mismatch() {
        let (conn, novel_dir, _dir) = setup();

        let id1 = create_chapter(&conn, &novel_dir, "第一章", None).unwrap();
        update_chapter_content(&conn, &novel_dir, &id1, "内容").unwrap();

        // 外部修改正文
        let row = chapters::get_chapter(&conn, &id1).unwrap();
        let content_path = novel_dir.join(&row.content_path);
        std::fs::write(&content_path, "外部修改").unwrap();

        // 验证 hash 应返回错误
        let result = list_chapters(&conn, &novel_dir, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_reimport_chapter() {
        let (conn, novel_dir, _dir) = setup();

        let chapter_id = create_chapter(&conn, &novel_dir, "测试", None).unwrap();
        update_chapter_content(&conn, &novel_dir, &chapter_id, "原始内容").unwrap();

        // 外部修改正文
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let content_path = novel_dir.join(&row.content_path);
        let new_content = "外部修改的内容";
        std::fs::write(&content_path, new_content).unwrap();

        // 重新导入
        reimport_chapter(&conn, &novel_dir, &chapter_id).unwrap();

        // 现在读取应成功
        let chapter = get_chapter(&conn, &novel_dir, &chapter_id).unwrap();
        assert_eq!(chapter.content, new_content);
    }

    // ===== F-02 一致性测试（三类失败点 + 成功路径）=====
    // 覆盖 design D1/D2：commit-then-rename + temp 清理补偿，验证任一失败点后
    // 正文 hash 与 SQLite content_hash 一致、.novel/tmp/ 无孤儿临时文件。

    fn tmp_file_count(novel_dir: &Path) -> usize {
        let tmp_dir = novel_dir.join("tmp");
        std::fs::read_dir(&tmp_dir)
            .map(|entries| entries.count())
            .unwrap_or(0)
    }

    #[cfg(unix)]
    fn set_readonly(path: &Path, readonly: bool) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(if readonly { 0o500 } else { 0o755 });
        std::fs::set_permissions(path, perms).unwrap();
    }

    #[test]
    fn test_update_sqlite_metadata_failure_consistent() {
        // F-02 失败点①：SQLite 元数据更新失败时，正文未替换、hash 一致、无孤儿 temp。
        // 注入方式：在 chapters 表上建 BEFORE UPDATE 触发器 RAISE(ABORT)，使
        // update_chapter_metadata 的 UPDATE 失败，而 get_chapter（SELECT）仍成功。
        let (conn, novel_dir, _dir) = setup();
        let chapter_id = create_chapter(&conn, &novel_dir, "测试", None).unwrap();
        let old_content = "旧内容";
        update_chapter_content(&conn, &novel_dir, &chapter_id, old_content).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let old_hash = row.content_hash.clone();
        let content_path = novel_dir.join(&row.content_path);

        // 注入元数据更新失败触发器
        conn.execute_batch(
            "CREATE TRIGGER fail_chapter_metadata_update BEFORE UPDATE ON chapters \
             BEGIN SELECT RAISE(ABORT, 'injected metadata failure'); END;",
        )
        .unwrap();

        let content = "新内容";
        let result = update_chapter_content(&conn, &novel_dir, &chapter_id, content);
        assert!(result.is_err(), "元数据更新失败应返回错误");
        assert!(
            result.unwrap_err().to_string().contains("injected"),
            "错误应来自注入的元数据失败"
        );

        // 失败后：正文文件仍是旧内容（原子替换未执行）
        let saved = std::fs::read_to_string(&content_path).unwrap();
        assert_eq!(saved, old_content, "失败后正文应保持旧内容");

        // SQLite content_hash 仍是旧值（UPDATE 被触发器中止，未变更）
        let row_after = chapters::get_chapter(&conn, &chapter_id).unwrap();
        assert_eq!(
            row_after.content_hash, old_hash,
            "失败后 SQLite content_hash 应为旧值（未更新）"
        );

        // 正文文件 hash 与 SQLite content_hash 一致（均为旧）
        let file_hash = chapter_validator::calculate_content_hash(&saved);
        assert_eq!(
            file_hash, row_after.content_hash,
            "失败后正文 hash 与 SQLite content_hash 应一致"
        );

        // .novel/tmp/ 无孤儿临时文件
        assert_eq!(tmp_file_count(&novel_dir), 0, "失败后应清理临时文件");
    }

    #[cfg(unix)]
    #[test]
    fn test_update_atomic_replace_failure_cleans_temp() {
        // F-02 失败点②：原子替换失败（commit 成功后）时清理 temp。
        // 注入方式：使正式正文的目标目录只读，atomic_replace 的 rename 失败。
        let (conn, novel_dir, _dir) = setup();
        let chapter_id = create_chapter(&conn, &novel_dir, "测试", None).unwrap();
        let content = "新内容";
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let target_dir = novel_dir.join("content").join("chapters");

        // 准备（写 temp + 元数据成功）——目标目录此刻可写
        let pending = prepare_content_update(&conn, &novel_dir, &chapter_id, content).unwrap();

        // 元数据已自动提交为新 hash（裸 conn 自动提交）
        let new_row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let new_hash = chapter_validator::calculate_content_hash(content);
        assert_eq!(new_row.content_hash, new_hash, "元数据应已更新为新 hash");

        // 使目标目录只读 → finalize 的 rename 失败
        set_readonly(&target_dir, true);
        let result = finalize_content_replace(pending);
        set_readonly(&target_dir, false); // 恢复以便后续断言可读
        assert!(result.is_err(), "原子替换应失败");

        // temp 已清理
        assert_eq!(tmp_file_count(&novel_dir), 0, "替换失败后应清理临时文件");

        // 正文文件保持旧内容（替换未成功）
        let content_path = novel_dir.join(&row.content_path);
        let saved = std::fs::read_to_string(&content_path).unwrap();
        assert_eq!(saved, "", "失败后正文应保持原状（创建时的空内容）");

        // 此时元数据为新 hash、正文为旧 → mismatch（可被 get_chapter 检测）
        let get_result = get_chapter(&conn, &novel_dir, &chapter_id);
        assert!(get_result.is_err(), "mismatch 应被 hash 检测拒绝");
    }

    #[cfg(unix)]
    #[test]
    fn test_update_temp_write_failure_no_side_effect() {
        // F-02 失败点③：临时文件写入失败时，无副作用（正文未变、元数据未变、无 temp）。
        // 注入方式：使 .novel/tmp/ 只读 → write_temp_file 失败。
        let (conn, novel_dir, _dir) = setup();
        let chapter_id = create_chapter(&conn, &novel_dir, "测试", None).unwrap();
        let old_content = "";
        update_chapter_content(&conn, &novel_dir, &chapter_id, old_content).unwrap();
        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let old_hash = row.content_hash.clone();

        // 使 tmp 目录只读 → write_temp_file 的 File::create 失败
        let tmp_dir = novel_dir.join("tmp");
        set_readonly(&tmp_dir, true);
        let result = update_chapter_content(&conn, &novel_dir, &chapter_id, "新内容");
        set_readonly(&tmp_dir, false);
        assert!(result.is_err(), "临时文件写入应失败");

        // 正文文件未变（仍为旧内容）
        let content_path = novel_dir.join(&row.content_path);
        let saved = std::fs::read_to_string(&content_path).unwrap();
        assert_eq!(saved, old_content, "临时写入失败不应改变正文");

        // SQLite 元数据未变更（content_hash 仍为旧值）
        let row_after = chapters::get_chapter(&conn, &chapter_id).unwrap();
        assert_eq!(row_after.content_hash, old_hash, "临时写入失败不应更新元数据");

        // 无临时文件残留
        assert_eq!(tmp_file_count(&novel_dir), 0, "不应产生临时文件");
    }

    #[test]
    fn test_update_success_syncs_all() {
        // F-02 成功路径：正文内容、content_hash、word_count、updated_at 四者同步更新且 hash 一致。
        let (conn, novel_dir, _dir) = setup();
        let chapter_id = create_chapter(&conn, &novel_dir, "测试", None).unwrap();

        let content = "成功路径的章节正文。\n包含多行。";
        update_chapter_content(&conn, &novel_dir, &chapter_id, content).unwrap();

        let row = chapters::get_chapter(&conn, &chapter_id).unwrap();
        let expected_hash = chapter_validator::calculate_content_hash(content);
        let expected_words = chapter_validator::calculate_word_count(content) as i64;

        // 正文文件内容 = 写入内容
        let content_path = novel_dir.join(&row.content_path);
        let saved = std::fs::read_to_string(&content_path).unwrap();
        assert_eq!(saved, content, "正文文件内容应已更新");

        // content_hash / word_count 同步
        assert_eq!(row.content_hash, expected_hash, "content_hash 应同步");
        assert_eq!(row.word_count, expected_words, "word_count 应同步");
        assert!(row.updated_at.len() > 0, "updated_at 应已更新");

        // 正文 hash 与 SQLite content_hash 一致
        let file_hash = chapter_validator::calculate_content_hash(&saved);
        assert_eq!(file_hash, row.content_hash, "正文 hash 与 SQLite 应一致");

        // 成功后无孤儿 temp
        assert_eq!(tmp_file_count(&novel_dir), 0, "成功后应无临时文件残留");

        // 读取章节成功（hash 一致）
        let chapter = get_chapter(&conn, &novel_dir, &chapter_id).unwrap();
        assert_eq!(chapter.content, content);
    }
}
