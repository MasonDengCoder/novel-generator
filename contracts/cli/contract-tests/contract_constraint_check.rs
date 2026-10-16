// Contract test: novel constraint check 命令
// 验证 s03-rule-checking-engine 的核心功能

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// 辅助函数：初始化测试世界并返回临时目录
fn setup_world(name: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("init")
        .arg(name)
        .assert()
        .success();
    temp_dir
}

/// 辅助函数：创建章节并返回章节 ID
fn create_chapter(temp_dir: &TempDir, title: &str) -> String {
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(temp_dir)
        .arg("chapter")
        .arg("create")
        .arg(title)
        .output()
        .unwrap();

    assert!(output.status.success());

    // 从输出中提取章节 ID（格式：ID: chapter-001）
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find(|line| line.starts_with("ID:"))
        .unwrap()
        .split(':')
        .nth(1)
        .unwrap()
        .trim()
        .to_string()
}

/// 辅助函数：获取章节的 contentHash
fn get_chapter_hash(temp_dir: &TempDir, chapter_id: &str) -> String {
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(temp_dir)
        .arg("chapter")
        .arg("get")
        .arg(chapter_id)
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    json["contentHash"].as_str().unwrap().to_string()
}

#[test]
fn test_constraint_check_write_success() {
    let temp_dir = setup_world("test-check-write");
    let chapter_id = create_chapter(&temp_dir, "测试章节");
    let content_hash = get_chapter_hash(&temp_dir, &chapter_id);

    // 创建检查结果 JSON 文件
    let check_json = format!(
        r#"{{
  "contentHash": "{}",
  "checks": [
    {{
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "时间线一致性",
      "severity": "hard",
      "passed": true,
      "description": "测试通过",
      "location": null
    }}
  ]
}}"#,
        content_hash
    );

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    // 批量写入检查结果
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .assert()
        .success()
        .stdout(predicate::str::is_empty()); // 人读模式静默成功
}

#[test]
fn test_constraint_check_hash_mismatch() {
    let temp_dir = setup_world("test-check-hash-mismatch");
    let chapter_id = create_chapter(&temp_dir, "测试章节");

    // 使用错误的 contentHash
    let check_json = r#"{
  "contentHash": "wronghash123456",
  "checks": [
    {
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "测试规则",
      "severity": "hard",
      "passed": true,
      "description": null,
      "location": null
    }
  ]
}"#;

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    // 应该返回退出码 5，错误码 content_hash_mismatch
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("content_hash_mismatch"))
        .stderr(predicate::str::contains("contentHash 不匹配"));
}

#[test]
fn test_constraint_check_list_json() {
    let temp_dir = setup_world("test-check-list");
    let chapter_id = create_chapter(&temp_dir, "测试章节");
    let content_hash = get_chapter_hash(&temp_dir, &chapter_id);

    // 写入检查结果
    let check_json = format!(
        r#"{{
  "contentHash": "{}",
  "checks": [
    {{
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "规则1",
      "severity": "hard",
      "passed": true,
      "description": "描述1",
      "location": null
    }},
    {{
      "constraintType": "character",
      "constraintScope": "chapter:{}",
      "constraintName": "规则2",
      "severity": "warning",
      "passed": false,
      "description": "描述2",
      "location": "第3段"
    }}
  ]
}}"#,
        content_hash, chapter_id
    );

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .assert()
        .success();

    // 查询检查结果（JSON 格式）
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--list")
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());

    // 验证 JSON 结构
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["chapterId"], chapter_id);
    assert_eq!(json["currentContentHash"], content_hash);

    let checks = json["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 2);

    // 验证第一条检查结果
    assert_eq!(checks[0]["constraintType"], "plot");
    assert_eq!(checks[0]["constraintScope"], "world");
    assert_eq!(checks[0]["constraintName"], "规则1");
    assert_eq!(checks[0]["severity"], "hard");
    assert_eq!(checks[0]["passed"], true);
    assert_eq!(checks[0]["stale"], false);

    // 验证第二条检查结果
    assert_eq!(checks[1]["constraintType"], "character");
    assert_eq!(checks[1]["constraintName"], "规则2");
    assert_eq!(checks[1]["passed"], false);
    assert_eq!(checks[1]["location"], "第3段");
}

