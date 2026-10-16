// 契约测试：验证 `novel chapter update` 的退出码和错误对象（task 10.2）。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_chapter_update_success() {
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
        .arg("第一章")
        .assert()
        .success();

    // 写入正文
    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "测试正文内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&content_file)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_chapter_update_success_json_matches_contract() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    Command::cargo_bin("novel")
        .unwrap()
        .args(["init", "test-project", "--path"])
        .arg(project_dir)
        .assert()
        .success();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .args(["chapter", "create", "第一章"])
        .assert()
        .success();

    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "测试正文内容").unwrap();
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .args(["chapter", "update", "chapter-001", "--content"])
        .arg(&content_file)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&output.stdout).expect("stdout 应为单一合法 JSON");
    let golden: Value = serde_json::from_str(include_str!(
        "../../../contracts/cli/examples/chapter-update-ok.golden.json"
    ))
    .unwrap();
    assert_eq!(actual["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(actual["schemaVersion"], golden["schemaVersion"]);
    assert_eq!(actual["id"], golden["id"]);
    assert!(actual["contentHash"].as_str().unwrap().len() >= 64);
    assert!(actual["wordCount"].as_i64().unwrap() > 0);
    assert!(!actual["updatedAt"].as_str().unwrap().is_empty());
    assert_eq!(actual.as_object().unwrap().keys().count(), 5);
}

#[test]
fn test_chapter_update_chapter_not_found() {
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

    // 更新不存在的章节
    let content_file = dir.path().join("content.md");
    fs::write(&content_file, "内容").unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-999")
        .arg("--content")
        .arg(&content_file)
        .arg("--json")
        .assert()
        .failure()
        .code(6)
        .stdout(predicate::function(|stdout: &str| {
            let value: Value = serde_json::from_str(stdout).unwrap();
            value["status"] == "failed"
                && value["errorId"] == "chapter_not_found"
                && value["exitCode"] == 6
        }));
}

#[test]
fn test_chapter_update_file_not_found() {
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
        .arg("第一章")
        .assert()
        .success();

    // 更新章节，使用不存在的文件
    let nonexistent = dir.path().join("nonexistent.md");

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&nonexistent)
        .arg("--json")
        .assert()
        .failure()
        .code(6)
        .stdout(predicate::function(|stdout: &str| {
            let value: Value = serde_json::from_str(stdout).unwrap();
            value["status"] == "failed"
                && value["errorId"] == "file_not_found"
                && value["exitCode"] == 6
        }));
}

#[test]
fn test_chapter_update_invalid_utf8() {
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
        .arg("第一章")
        .assert()
        .success();

    // 创建非 UTF-8 文件
    let binary_file = dir.path().join("binary.dat");
    fs::write(&binary_file, &[0xFF, 0xFE, 0xFD]).unwrap();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("update")
        .arg("chapter-001")
        .arg("--content")
        .arg(&binary_file)
        .arg("--json")
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::function(|stdout: &str| {
            let value: Value = serde_json::from_str(stdout).unwrap();
            value["status"] == "failed"
                && value["errorId"] == "invalid_content"
                && value["exitCode"] == 2
        }));
}
