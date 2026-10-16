// 契约测试：验证 `novel chapter list --json` 输出符合 JSON schema（task 10.4）。

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn test_chapter_list_json_schema() {
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

    // 创建多个章节
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第二章")
        .assert()
        .success();

    // 列出章节
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");

    // 验证 JSON schema
    assert!(json.get("schemaVersion").is_some(), "必须包含 schemaVersion");
    assert!(json.get("chapters").is_some(), "必须包含 chapters");

    let chapters = json["chapters"].as_array().unwrap();
    assert_eq!(chapters.len(), 2);

    // 验证第一个章节
    let ch1 = &chapters[0];
    assert!(ch1.get("id").is_some());
    assert!(ch1.get("title").is_some());
    assert!(ch1.get("wordCount").is_some());
    assert!(ch1.get("createdAt").is_some());
    assert!(ch1.get("updatedAt").is_some());

    assert_eq!(ch1["id"].as_str().unwrap(), "chapter-001");
    assert_eq!(ch1["title"].as_str().unwrap(), "第一章");
}

#[test]
fn test_chapter_list_empty() {
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

    // 列出章节（空列表）
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");

    let chapters = json["chapters"].as_array().unwrap();
    assert_eq!(chapters.len(), 0);
}

#[test]
fn test_chapter_list_human_output() {
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

    // 列出章节（人读输出）
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("chapter-001"))
        .stdout(predicate::str::contains("测试章节"));
}

#[test]
fn test_chapter_list_human_empty() {
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

    // 列出章节（空列表，人读输出）
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("暂无章节"));
}
