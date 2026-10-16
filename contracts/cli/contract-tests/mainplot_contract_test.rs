// mainplot 命令契约测试（s06-destiny-weaving）。
//
// 验证：
// - mainplot node create --json 输出符合 mainplot-node.schema.json（字段集合与值）
// - mainplot node list --json 输出符合 mainplot-node-list.schema.json
// - mainplot node get --json 输出符合 mainplot-node.schema.json
// - 错误码 6（mainplot_node_not_found）结构化错误对象符合 V2 错误信封
//
// 注意：volume create 不支持 --json，从人读输出解析 "ID = N"；
// chapter create 不支持 --path，用 current_dir 执行。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

/// 初始化世界并创建一个卷，返回 (TempDir, volume_id)。
fn setup_world_with_volume() -> (tempfile::TempDir, i64) {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "demo", "--path"]).arg(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["volume", "create", "卷一", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let line = stdout
        .lines()
        .find(|l| l.contains("ID ="))
        .expect("volume create 输出应包含 ID =");
    let vid: i64 = line.split("ID =").nth(1).unwrap().trim().parse().unwrap();
    (dir, vid)
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout 应为单一合法 JSON")
}

#[test]
fn create_node_json_matches_schema() {
    let (dir, vid) = setup_world_with_volume();

    let mut cmd = novel();
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "契约节点",
        "--description", "契约描述",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // mainplot-node.schema.json required 字段
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert!(actual["id"].as_str().unwrap().starts_with(&format!("mp-{}-", vid)));
    assert_eq!(actual["volumeId"].as_i64().unwrap(), vid);
    assert!(actual["sequenceNumber"].as_i64().unwrap() >= 1);
    assert_eq!(actual["title"].as_str().unwrap(), "契约节点");
    assert_eq!(actual["description"].as_str().unwrap(), "契约描述");
    // mainplot-node.schema.json：achieved 为 integer enum [0,1]
    let achieved = actual["achieved"].as_i64().expect("achieved 应为 integer");
    assert!(achieved == 0 || achieved == 1, "achieved 必须为 0 或 1，实际 {}", achieved);
    assert!(!actual["achieved"].is_boolean(), "achieved 不得为 bool，必须为 integer");
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);
    assert!(actual["updatedAt"].as_str().unwrap().len() > 0);
    // 字段集合完整（8 required + 2 optional = 最多 10）
    let keys = actual.as_object().unwrap().keys().count();
    assert!(keys >= 8 && keys <= 10, "字段集合应为 8-10 个，实际 {}", keys);
}

#[test]
fn create_node_id_format_matches_pattern() {
    let (dir, vid) = setup_world_with_volume();

    let mut cmd = novel();
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "节点",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);
    let id = actual["id"].as_str().unwrap();
    // schema pattern: ^mp-[0-9]+-[0-9]+$
    assert!(id.starts_with("mp-"));
    let rest = &id[3..];
    let parts: Vec<&str> = rest.splitn(2, '-').collect();
    assert_eq!(parts.len(), 2);
    assert!(parts[0].parse::<i64>().is_ok(), "volume_id 部分应为数字");
    assert!(parts[1].parse::<i64>().is_ok(), "seq 部分应为数字");
}

