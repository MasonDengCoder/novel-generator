// chapter context 命令契约测试：chapter-context-packet 切片。
//
// 验证（黑盒 CLI，对齐 spec scenario）：
// - 三 view（writer/verifier/repair）渲染退出 0，JSON 顶层含 schemaVersion/
//   chapterId/view/budget/fragments/audit/overflow，每个 fragment 含 sourceRefs，
//   无数据层 omitted 记录存在（reason="no_data_backing_table"）。
// - 预算超限：--budget 1 返回非零退出码 + context_budget_exceeded + audit.failures
//   非空（部分 packet）。
// - 阻断场景：受控 view 非法返回 2+usage、缺 --view 返回 2、--budget 非正返回 2、
//   章节不存在返回 6+chapter_not_found、世界未初始化返回 3+world_not_initialized。
//
// 测试通过 crates/novel-cli/Cargo.toml 的 [[test]] 目标纳入 `cargo test`。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

/// 建世界 + chapter-001 + 写正文（最小环境，验证结构契约；P1 数据在 e2e 测试补）。
fn setup_min_world() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let path = dir.path();
    novel().args(["init", "demo", "--path"]).arg(path).assert().success();

    let content_file = path.join("content.md");
    std::fs::write(&content_file, "# 第一章\n\n这是正文内容。").unwrap();
    novel()
        .args(["chapter", "create", "第一章", "--path"])
        .arg(path)
        .assert()
        .success();
    novel()
        .args(["chapter", "update", "chapter-001", "--content"])
        .arg(&content_file)
        .args(["--path"])
        .arg(path)
        .assert()
        .success();
    dir
}

/// 断言 JSON 顶层契约结构（schemaVersion/chapterId/view/budget/fragments/audit/overflow）。
fn assert_packet_structure(actual: &Value, expected_chapter: &str, expected_view: &str) {
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["chapterId"].as_str().unwrap(), expected_chapter);
    assert_eq!(actual["view"].as_str().unwrap(), expected_view);
    assert!(actual["budget"]["limit"].is_i64());
    assert!(actual["budget"]["used"].is_i64());
    assert!(actual["fragments"].is_array());
    assert!(actual["audit"]["compressed"].is_array());
    assert!(actual["audit"]["failures"].is_array());
    assert!(actual["overflow"]["omitted"].is_array());
    // 每个 fragment 含 id/type/priority/sourceRefs/content 且 sourceRefs 为非空数组。
    for frag in actual["fragments"].as_array().unwrap() {
        assert!(frag["id"].is_string(), "fragment 缺 id");
        assert!(frag["type"].is_string(), "fragment 缺 type");
        assert!(frag["priority"].is_string(), "fragment 缺 priority");
        let pri = frag["priority"].as_str().unwrap();
        assert!(pri == "P0" || pri == "P1", "priority 非法: {}", pri);
        let refs = frag["sourceRefs"].as_array().expect("fragment 缺 sourceRefs");
        assert!(!refs.is_empty(), "fragment sourceRefs 为空: {}", frag["id"]);
        // sourceRefs 不含 rowid 或内部路径（不含 '/'）。
        for r in refs {
            let s = r.as_str().unwrap();
            assert!(!s.contains('/'), "sourceRefs 含路径: {}", s);
        }
        assert!(frag["content"].is_string(), "fragment 缺 content");
    }
}

/// 断言无数据层 omitted 记录存在（章契约/Snapshot 无表）。
fn assert_omitted_no_data_layers(actual: &Value) {
    let omitted = actual["overflow"]["omitted"].as_array().unwrap();
    assert!(
        !omitted.is_empty(),
        "无数据层应至少有一条 omitted 记录"
    );
    let has_no_data = omitted.iter().any(|o| {
        o["reason"].as_str().map_or(false, |r| r == "no_data_backing_table")
    });
    assert!(has_no_data, "应含 reason=no_data_backing_table 的 omitted 记录");
}

#[test]
fn chapter_context_writer_view_json() {
    let dir = setup_min_world();
    let out = novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--budget", "60000", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");
    assert_packet_structure(&actual, "chapter-001", "writer");
    assert_omitted_no_data_layers(&actual);
    // audit.failures 空（成功）。
    assert!(actual["audit"]["failures"].as_array().unwrap().is_empty());
}

#[test]
fn chapter_context_verifier_and_repair_views() {
    let dir = setup_min_world();
    for view in ["verifier", "repair"] {
        let out = novel()
            .args(["chapter", "context", "chapter-001", "--view", view, "--budget", "60000", "--json", "--path"])
            .arg(dir.path())
            .assert()
            .success()
            .get_output()
            .clone();
        let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_packet_structure(&actual, "chapter-001", view);
        assert_omitted_no_data_layers(&actual);
    }
}

#[test]
fn chapter_context_budget_exceeded_p0() {
    let dir = setup_min_world();
    // --budget 1：P0 装填后超限（退出码 3 + context_budget_exceeded + failures 非空）。
    let out = novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--budget", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(3)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).expect("错误输出应为 JSON");
    assert_eq!(err["errorId"].as_str().unwrap(), "context_budget_exceeded");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 3);
    assert!(err["details"]["budget"]["limit"].is_i64());
    assert!(err["details"]["budget"]["used"].is_i64());
    let failures = err["details"]["failures"].as_array().expect("应含 failures 数组");
    assert!(!failures.is_empty(), "audit.failures 应非空（部分 packet）");
    assert_eq!(failures[0]["layer"].as_str().unwrap(), "P0");
    // 部分 packet 存在（partialPacket）。
    assert!(err["details"]["partialPacket"].is_object(), "应含 partialPacket（已装填 fragments）");
    let partial = &err["details"]["partialPacket"];
    assert!(partial["fragments"].is_array());
    assert!(!partial["fragments"].as_array().unwrap().is_empty());
}

#[test]
fn chapter_context_invalid_view() {
    let dir = setup_min_world();
    let out = novel()
        .args(["chapter", "context", "chapter-001", "--view", "invalid", "--budget", "60000", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "usage");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 2);
    assert!(err["message"].as_str().unwrap().contains("writer|verifier|repair"));
}

#[test]
fn chapter_context_missing_view() {
    let dir = setup_min_world();
    // 缺 --view：clap 拒绝（退出码 2）。
    novel()
        .args(["chapter", "context", "chapter-001", "--budget", "60000", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn chapter_context_missing_budget() {
    let dir = setup_min_world();
    // 缺 --budget：clap 拒绝（退出码 2）。
    novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn chapter_context_budget_not_positive() {
    let dir = setup_min_world();
    // --budget 0：退出码 2 + usage。
    let out = novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--budget", "0", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "usage");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 2);

    // --budget=-1：退出码 2 + usage。
    let out2 = novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--budget=-1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2)
        .get_output()
        .clone();
    let err2: Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(err2["errorId"].as_str().unwrap(), "usage");
}

#[test]
fn chapter_context_chapter_not_found() {
    let dir = setup_min_world();
    let out = novel()
        .args(["chapter", "context", "chapter-999", "--view", "writer", "--budget", "60000", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(6)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "chapter_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
}

#[test]
fn chapter_context_world_not_initialized() {
    let dir = tempdir().unwrap();
    let out = novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--budget", "60000", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(3)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "world_not_initialized");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 3);
}

#[test]
fn chapter_context_human_readable() {
    let dir = setup_min_world();
    // 无 --json：人读格式，含 view 与 fragments 摘要，退出 0。
    novel()
        .args(["chapter", "context", "chapter-001", "--view", "writer", "--budget", "60000", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("View: writer"))
        .stdout(predicate::str::contains("Fragments"));
}
