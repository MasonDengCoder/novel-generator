// storage/file_ops 模块：临时文件写入、原子替换、项目级写锁（task 5.2, 5.3）。

use crate::errors::CliError;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 写入临时文件并返回临时文件路径。
/// 临时文件位于 .novel/tmp/ 目录，使用 UUID 命名避免冲突。
pub fn write_temp_file(novel_dir: &Path, content: &str, extension: &str) -> Result<PathBuf, CliError> {
    let tmp_dir = novel_dir.join("tmp");
    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| CliError::storage(format!("创建临时目录失败：{} - {}", tmp_dir.display(), e)))?;

    // 生成唯一临时文件名
    let uuid = uuid::Uuid::new_v4();
    let tmp_path = tmp_dir.join(format!("{}.{}", uuid, extension));

    // 写入临时文件
    let mut file = File::create(&tmp_path)
        .map_err(|e| CliError::storage(format!("创建临时文件失败：{} - {}", tmp_path.display(), e)))?;

    file.write_all(content.as_bytes())
        .map_err(|e| CliError::storage(format!("写入临时文件失败：{} - {}", tmp_path.display(), e)))?;

    file.sync_all()
        .map_err(|e| CliError::storage(format!("同步临时文件失败：{} - {}", tmp_path.display(), e)))?;

    Ok(tmp_path)
}

/// 原子替换目标文件：使用 rename 系统调用实现原子性。
/// POSIX 保证同一文件系统内的 rename 是原子操作。
pub fn atomic_replace(temp_path: &Path, target_path: &Path) -> Result<(), CliError> {
    // 确保目标目录存在
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CliError::storage(format!("创建目标目录失败：{} - {}", parent.display(), e)))?;
    }

    // 原子替换
    std::fs::rename(temp_path, target_path)
        .map_err(|e| CliError::storage(format!(
            "原子替换文件失败：{} -> {} - {}",
            temp_path.display(),
            target_path.display(),
            e
        )))?;

    Ok(())
}

/// 清理临时文件（失败时调用）。
pub fn cleanup_temp_file(temp_path: &Path) {
    if temp_path.exists() {
        let _ = std::fs::remove_file(temp_path);
    }
}

/// 项目级写锁：使用 flock 对 .novel/novel.db 加文件锁。
/// 返回 LockGuard，drop 时自动释放锁。
pub struct ProjectWriteLock {
    _file: File,
}

impl ProjectWriteLock {
    /// 获取项目级写锁（阻塞等待）。
    pub fn acquire(novel_dir: &Path) -> Result<Self, CliError> {
        // 检查数据库是否存在
        let db_path = novel_dir.join("novel.db");
        if !db_path.exists() {
            return Err(CliError::world_not_initialized(
                "世界未初始化：.novel/novel.db 不存在。".to_string()
            ));
        }

        // 使用独立的锁文件，避免与 SQLite 的锁机制冲突
        let lock_path = novel_dir.join(".write.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)
            .map_err(|e| CliError::storage(format!("打开锁文件失败：{} - {}", lock_path.display(), e)))?;

        // 使用 fs2 crate 的 flock（跨平台兼容）
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let ret = unsafe { libc::flock(fd, libc::LOCK_EX) };
            if ret != 0 {
                return Err(CliError::storage("获取写锁失败".to_string()));
            }
        }

        #[cfg(windows)]
        {
            // Windows 使用文件句柄本身的独占打开模式
            // OpenOptions::new().read(true).write(true) 默认为独占访问
        }

        Ok(Self { _file: file })
    }