#[test]
fn test_constraint_check_stale_detection() {
    let temp_dir = setup_world("test-check-stale");
    let chapter_id = create_chapter(&temp_dir, "测试章节");
    let old_hash = get_chapter_hash(&temp_dir, &chapter_id);

    // 写入初始检查结果
    let check_json = format!(
        r#"{{
  "contentHash": "{}",
  "checks": [
    {{
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "测试规则",
      "severity": "hard",
      "passed": true,
      "description": null,
      "location": null
    }}
  ]
}}"#,
        old_hash
    );

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .assert()
        .success();

    // 修改章节正文
    let content_file = temp_dir.path().join("new-content.txt");
    fs::write(&content_file, "新的章节内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("chapter")
        .arg("update")
        .arg(&chapter_id)
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success();

    // 查询检查结果，应该标记为 stale
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--list")
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let checks = json["checks"].as_array().unwrap();

    // 验证检查结果被标记为 stale
    assert_eq!(checks[0]["stale"], true);
    assert_ne!(checks[0]["contentHash"], json["currentContentHash"]);
}

#[test]
fn test_constraint_check_verify_hash_fails_on_stale() {
    let temp_dir = setup_world("test-check-verify-hash");
    let chapter_id = create_chapter(&temp_dir, "测试章节");
    let old_hash = get_chapter_hash(&temp_dir, &chapter_id);

    // 写入检查结果
    let check_json = format!(
        r#"{{
  "contentHash": "{}",
  "checks": [
    {{
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "测试规则",
      "severity": "hard",
      "passed": true,
      "description": null,
      "location": null
    }}
  ]
}}"#,
        old_hash
    );

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .assert()
        .success();

    // 修改章节正文
    let content_file = temp_dir.path().join("new-content.txt");
    fs::write(&content_file, "新的章节内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("chapter")
        .arg("update")
        .arg(&chapter_id)
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success();

    // 使用 --verify-hash 查询，应该返回退出码 5
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--list")
        .arg("--verify-hash")
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("stale_rule_checks"))
        .stderr(predicate::str::contains("检查结果已过期"));
}

