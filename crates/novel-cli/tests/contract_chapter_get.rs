// 契约测试：验证 `novel chapter get --json` 输出符合 JSON schema（task 10.3）。

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_chapter_get_json_schema() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 创建章节
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .assert()
        .success();

    // 写入正文
    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "章节正文内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success();

    // 读取章节
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-001")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");

    // 验证 JSON schema
    assert!(json.get("schemaVersion").is_some(), "必须包含 schemaVersion");
    assert!(json.get("id").is_some(), "必须包含 id");
    assert!(json.get("title").is_some(), "必须包含 title");
    assert!(json.get("content").is_some(), "必须包含 content");
    assert!(json.get("contentHash").is_some(), "必须包含 contentHash");
    assert!(json.get("wordCount").is_some(), "必须包含 wordCount");
    assert!(json.get("status").is_some(), "必须包含 status");
    assert!(json.get("createdAt").is_some(), "必须包含 createdAt");
    assert!(json.get("updatedAt").is_some(), "必须包含 updatedAt");

    // 验证字段值
    assert_eq!(json["id"].as_str().unwrap(), "chapter-001");
    assert_eq!(json["title"].as_str().unwrap(), "第一章");
    assert_eq!(json["content"].as_str().unwrap(), "章节正文内容");
    assert!(json["wordCount"].as_i64().unwrap() > 0);
    assert_eq!(json["status"].as_str().unwrap(), "outline");
}

#[test]
fn test_chapter_get_not_found() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 读取不存在的章节应失败
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-999")
        .assert()
        .failure()
        .code(6); // chapter_not_found → not_found 类别退出码
}

#[test]
fn test_chapter_get_human_output() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目并创建章节
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("测试章节")
        .assert()
        .success();

    // 读取章节（人读输出）
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-001")
        .assert()
        .success()
        .stdout(predicate::str::contains("章节 ID: chapter-001"))
        .stdout(predicate::str::contains("标题: 测试章节"));
}
