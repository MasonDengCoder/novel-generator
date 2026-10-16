// 集成测试：mainplot 命令族（task 14.1-14.6）
// 通过真实 CLI 子进程测试 mainplot node create/list/get/update 的成功与错误路径。

mod common;

use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::Value;

/// 解析 CLI --json stdout 为 JSON Value。
fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap()
}

#[test]
fn create_node_success_json_matches_schema() {
    let (dir, vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "主线节点",
        "--description", "描述",
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);

    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["volumeId"], vid);
    assert_eq!(json["title"], "主线节点");
    assert_eq!(json["description"], "描述");
    assert_eq!(json["achieved"], 0);
    assert!(json["id"].as_str().unwrap().starts_with(&format!("mp-{}-", vid)));
}

#[test]
fn create_node_with_target_chapters_success() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "带目标章节的节点",
        "--target-chapters", &ch_id,
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);

    let targets = json["targetChapters"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0], ch_id);
}

#[test]
fn create_node_with_nonexistent_chapter_returns_not_found() {
    let (dir, vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "节点",
        "--target-chapters", "ch-999",
        "--json",
    ]);
    let output = cmd.assert().failure().code(6).get_output().clone();
    // mainplot_validator.validate_target_chapters 返回 CliError::not_found（退出码 6）
    // 结构化错误对象输出到 stdout。
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ch-999"));
    assert!(stdout.contains("not_found"));
}

#[test]
fn list_empty_volume_returns_empty_array() {
    let (dir, vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "list",
        &vid.to_string(),
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);

    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert!(json["nodes"].as_array().unwrap().is_empty());
}

#[test]
fn list_returns_created_nodes() {
    let (dir, vid) = common::setup_world_with_volume();

    // 创建两个节点
    for title in ["节点A", "节点B"] {
        let mut cmd = common::novel_at(&dir);
        cmd.args([
            "mainplot", "node", "create",
            &vid.to_string(),
            "--title", title,
            "--json",
        ]);
        cmd.assert().success();
    }

    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "list", &vid.to_string(), "--json"]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);

    let nodes = json["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0]["title"], "节点A");
    assert_eq!(nodes[1]["title"], "节点B");
    // 按序号递增
    assert_eq!(nodes[0]["sequenceNumber"], 1);
    assert_eq!(nodes[1]["sequenceNumber"], 2);
}

#[test]
fn get_node_returns_correct_detail() {
    let (dir, vid) = common::setup_world_with_volume();

    // 先创建
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "详情节点",
        "--description", "详情描述",
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let created = parse_json(&output.stdout);
    let node_id = created["id"].as_str().unwrap();

    // 再 get
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "get", node_id, "--json"]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);
    assert_eq!(json["id"], node_id);
    assert_eq!(json["title"], "详情节点");
    assert_eq!(json["description"], "详情描述");
}

#[test]
fn get_node_not_found_returns_exit_43() {
    let (dir, _vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "get", "mp-999-999", "--json"]);
    cmd.assert().failure().code(6); // mainplot_node_not_found → category 6
}

#[test]
fn update_node_achieved_updates_status() {
    let (dir, vid) = common::setup_world_with_volume();

    // 创建节点
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "待达成节点",
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let node_id = parse_json(&output.stdout)["id"].as_str().unwrap().to_string();

    // 标记达成
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "update", &node_id, "--achieved", "1"]);
    cmd.assert().success();

    // 验证状态已更新
    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "get", &node_id, "--json"]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);
    assert_eq!(json["achieved"], 1);
}

#[test]
fn update_node_not_found_returns_exit_43() {
    let (dir, _vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["mainplot", "node", "update", "mp-999-999", "--achieved", "1"]);
    cmd.assert().failure().code(6); // mainplot_node_not_found → category 6
}

#[test]
fn human_readable_output_is_readable() {
    let (dir, vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "人读节点",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("主线节点"));
    assert!(stdout.contains("人读节点"));
}
