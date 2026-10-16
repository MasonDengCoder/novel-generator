// 契约测试：验证 `novel verify integrity` 命令输出符合 CLI Contract（s07-archive-and-export）。
//
// 覆盖任务：
// - 6.1-6.5（s04-error-render-unification）：全通过（0）、hash 失配（21）、悬空引用（70）、
//   stale 裁决（5）、未初始化（3）、多失败项取最严重退出码且 details 承载全部失败项。
//
// hard 违规路径（hash 失配/悬空/stale/未初始化）断言 stdout 为**结构化错误对象**（code/exitCode/
// details），错误对象经 `lib::run()`→`render_error` 渲染，不复用 command 层直出报告 JSON 的旧行为。
// 成功路径（全通过、人读摘要）保持报告 JSON / 人读摘要到 stdout。

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

// 全通过（退出码 0），JSON 报告结构正确。
#[test]
fn test_verify_integrity_all_pass() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert!(json["verifiedAt"].is_string());
    assert_eq!(json["hashIntegrity"]["passed"], true);
    assert_eq!(json["referenceIntegrity"]["passed"], true);
    assert_eq!(json["stateConsistency"]["passed"], true);
    assert_eq!(json["hashIntegrity"]["mismatches"].as_array().unwrap().len(), 0);
    assert_eq!(json["hashIntegrity"]["missingFiles"].as_array().unwrap().len(), 0);
    assert_eq!(json["referenceIntegrity"]["danglingReferences"].as_array().unwrap().len(), 0);
    assert_eq!(json["stateConsistency"]["inconsistencies"].as_array().unwrap().len(), 0);
}

// hash 失配（退出码 21）——错误对象形态（s04）。
#[test]
fn test_verify_integrity_hash_mismatch() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    std::fs::create_dir_all(dir.path().join(".novel/content/chapters")).unwrap();
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "原始正文").unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 4, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![sha256("原始正文")],
    ).unwrap();

    // 外部篡改正文 → hash 失配
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "外部篡改").unwrap();

    let output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(5)
        .get_output()
        .stdout
        .clone();

    // hard 违规：stdout 输出结构化错误对象（非报告 JSON），经 render_error 渲染。
    let err: Value = serde_json::from_slice(&output).expect("hard 违规 stdout 应为错误对象 JSON");
    assert_eq!(err["errorId"], "content_modified_externally");
    assert_eq!(err["exitCode"], 5);
    // details 承载完整报告失败详情（同成功路径结构），含 hashIntegrity.mismatches 失配项。
    assert_eq!(err["details"]["hashIntegrity"]["passed"], false);
    let mismatches = err["details"]["hashIntegrity"]["mismatches"].as_array().unwrap();
    assert_eq!(mismatches.len(), 1);
    assert_eq!(mismatches[0]["chapterId"], "chapter-001");
    assert!(mismatches[0]["storedHash"].is_string());
    assert!(mismatches[0]["actualHash"].is_string());
    // V2 错误信封顶层 schemaVersion 为 error-protocol/v2（非报告的 schemaVersion）。
    assert_eq!(err["schemaVersion"], "error-protocol/v2");
}

// 悬空引用（退出码 70）——错误对象形态（s04）。
#[test]
fn test_verify_integrity_dangling_reference() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    // 创建正文文件（hash 校验需通过，让引用悬空成为唯一的失败项）。
    std::fs::create_dir_all(dir.path().join(".novel/content/chapters")).unwrap();
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "正文").unwrap();
    let real_hash = sha256("正文");
    // 直接插入引用不存在卷 999 的章节（关闭外键约束以模拟数据损坏）
    conn.execute("PRAGMA foreign_keys = OFF;", []).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 2, 'approved', 999, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![real_hash],
    ).unwrap();

    let output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(6)
        .get_output()
        .stdout
        .clone();

    let err: Value = serde_json::from_slice(&output).expect("hard 违规 stdout 应为错误对象 JSON");
    assert_eq!(err["errorId"], "not_found");
    assert_eq!(err["exitCode"], 6);
    assert_eq!(err["details"]["referenceIntegrity"]["passed"], false);
    let dangling = err["details"]["referenceIntegrity"]["danglingReferences"].as_array().unwrap();
    assert_eq!(dangling.len(), 1);
    assert_eq!(dangling[0]["entityType"], "chapter");
    assert_eq!(dangling[0]["referencedEntityType"], "volume");
}