#[test]
fn test_recheck_replaces_stale_set_and_allows_review() {
    let temp_dir = setup_world("test-check-repair-recheck");

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["volume", "create", "第一卷", "--order", "1"])
        .assert()
        .success();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["mainplot", "node", "create", "1", "--title", "主线节点"])
        .assert()
        .success();

    let create_output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["chapter", "create", "第一章", "--volume", "1", "--json"])
        .output()
        .unwrap();
    assert!(create_output.status.success());
    let create_json: serde_json::Value = serde_json::from_slice(&create_output.stdout).unwrap();
    let chapter_id = create_json["id"].as_str().unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["chapter", "status", chapter_id, "drafting"])
        .assert()
        .success();

    let content_v1 = temp_dir.path().join("content-v1.md");
    fs::write(&content_v1, "第一版正文，主线线索尚待修复。").unwrap();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("chapter")
        .arg("update")
        .arg(chapter_id)
        .arg("--content")
        .arg(&content_v1)
        .assert()
        .success();

    let hash_v1 = get_chapter_hash(&temp_dir, chapter_id);
    let checks_v1 = temp_dir.path().join("checks-v1.json");
    fs::write(
        &checks_v1,
        format!(
            r#"{{"contentHash":"{hash_v1}","checks":[{{"constraintType":"plot","constraintScope":"world","constraintName":"旧检查","severity":"hard","passed":true}}]}}"#
        ),
    )
    .unwrap();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(chapter_id)
        .arg("--input")
        .arg(&checks_v1)
        .assert()
        .success();

    let content_v2 = temp_dir.path().join("content-v2.md");
    fs::write(&content_v2, "修复后的正文，主角确认主线线索已经落地。").unwrap();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("chapter")
        .arg("update")
        .arg(chapter_id)
        .arg("--content")
        .arg(&content_v2)
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["constraint", "check", "--chapter", chapter_id, "--list", "--verify-hash"])
        .assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("stale_rule_checks"));

    let hash_v2 = get_chapter_hash(&temp_dir, chapter_id);
    let checks_v2 = temp_dir.path().join("checks-v2.json");
    fs::write(
        &checks_v2,
        format!(
            r#"{{"contentHash":"{hash_v2}","checks":[{{"constraintType":"plot","constraintScope":"world","constraintName":"新检查","severity":"hard","passed":true}}]}}"#
        ),
    )
    .unwrap();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(chapter_id)
        .arg("--input")
        .arg(&checks_v2)
        .assert()
        .success();

    let verify_output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args([
            "constraint",
            "check",
            "--chapter",
            chapter_id,
            "--list",
            "--verify-hash",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        verify_output.status.success(),
        "recheck should remove stale results: {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
    let verify_json: serde_json::Value = serde_json::from_slice(&verify_output.stdout).unwrap();
    let checks = verify_json["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0]["constraintName"], "新检查");
    assert_eq!(checks[0]["contentHash"], hash_v2);
    assert_eq!(checks[0]["stale"], false);

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args([
            "verdict",
            "set-mainplot-landing",
            chapter_id,
            "--node-id",
            "mp-1-001",
            "--achieved",
            "1",
        ])
        .assert()
        .success();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .args(["chapter", "status", chapter_id, "review", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"review\""));
}

#[test]
fn test_constraint_check_chapter_not_found() {
    let temp_dir = setup_world("test-check-not-found");

    let check_json = r#"{
  "contentHash": "somehash",
  "checks": [
    {
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "测试规则",
      "severity": "hard",
      "passed": true,
      "description": null,
      "location": null
    }
  ]
}"#;

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    // 章节不存在，应该返回退出码 6
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg("chapter-999")
        .arg("--input")
        .arg(&input_file)
        .assert()
        .failure()
        .code(6)
        .stderr(predicate::str::contains("chapter_not_found"))
        .stderr(predicate::str::contains("章节"));
}

// ===== D2: check-write 契约补全（schemaVersion / invalid_severity / invalid_input）=====

#[test]
fn test_constraint_check_write_success_json_schema_version() {
    // --json 成功输出含 schemaVersion/success/message（对齐 constraint-check-write.schema.json）
    let temp_dir = setup_world("test-check-write-json");
    let chapter_id = create_chapter(&temp_dir, "测试章节");
    let content_hash = get_chapter_hash(&temp_dir, &chapter_id);

    let check_json = format!(
        r#"{{
  "contentHash": "{}",
  "checks": [
    {{
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "时间线一致性",
      "severity": "warning",
      "passed": true
    }}
  ]
}}"#,
        content_hash
    );

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let golden: serde_json::Value = serde_json::from_str(include_str!(
        "../examples/constraint-check-write-ok.golden.json"
    ))
    .unwrap();
    assert_eq!(json["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(json, golden);
    assert_eq!(json.as_object().unwrap().keys().count(), 3);
}

#[test]
fn test_constraint_check_write_invalid_severity() {
    // 检查结果 severity 取值非法（INVALID），应返回退出码 2（invalid_input），而非 70
    let temp_dir = setup_world("test-check-invalid-sev");
    let chapter_id = create_chapter(&temp_dir, "测试章节");
    let content_hash = get_chapter_hash(&temp_dir, &chapter_id);

    let check_json = format!(
        r#"{{
  "contentHash": "{}",
  "checks": [
    {{
      "constraintType": "plot",
      "constraintScope": "world",
      "constraintName": "测试",
      "severity": "INVALID",
      "passed": true
    }}
  ]
}}"#,
        content_hash
    );

    let input_file = temp_dir.path().join("check-input.json");
    fs::write(&input_file, check_json).unwrap();

    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .arg("--json")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let error: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["status"], "failed");
    assert_eq!(error["errorId"], "invalid_input");
    assert_eq!(error["category"], "invalid_input");
    assert_eq!(error["exitCode"], 2);
    assert!(error["schemaVersion"].is_string());
    assert!(error["message"].as_str().unwrap().contains("severity"));
}

#[test]
fn test_constraint_check_write_invalid_json_input() {
    // 输入 JSON 格式错误，应返回退出码 2（invalid_input）
    let temp_dir = setup_world("test-check-invalid-json");
    let chapter_id = create_chapter(&temp_dir, "测试章节");

    let input_file = temp_dir.path().join("bad-input.json");
    fs::write(&input_file, "not a valid json {{{").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("constraint")
        .arg("check")
        .arg("--chapter")
        .arg(&chapter_id)
        .arg("--input")
        .arg(&input_file)
        .arg("--json")
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("invalid_input"));
}
