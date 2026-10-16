// 集成测试：emergence 命令族（task 16.1-16.7）
// 通过真实 CLI 子进程测试涌现完整生命周期与状态机错误路径。

mod common;

use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::Value;

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap()
}

#[test]
fn create_emergence_success_json_matches_schema() {
    let (dir, vid, _) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "新涌现",
        "--description", "一个意外的情节机会",
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);

    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["volumeId"], vid);
    assert_eq!(json["title"], "新涌现");
    assert_eq!(json["description"], "一个意外的情节机会");
    assert_eq!(json["status"], "pending");
    assert!(json["id"].as_str().unwrap().starts_with(&format!("em-{}-", vid)));
    assert!(json["rating"].is_null());
    assert!(json["decision"].is_null());
}

#[test]
fn full_lifecycle_create_rate_decide_execute_archive() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // create
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "生命周期涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // rate（pending → rated）
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "rate", &em_id,
        "--rating", "9",
        "--reason", "高价值涌现",
        "--json",
    ]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(json["status"], "rated");
    assert_eq!(json["rating"], 9);
    assert_eq!(json["ratingReason"], "高价值涌现");

    // decide accept（rated → decided）
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "decide", &em_id,
        "--decision", "accept",
        "--reason", "决定采纳",
        "--json",
    ]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(json["status"], "decided");
    assert_eq!(json["decision"], "accept");

    // execute（decided → executed）
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "execute", &em_id,
        "--target", &ch_id,
        "--note", "执行完毕",
        "--json",
    ]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(json["status"], "executed");
    assert_eq!(json["executionTarget"], ch_id);

    // archive（executed → archived）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "archive", &em_id, "--json"]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(json["status"], "archived");
    assert!(json["archivedAt"].as_str().is_some());
}

#[test]
fn decide_before_rate_returns_invalid_state_transition() {
    let (dir, vid, _) = common::setup_world_with_volume_and_chapter();

    // create
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 未 rate 直接 decide
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "decide", &em_id, "--decision", "accept", "--reason", "理由"]);
    cmd.assert().failure().code(3); // invalid_state_transition → category 3
}

#[test]
fn execute_defer_decision_returns_invalid_state_transition() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // create + rate + decide defer
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "rate", &em_id, "--rating", "5", "--reason", "理由", "--json"]);
    cmd.assert().success();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "decide", &em_id, "--decision", "defer", "--reason", "暂缓", "--json"]);
    cmd.assert().success();

    // defer 决策不能执行
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "execute", &em_id, "--target", &ch_id, "--note", "备注"]);
    cmd.assert().failure().code(3); // invalid_state_transition → category 3
}

#[test]
fn modify_archived_emergence_returns_emergence_archived() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // 走完完整生命周期到 archived
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    common::novel_at(&dir)
        .args(["emergence", "rate", &em_id, "--rating", "9", "--reason", "理由"])
        .assert().success();
    common::novel_at(&dir)
        .args(["emergence", "decide", &em_id, "--decision", "accept", "--reason", "采纳"])
        .assert().success();
    common::novel_at(&dir)
        .args(["emergence", "execute", &em_id, "--target", &ch_id, "--note", "执行"])
        .assert().success();
    common::novel_at(&dir)
        .args(["emergence", "archive", &em_id])
        .assert().success();

    // 对已归档涌现再 rate：应返回 emergence_archived（category 3）
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "rate", &em_id, "--rating", "5", "--reason", "理由"]);
    cmd.assert().failure().code(3); // emergence_archived → category 3
}

#[test]
fn list_with_status_filter() {
    let (dir, vid, _) = common::setup_world_with_volume_and_chapter();

    // 创建两条 pending
    for title in ["涌现A", "涌现B"] {
        let mut cmd = common::novel_at(&dir);
        cmd.args([
            "emergence", "create",
            &vid.to_string(),
            "--title", title,
            "--description", "描述",
            "--json",
        ]);
        cmd.assert().success();
    }

    // 第三条 rate 为 rated
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现C",
        "--description", "描述",
        "--json",
    ]);
    let em_c = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "rate", &em_c, "--rating", "7", "--reason", "理由"]);
    cmd.assert().success();

    // 过滤 pending
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "list", &vid.to_string(), "--status", "pending", "--json"]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    let emergences = json["emergences"].as_array().unwrap();
    assert_eq!(emergences.len(), 2);
    for em in emergences {
        assert_eq!(em["status"], "pending");
    }

    // 过滤 rated
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "list", &vid.to_string(), "--status", "rated", "--json"]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    let emergences = json["emergences"].as_array().unwrap();
    assert_eq!(emergences.len(), 1);
    assert_eq!(emergences[0]["title"], "涌现C");
}

#[test]
fn rate_out_of_range_returns_usage() {
    let (dir, vid, _) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // rating = 11 越界
    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "rate", &em_id, "--rating", "11", "--reason", "理由"]);
    cmd.assert().failure().code(2); // usage（ERR_INVALID_PARAMETER 走 usage 退出码）
}

#[test]
fn get_emergence_success() {
    let (dir, vid, _) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "emergence", "create",
        &vid.to_string(),
        "--title", "查询涌现",
        "--description", "描述",
        "--json",
    ]);
    let em_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "get", &em_id, "--json"]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    assert_eq!(json["id"], em_id);
    assert_eq!(json["title"], "查询涌现");
}

#[test]
fn get_emergence_not_found_returns_45() {
    let (dir, _vid, _ch) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["emergence", "get", "em-999-999", "--json"]);
    cmd.assert().failure().code(6); // emergence_not_found → category 6
}
