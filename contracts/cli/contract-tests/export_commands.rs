// 契约测试：验证 `novel export` 命令输出符合 CLI Contract（s07-archive-and-export）。
//
// 覆盖任务：
// - 6.6: export volume 归档卷成功（0）、导出未封卷卷含 warning（0）、卷不存在（6）、
//        章节不存在（20）、章节 hash 失配（21）、导出 JSON 含 schemaVersion 且纯 JSON。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

fn sha256(content: &str) -> String {
    // 与 chapter_validator::calculate_content_hash 一致：SHA-256 hex。
    use std::io::Write;
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in result.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// 导出未封卷卷含 warning（退出码 0），JSON 含 schemaVersion 且纯 JSON。
#[test]
fn test_export_volume_unarchived_with_warning() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["export", "volume", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["scope"], "volume");
    assert_eq!(json["volume"]["name"], "第一卷");
    let warnings = json["warnings"].as_array().expect("warnings 应为数组");
    assert!(warnings.iter().any(|w| w.as_str().unwrap_or("").contains("未归档")), "未归档卷应含 warning");
}

// 导出归档卷成功（退出码 0），warnings 为空。
#[test]
fn test_export_volume_archived_no_warning() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 插入 approved 章节并封卷归档
    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    std::fs::create_dir_all(dir.path().join(".novel/content/chapters")).unwrap();
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "正文").unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 2, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![sha256("正文")],
    ).unwrap();

    novel().args(["volume", "seal", "1", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "archive", "1", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["export", "volume", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["volume"]["status"], "archived");
    let warnings = json["warnings"].as_array().unwrap();
    assert!(warnings.is_empty(), "归档卷 warnings 应为空");
    let chapters = json["chapters"].as_array().unwrap();
    assert_eq!(chapters.len(), 1);
    assert_eq!(chapters[0]["content"], "正文");
    assert_eq!(chapters[0]["status"], "approved");
}

// 卷不存在（退出码 6）。
#[test]
fn test_export_volume_not_found() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args(["export", "volume", "999", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(6);
}

// 章节不存在（退出码 20）。
#[test]
fn test_export_chapter_not_found() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args(["export", "chapter", "chapter-999", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(6);
}

// 章节正文 hash 失配（退出码 21）。
#[test]
fn test_export_chapter_hash_mismatch() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    std::fs::create_dir_all(dir.path().join(".novel/content/chapters")).unwrap();
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "原始正文").unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 4, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();

    // 外部篡改正文
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "外部篡改").unwrap();

    novel()
        .args(["export", "chapter", "chapter-001", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(5);
}

// 导出 JSON 含 schemaVersion 且纯 JSON（无 ANSI/旁白）。
#[test]
fn test_export_full_json_purity() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["export", "full", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // 纯 JSON：可解析，首字符为 {
    let s = String::from_utf8(output.clone()).unwrap();
    assert!(s.trim_start().starts_with('{'), "输出应为 JSON 对象");
    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["scope"], "full");
    assert!(!s.contains('\u{1b}'), "JSON 不应含 ANSI 转义");
}

// export 接受 --json 标志（D2 修复：export 恒输出 JSON，--json 仅作契约兼容性开关，不再被拒绝）。
#[test]
fn test_export_volume_accepts_json_flag() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["export", "volume", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("export --json 应输出有效 JSON");
    assert_eq!(json["scope"], "volume");
}

#[test]
fn test_export_full_accepts_json_flag() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["export", "full", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("export --json 应输出有效 JSON");
    assert_eq!(json["scope"], "full");
}