#[test]
fn list_node_json_matches_schema() {
    let (dir, vid) = setup_world_with_volume();

    // 创建两个节点
    for title in ["节点A", "节点B"] {
        let mut cmd = novel();
        cmd.args([
            "mainplot", "node", "create",
            &vid.to_string(),
            "--title", title,
            "--json",
        ])
        .current_dir(dir.path());
        cmd.assert().success();
    }

    let mut cmd = novel();
    cmd.args(["mainplot", "node", "list", &vid.to_string(), "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // mainplot-node-list.schema.json: schemaVersion + nodes 数组
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    let nodes = actual["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2);
    // 每个节点符合 mainplot-node schema 的 required 字段
    for node in nodes {
        assert!(node["id"].is_string());
        assert!(node["volumeId"].is_i64());
        assert!(node["sequenceNumber"].is_i64());
        assert!(node["title"].is_string());
        assert!(node["achieved"].is_i64(), "每个节点的 achieved 应为 integer");
        let a = node["achieved"].as_i64().unwrap();
        assert!(a == 0 || a == 1, "achieved 必须为 0 或 1");
        assert!(node["createdAt"].is_string());
        assert!(node["updatedAt"].is_string());
    }
}

#[test]
fn get_node_json_matches_schema() {
    let (dir, vid) = setup_world_with_volume();

    // 先创建
    let mut cmd = novel();
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "查询节点",
        "--description", "查询描述",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let node_id = parse_json(&out.stdout)["id"].as_str().unwrap().to_string();

    // get
    let mut cmd = novel();
    cmd.args(["mainplot", "node", "get", &node_id, "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["id"].as_str().unwrap(), node_id);
    assert_eq!(actual["volumeId"].as_i64().unwrap(), vid);
    assert_eq!(actual["title"].as_str().unwrap(), "查询节点");
    assert_eq!(actual["description"].as_str().unwrap(), "查询描述");
}

#[test]
fn get_node_not_found_error_object_matches_contract() {
    let (dir, _vid) = setup_world_with_volume();

    let mut cmd = novel();
    cmd.args(["mainplot", "node", "get", "mp-999-999", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(6).get_output().clone();

    let err = parse_json(&out.stdout);
    // 错误对象契约：errorId + message + exitCode
    assert_eq!(err["errorId"].as_str().unwrap(), "mainplot_node_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
    assert!(err["message"].as_str().unwrap().len() > 0);
}

// ===== T-01：mainplot node update 契约测试（s05-contract-test-coverage）=====
//
// update 为 no-JSON 命令（commands.md `## novel mainplot node update` 条目无 --json 选项）。
// 按 json-output.md §79：no-JSON 成功输出人读到 stdout，no-JSON 错误输出人读到 stderr（非 stdout 结构化错误对象）。
// 故 update 契约测试覆盖：人读成功输出（stdout）、退出码、跨命令 get --json 反向校验落库值、stderr 错误。
//
// 注：commands.md 条目声明成功文案 `✓ 主线节点状态已更新`，但实现（renderers/mainplot_renderer.rs
// ::render_node_update_success）输出 `主线节点 <id> 达成状态已更新。`——SSOT 与实现不一致，归 s01/簇1，
// 本 slice 不修契约文档/实现，测试按实现当前行为断言并标注偏差；待 s01 对齐后同步更新断言。

#[test]
fn update_node_success_human_readable_output() {
    let (dir, vid) = setup_world_with_volume();

    // 创建节点（初始 achieved=0）
    let mut cmd = novel();
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "待更新节点",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let node_id = parse_json(&out.stdout)["id"].as_str().unwrap().to_string();

    // update 为 no-JSON 命令，成功输出人读到 stdout
    let mut cmd = novel();
    cmd.args([
        "mainplot", "node", "update",
        &node_id,
        "--achieved", "1",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 成功文案：按实现当前行为断言（SSOT 偏差见文件头注释，归 s01/簇1）
    assert!(stdout.contains("主线节点"), "成功输出应含「主线节点」: {}", stdout);
    assert!(stdout.contains("达成状态已更新"), "成功输出应含「达成状态已更新」: {}", stdout);

    // 跨命令 get --json 反向校验落库值：achieved 应为 integer 1（非 bool）
    let mut cmd = novel();
    cmd.args(["mainplot", "node", "get", &node_id, "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);
    let achieved = actual["achieved"].as_i64().expect("achieved 应为 integer");
    assert_eq!(achieved, 1, "update --achieved 1 后 get 应反映 achieved=1");
    assert!(!actual["achieved"].is_boolean(), "achieved 不得为 bool，必须为 integer");
}

#[test]
fn update_node_allows_state_rollback() {
    let (dir, vid) = setup_world_with_volume();

    // 创建节点
    let mut cmd = novel();
    cmd.args([
        "mainplot", "node", "create",
        &vid.to_string(),
        "--title", "回退节点",
        "--json",
    ])
    .current_dir(dir.path());
    let node_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 先 update --achieved 1
    let mut cmd = novel();
    cmd.args(["mainplot", "node", "update", &node_id, "--achieved", "1"])
        .current_dir(dir.path());
    cmd.assert().success();

    // 再 update --achieved 0：commands.md 条目声明「允许状态回退」，退出码应为 0
    let mut cmd = novel();
    cmd.args(["mainplot", "node", "update", &node_id, "--achieved", "0"])
        .current_dir(dir.path());
    cmd.assert().success();

    // 跨命令 get --json 反向校验：achieved 应为 integer 0
    let mut cmd = novel();
    cmd.args(["mainplot", "node", "get", &node_id, "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);
    let achieved = actual["achieved"].as_i64().expect("achieved 应为 integer");
    assert_eq!(achieved, 0, "回退后 get 应反映 achieved=0");
    assert!(!actual["achieved"].is_boolean(), "achieved 不得为 bool，必须为 integer");
}

#[test]
fn update_node_not_found_error_matches_contract() {
    let (dir, _vid) = setup_world_with_volume();

    // update 为 no-JSON 命令，节点不存在时按 json-output.md §79 输出人读错误到 stderr
    // 退出码 6 / mainplot_node_not_found 取值以 exit-codes.md / error-codes.md SSOT 为准
    let mut cmd = novel();
    cmd.args(["mainplot", "node", "update", "mp-999-999", "--achieved", "1"])
        .current_dir(dir.path());
    cmd.assert()
        .failure()
        .code(6)
        // no-JSON 错误走 stderr（非 stdout 结构化错误对象）
        .stderr(predicate::str::contains("mainplot_node_not_found"))
        .stderr(predicate::str::contains("主线节点"));
}