// stale 裁决（退出码 5）——错误对象形态（s04）。
#[test]
fn test_verify_integrity_stale_verdict() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    std::fs::create_dir_all(dir.path().join(".novel/content/chapters")).unwrap();
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "正文").unwrap();
    let real_hash = sha256("正文");
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 2, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![real_hash],
    ).unwrap();
    // approved 章节绑定一个旧 hash 的裁决 → stale
    conn.execute(
        "INSERT INTO verdicts (chapter_id, verdict_type, node_id, achieved, content_hash, created_at) \
         VALUES ('chapter-001', 'mainplot-landing', 'node-001', 1, 'old-hash', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();

    let output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(5)
        .get_output()
        .stdout
        .clone();

    let err: Value = serde_json::from_slice(&output).expect("hard 违规 stdout 应为错误对象 JSON");
    assert_eq!(err["errorId"], "state_conflict");
    assert_eq!(err["exitCode"], 5);
    assert_eq!(err["details"]["stateConsistency"]["passed"], false);
    let inc = err["details"]["stateConsistency"]["inconsistencies"].as_array().unwrap();
    assert_eq!(inc.len(), 1);
    assert_eq!(inc[0]["type"], "stale_verdict");
    assert_eq!(inc[0]["chapterId"], "chapter-001");
}

// 未初始化（退出码 3）——错误对象形态（s04，未初始化路径已走 render_error，补断言错误对象）。
#[test]
fn test_verify_integrity_uninitialized() {
    let dir = tempdir().unwrap();

    let output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let err: Value = serde_json::from_slice(&output).expect("未初始化 stdout 应为错误对象 JSON");
    assert_eq!(err["errorId"], "world_not_initialized");
    assert_eq!(err["exitCode"], 3);
    // 未初始化路径不产生 IntegrityReport，details 不承载报告。
    assert!(err.get("details").is_none() || err["details"].is_null());
}

// 人读摘要输出（无 --json）——成功路径不变。
#[test]
fn test_verify_integrity_human_readable() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["verify", "integrity", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let s = String::from_utf8(output).unwrap();
    assert!(s.contains("完整性验证报告"));
    assert!(s.contains("hash 完整性"));
    assert!(s.contains("引用完整性"));
    assert!(s.contains("状态一致性"));
}

// 多失败项：details 承载全部失败项（s04 specs ADDED 多失败项 scenario）。
// 同时存在 hash 失配与悬空引用：verify_exit_mapping 先判 hash 分项，故 hash 失配(21) 为退出码；
// details 同时承载 hashIntegrity 与 referenceIntegrity 失败项（不止最严重项）。
#[test]
fn test_verify_integrity_multi_failure_carries_all_in_details() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    std::fs::create_dir_all(dir.path().join(".novel/content/chapters")).unwrap();
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "原始正文").unwrap();
    // 悬空引用（引用不存在的卷 999）+ hash 失配（篡改正文）同时存在。
    conn.execute("PRAGMA foreign_keys = OFF;", []).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', ?1, 4, 'approved', 999, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        rusqlite::params![sha256("原始正文")],
    ).unwrap();
    // 外部篡改正文 → hash 失配（与悬空引用并存）
    std::fs::write(dir.path().join(".novel/content/chapters/chapter-001.md"), "外部篡改").unwrap();

    // verify_exit_mapping 先判 hash 分项，故 hash 失配(21) 为退出码与错误码。
    let output = novel()
        .args(["verify", "integrity", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(5)
        .get_output()
        .stdout
        .clone();

    let err: Value = serde_json::from_slice(&output).expect("hard 违规 stdout 应为错误对象 JSON");
    assert_eq!(err["exitCode"], 5);
    assert_eq!(err["errorId"], "content_modified_externally");
    // details 承载全部失败项：hash 失配项在 hashIntegrity，悬空引用项在 referenceIntegrity。
    assert_eq!(err["details"]["hashIntegrity"]["passed"], false);
    assert_eq!(
        err["details"]["hashIntegrity"]["mismatches"].as_array().unwrap().len(),
        1,
        "details 应承载 hash 失配项（不止最严重项）"
    );
    assert_eq!(err["details"]["referenceIntegrity"]["passed"], false);
    assert_eq!(
        err["details"]["referenceIntegrity"]["danglingReferences"].as_array().unwrap().len(),
        1,
        "details 应承载悬空引用项（不止最严重项）"
    );
}