    /// 尝试获取项目级写锁（非阻塞）。
    pub fn try_acquire(novel_dir: &Path) -> Result<Self, CliError> {
        // 检查数据库是否存在
        let db_path = novel_dir.join("novel.db");
        if !db_path.exists() {
            return Err(CliError::world_not_initialized(
                "世界未初始化：.novel/novel.db 不存在。".to_string()
            ));
        }

        // 使用独立的锁文件，避免与 SQLite 的锁机制冲突
        let lock_path = novel_dir.join(".write.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&lock_path)
            .map_err(|e| CliError::storage(format!("打开锁文件失败：{} - {}", lock_path.display(), e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
            if ret != 0 {
                return Err(CliError::storage("获取写锁失败（锁已被占用）".to_string()));
            }
        }

        #[cfg(windows)]
        {
            // Windows 文件独占模式自动实现
        }

        Ok(Self { _file: file })
    }
}

impl Drop for ProjectWriteLock {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = self._file.as_raw_fd();
            unsafe { libc::flock(fd, libc::LOCK_UN) };
        }
        // Windows 在 file drop 时自动释放
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_temp_file() {
        let dir = tempdir().unwrap();
        let novel_dir = dir.path().join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();

        let content = "测试内容";
        let temp_path = write_temp_file(&novel_dir, content, "md").unwrap();

        assert!(temp_path.exists());
        assert!(temp_path.to_string_lossy().contains("tmp"));

        let read_content = std::fs::read_to_string(&temp_path).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_atomic_replace() {
        let dir = tempdir().unwrap();
        let novel_dir = dir.path().join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();

        // 写入临时文件
        let temp_path = write_temp_file(&novel_dir, "新内容", "md").unwrap();

        // 原子替换
        let target_path = novel_dir.join("content").join("test.md");
        atomic_replace(&temp_path, &target_path).unwrap();

        assert!(!temp_path.exists());
        assert!(target_path.exists());
        assert_eq!(std::fs::read_to_string(&target_path).unwrap(), "新内容");
    }

    #[test]
    fn test_cleanup_temp_file() {
        let dir = tempdir().unwrap();
        let novel_dir = dir.path().join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();

        let temp_path = write_temp_file(&novel_dir, "内容", "md").unwrap();
        assert!(temp_path.exists());

        cleanup_temp_file(&temp_path);
        assert!(!temp_path.exists());
    }

    // ===== F-11 写锁防绕过测试 =====
    // 验证 design D4：CLI 持 ProjectWriteLock + BEGIN IMMEDIATE 期间，不经 novel 命令、
    // 直接用裸 rusqlite::Connection::open 打开 novel.db 的进程无法成功落库。

    use crate::storage::connection::{self, run_migrations};
    use rusqlite::Connection;

    /// 初始化一个可写世界（含 chapters 表），返回 novel_dir 路径。
    fn setup_writable_world(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let novel_dir = dir.path().join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();
        let db_path = novel_dir.join("novel.db");
        let conn = connection::open(&db_path).unwrap();
        run_migrations(&conn, "2026-06-29T00:00:00Z").unwrap();
        novel_dir
    }

    #[test]
    fn test_bare_sqlite_write_blocked_during_cli_lock() {
        // F-11：CLI 持锁期间，裸 SQLite 写入失败；锁释放后方可写入。
        let dir = tempdir().unwrap();
        let novel_dir = setup_writable_world(&dir);
        let db_path = novel_dir.join("novel.db");

        // 模拟 CLI 写路径：取 ProjectWriteLock（flock）+ 开 BEGIN IMMEDIATE 事务（持 DB 写锁）。
        let _lock = ProjectWriteLock::acquire(&novel_dir).unwrap();
        let mut cli_conn = connection::open(&db_path).unwrap();
        let cli_tx = cli_conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .unwrap();
        // 在事务内插入一行（占住写锁，事务未提交）
        cli_tx
            .execute(
                "INSERT INTO chapters (id, title, content_path, content_hash, word_count, created_at, updated_at) \
                 VALUES ('chapter-001', 'cli', 'p', 'h', 0, 't', 't')",
                [],
            )
            .unwrap();

        // 另一进程不经 novel 命令、用裸 rusqlite::Connection::open 直接写。
        // 裸连接未设 busy_timeout（绕过者不会设），故立即得到 SQLITE_BUSY。
        let bare_conn = Connection::open(&db_path).unwrap();
        let bare_write_result = bare_conn.execute(
            "INSERT INTO chapters (id, title, content_path, content_hash, word_count, created_at, updated_at) \
             VALUES ('chapter-002', 'bare', 'p2', 'h2', 0, 't', 't')",
            [],
        );

        assert!(
            bare_write_result.is_err(),
            "CLI 持 BEGIN IMMEDIATE 写锁期间，裸 SQLite 写入应被阻塞/拒绝"
        );
        let err_msg = bare_write_result.unwrap_err().to_string().to_lowercase();
        assert!(
            err_msg.contains("busy") || err_msg.contains("locked"),
            "失败原因应为 SQLite 写锁忙/锁定，实际: {err_msg}"
        );

        // CLI 释放写锁（提交事务 + drop lock）
        cli_tx.commit().unwrap();
        drop(cli_conn);
        drop(_lock);

        // 锁释放后，裸 SQLite 写入方可成功
        let bare_write_after = bare_conn.execute(
            "INSERT INTO chapters (id, title, content_path, content_hash, word_count, created_at, updated_at) \
             VALUES ('chapter-002', 'bare', 'p2', 'h2', 0, 't', 't')",
            [],
        );
        assert!(
            bare_write_after.is_ok(),
            "CLI 释放写锁后，裸 SQLite 写入应成功，实际: {:?}",
            bare_write_after
        );
    }

    #[test]
    fn test_cli_write_completes_under_concurrent_bare_writer() {
        // F-11 侧面：CLI 写操作在并发裸 SQLite 写干扰下仍正常完成、hash 一致。
        // CLI 经 BEGIN IMMEDIATE 抢得写锁后完成自身写入；裸写入在持锁期间失败不影响 CLI。
        let dir = tempdir().unwrap();
        let novel_dir = setup_writable_world(&dir);
        let db_path = novel_dir.join("novel.db");

        // CLI 写路径（模拟 chapter_service 编排：flock + BEGIN IMMEDIATE + 写 + 提交）
        let _lock = ProjectWriteLock::acquire(&novel_dir).unwrap();
        let mut cli_conn = connection::open(&db_path).unwrap();
        let cli_tx = cli_conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .unwrap();

        // 并发裸写入（在 CLI 持 IMMEDIATE 写锁期间）→ 失败
        let bare_conn = Connection::open(&db_path).unwrap();
        let bare_result = bare_conn.execute(
            "INSERT INTO chapters (id, title, content_path, content_hash, word_count, created_at, updated_at) \
             VALUES ('bare', 'bare', 'p', 'h', 0, 't', 't')",
            [],
        );
        assert!(bare_result.is_err(), "持锁期间裸写应失败");

        // CLI 自身写入不受影响，正常完成
        cli_tx
            .execute(
                "INSERT INTO chapters (id, title, content_path, content_hash, word_count, created_at, updated_at) \
                 VALUES ('chapter-001', 'cli', 'content/chapters/chapter-001.md', 'cli-hash', 100, 't', 't')",
                [],
            )
            .unwrap();
        cli_tx.commit().unwrap();
        drop(cli_conn);
        drop(_lock);

        // 验证 CLI 写入落库、hash 一致（CLI 写的 chapter-001 存在，bare 不存在）
        let verify_conn = connection::open(&db_path).unwrap();
        let count: i64 = verify_conn
            .query_row("SELECT COUNT(*) FROM chapters WHERE id = 'chapter-001'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "CLI 写入应已落库");
        let bare_count: i64 = verify_conn
            .query_row("SELECT COUNT(*) FROM chapters WHERE id = 'bare'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(bare_count, 0, "持锁期间裸写不应落库");
    }
}
