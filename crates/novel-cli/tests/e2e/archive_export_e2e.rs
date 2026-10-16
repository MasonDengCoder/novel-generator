// s07 端到端测试：init → volume create → chapter (approved) → volume seal（真实检查）→
// 监测报告 → volume archive → export volume → verify integrity，断言各步退出码与 JSON 结构。
//
// 覆盖任务 9.3。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

fn sha256(content: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let mut s = String::with_capacity(64);
    for b in result.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[test]
fn test_s07_full_lifecycle_e2e() {
    let dir = tempdir().unwrap();

    // 1. init
    novel()
        .args(["init", "test-world", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 2. volume create
    novel()
        .args(["volume", "create", "第一卷", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 3. chapter（fixture approved，含正文文件与正确 hash）
    let novel_dir = dir.path().join(".novel");
    std::fs::create_dir_all(novel_dir.join("content/chapters")).unwrap();
    std::fs::write(novel_dir.join("content/chapters/chapter-001.md"), "第一章正文内容").unwrap();
    let db_path = novel_dir.join("novel.db");
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
             VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 8, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            rusqlite::params![sha256("第一章正文内容")],
        ).unwrap();
    }

    // 4. volume seal（真实检查通过，推进到 sealing，退出码 0）
    novel()
        .args(["volume", "seal", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 验证卷状态为 sealing
    {
        let conn = Connection::open(&db_path).unwrap();
        let status: String = conn.query_row("SELECT status FROM volumes WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(status, "sealing");
    }

    // 5. volume archive（生成监测报告，推进到 archived，退出码 0）
    novel()
        .args(["volume", "archive", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 验证卷状态为 archived 且监测报告文件已生成
    {
        let conn = Connection::open(&db_path).unwrap();
        let status: String = conn.query_row("SELECT status FROM volumes WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(status, "archived");
        let report_path: String = conn
            .query_row("SELECT monitoring_report_path FROM volumes WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert!(report_path.contains("volume-1.md"), "报告路径应指向 volume-1.md");
        assert!(novel_dir.join(&report_path).exists(), "监测报告文件应存在");
    }

    // 6. export volume（归档卷，退出码 0，warnings 为空）
    let export_output = novel()
        .args(["export", "volume", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export_json: Value = serde_json::from_slice(&export_output).expect("export volume 应输出有效 JSON");
    assert_eq!(export_json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(export_json["scope"], "volume");
    assert_eq!(export_json["volume"]["status"], "archived");
    assert!(export_json["warnings"].as_array().unwrap().is_empty());
    assert_eq!(export_json["chapters"].as_array().unwrap().len(), 1);

    // 7. verify integrity（全通过，退出码 0）
    let verify_output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let verify_json: Value = serde_json::from_slice(&verify_output).expect("verify integrity 应输出有效 JSON");
    assert_eq!(verify_json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(verify_json["hashIntegrity"]["passed"], true);
    assert_eq!(verify_json["referenceIntegrity"]["passed"], true);
    assert_eq!(verify_json["stateConsistency"]["passed"], true);
}

#[test]
fn test_s07_export_full_and_verify_after_archived() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let novel_dir = dir.path().join(".novel");
    std::fs::create_dir_all(novel_dir.join("content/chapters")).unwrap();
    std::fs::write(novel_dir.join("content/chapters/chapter-001.md"), "正文").unwrap();
    let db_path = novel_dir.join("novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 2, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![sha256("正文")],
    ).unwrap();

    novel().args(["volume", "seal", "1", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "archive", "1", "--path"]).arg(dir.path()).assert().success();

    // export full（含一个归档卷，warnings 为空，退出码 0）
    let full_output = novel()
        .args(["export", "full", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let full_json: Value = serde_json::from_slice(&full_output).expect("export full 应输出有效 JSON");
    assert_eq!(full_json["scope"], "full");
    assert_eq!(full_json["volumes"].as_array().unwrap().len(), 1);
    assert!(full_json["warnings"].as_array().unwrap().is_empty());

    // verify integrity（全通过）
    novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success();
}
