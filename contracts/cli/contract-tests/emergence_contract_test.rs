// emergence 命令契约测试（s06-destiny-weaving）。
//
// 验证：
// - emergence create --json 输出符合 emergence.schema.json
// - emergence list --json 输出符合 emergence-list.schema.json
// - 错误码 3（invalid_state_transition / emergence_archived）、6（emergence_not_found）
//   结构化错误对象符合 V2 错误信封

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

/// 初始化世界、创建卷与一个章节，返回 (TempDir, volume_id, chapter_id)。
fn setup_world() -> (tempfile::TempDir, i64, String) {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "demo", "--path"]).arg(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["volume", "create", "卷一", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let vid: i64 = stdout
        .lines()
        .find(|l| l.contains("ID ="))
        .unwrap()
        .split("ID =")
        .nth(1)
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let mut cmd = novel();
    cmd.args(["chapter", "create", "第一章", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    let ch = chapter["id"].as_str().unwrap().to_string();

    (dir, vid, ch)
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout 应为单一合法 JSON")
}

#[test]
fn create_emergence_json_matches_schema() {
    let (dir, vid, _ch) = setup_world();

    let mut cmd = novel();
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "契约涌现",
        "--description", "契约描述",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // emergence.schema.json required 字段
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert!(actual["id"].as_str().unwrap().starts_with(&format!("em-{}-", vid)));
    assert_eq!(actual["volumeId"].as_i64().unwrap(), vid);
    assert_eq!(actual["title"].as_str().unwrap(), "契约涌现");
    assert_eq!(actual["description"].as_str().unwrap(), "契约描述");
    assert_eq!(actual["status"].as_str().unwrap(), "pending");
    assert!(actual["discoveredAt"].as_str().unwrap().len() > 0);
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);
    assert!(actual["updatedAt"].as_str().unwrap().len() > 0);
    // 生命周期初始为空
    assert!(actual["rating"].is_null());
    assert!(actual["decision"].is_null());
    assert!(actual["executionTarget"].is_null());
    assert!(actual["archivedAt"].is_null());
}

#[test]
fn list_emergence_json_matches_schema() {
    let (dir, vid, _ch) = setup_world();

    for title in ["涌现A", "涌现B"] {
        let mut cmd = novel();
        cmd.args([
            "emergence", "create",
            &vid.to_string(),
            "--title", title,
            "--description", "描述",
            "--json",
        ])
        .current_dir(dir.path());
        cmd.assert().success();
    }

    let mut cmd = novel();
    cmd.args(["emergence", "list", &vid.to_string(), "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // emergence-list.schema.json: schemaVersion + emergences 数组
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    let emergences = actual["emergences"].as_array().unwrap();
    assert_eq!(emergences.len(), 2);
    for em in emergences {
        assert!(em["id"].is_string());
        assert!(em["volumeId"].is_i64());
        assert!(em["status"].is_string());
        assert!(em["discoveredAt"].is_string());
    }
}

#[test]
fn invalid_state_transition_error_object() {
    let (dir, vid, _ch) = setup_world();

    // create
    let mut cmd = novel();
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现",
        "--description", "描述",
        "--json",
    ])
    .current_dir(dir.path());
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 未 rate 直接 decide：退出码 3（--json 使错误对象输出到 stdout）
    let mut cmd = novel();
    cmd.args(["emergence", "decide", &em_id, "--decision", "accept", "--reason", "理由", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err = parse_json(&out.stdout);
    assert_eq!(err["errorId"].as_str().unwrap(), "invalid_state_transition");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 3);
    assert!(err["message"].as_str().unwrap().len() > 0);
}

#[test]
fn emergence_not_found_error_object() {
    let (dir, _vid, _ch) = setup_world();

    let mut cmd = novel();
    cmd.args(["emergence", "get", "em-999-999", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(6).get_output().clone();
    let err = parse_json(&out.stdout);
    assert_eq!(err["errorId"].as_str().unwrap(), "emergence_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
}

#[test]
fn emergence_archived_error_object() {
    let (dir, vid, ch) = setup_world();

    // 走完整生命周期到 archived
    let mut cmd = novel();
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现",
        "--description", "描述",
        "--json",
    ])
    .current_dir(dir.path());
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    for args in [
        vec!["emergence", "rate", &em_id, "--rating", "9", "--reason", "理由"],
        vec!["emergence", "decide", &em_id, "--decision", "accept", "--reason", "采纳"],
        vec!["emergence", "execute", &em_id, "--target", &ch, "--note", "执行"],
        vec!["emergence", "archive", &em_id],
    ] {
        let mut cmd = novel();
        cmd.args(&args).current_dir(dir.path());
        cmd.assert().success();
    }

    // 对已归档涌现再 rate：退出码 3（--json 使错误对象输出到 stdout）
    let mut cmd = novel();
    cmd.args(["emergence", "rate", &em_id, "--rating", "5", "--reason", "理由", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err = parse_json(&out.stdout);
    assert_eq!(err["errorId"].as_str().unwrap(), "emergence_archived");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 3);
}

#[test]
fn full_lifecycle_json_fields_evolve() {
    let (dir, vid, ch) = setup_world();

    // create → pending
    let mut cmd = novel();
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "生命周期",
        "--description", "描述",
        "--json",
    ])
    .current_dir(dir.path());
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // rate → rated，rating/ratingReason/ratedAt 应填充
    let mut cmd = novel();
    cmd.args([
        "emergence", "rate", &em_id,
        "--rating", "8",
        "--reason", "理由",
        "--json",
    ])
    .current_dir(dir.path());
    let actual = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(actual["status"].as_str().unwrap(), "rated");
    assert_eq!(actual["rating"].as_i64().unwrap(), 8);
    assert_eq!(actual["ratingReason"].as_str().unwrap(), "理由");
    assert!(actual["ratedAt"].as_str().is_some());

    // decide accept → decided
    let mut cmd = novel();
    cmd.args([
        "emergence", "decide", &em_id,
        "--decision", "accept",
        "--reason", "采纳",
        "--json",
    ])
    .current_dir(dir.path());
    let actual = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(actual["status"].as_str().unwrap(), "decided");
    assert_eq!(actual["decision"].as_str().unwrap(), "accept");
    assert!(actual["decidedAt"].as_str().is_some());

    // execute → executed
    let mut cmd = novel();
    cmd.args([
        "emergence", "execute", &em_id,
        "--target", &ch,
        "--note", "执行完毕",
        "--json",
    ])
    .current_dir(dir.path());
    let actual = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(actual["status"].as_str().unwrap(), "executed");
    assert_eq!(actual["executionTarget"].as_str().unwrap(), ch);
    assert_eq!(actual["executionNote"].as_str().unwrap(), "执行完毕");
    assert!(actual["executedAt"].as_str().is_some());

    // archive → archived
    let mut cmd = novel();
    cmd.args(["emergence", "archive", &em_id, "--json"])
        .current_dir(dir.path());
    let actual = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(actual["status"].as_str().unwrap(), "archived");
    assert!(actual["archivedAt"].as_str().is_some());
}

// ===== T-02：emergence get 成功路径契约测试（s05-contract-test-coverage）=====
//
// 验证 `novel emergence get <id> --json` 成功输出匹配 contracts/cli/json-schemas/emergence.schema.json。
// get 为 --json 命令（commands.md `## novel emergence get` 条目），其契约面即 JSON 输出 schema。
// 测试不复制 schema 内容，仅引用 schema 文件名与字段名从实际 CLI 子进程输出比对。

#[test]
fn get_emergence_json_matches_schema() {
    let (dir, vid, _ch) = setup_world();

    // 创建一个 pending 初始态涌现
    let mut cmd = novel();
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "查询涌现",
        "--description", "查询描述",
        "--json",
    ])
    .current_dir(dir.path());
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // get --json 成功路径
    let mut cmd = novel();
    cmd.args(["emergence", "get", &em_id, "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // emergence.schema.json required 字段集合
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["id"].as_str().unwrap(), em_id);
    assert_eq!(actual["volumeId"].as_i64().unwrap(), vid);
    assert_eq!(actual["title"].as_str().unwrap(), "查询涌现");
    assert_eq!(actual["description"].as_str().unwrap(), "查询描述");
    assert_eq!(actual["status"].as_str().unwrap(), "pending");
    assert!(actual["discoveredAt"].as_str().unwrap().len() > 0);
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);
    assert!(actual["updatedAt"].as_str().unwrap().len() > 0);

    // id 符合 emergence.schema.json pattern ^em-[0-9]+-[0-9]+$
    assert!(em_id.starts_with("em-"));
    let rest = &em_id[3..];
    let parts: Vec<&str> = rest.splitn(2, '-').collect();
    assert_eq!(parts.len(), 2);
    assert!(parts[0].parse::<i64>().is_ok(), "volume_id 部分应为数字");
    assert!(parts[1].parse::<i64>().is_ok(), "seq 部分应为数字");

    // 生命周期字段在 pending 初始态为 null（与既有 create_emergence_json_matches_schema 断言的初始态一致）
    assert!(actual["rating"].is_null(), "pending 态 rating 应为 null");
    assert!(actual["decision"].is_null(), "pending 态 decision 应为 null");
    assert!(actual["executionTarget"].is_null(), "pending 态 executionTarget 应为 null");
    assert!(actual["archivedAt"].is_null(), "pending 态 archivedAt 应为 null");
}
