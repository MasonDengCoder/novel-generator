// 集成测试：thread 命令族（task 15.1-15.7）
// 通过真实 CLI 子进程测试 thread create/bind/list/query/update 的成功与错误路径。

mod common;

use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::Value;

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap()
}

#[test]
fn create_thread_success_json_matches_schema() {
    let (dir, vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "thread", "create",
        &vid.to_string(),
        "--name", "复仇线",
        "--description", "主角复仇主线",
        "--json",
    ]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);

    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert_eq!(json["volumeId"], vid);
    assert_eq!(json["name"], "复仇线");
    assert_eq!(json["description"], "主角复仇主线");
    assert_eq!(json["status"], "active");
    assert!(json["id"].as_str().unwrap().starts_with(&format!("th-{}-", vid)));
}

#[test]
fn bind_thread_to_chapter_success() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // 创建线索
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "thread", "create",
        &vid.to_string(),
        "--name", "线索1",
        "--json",
    ]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 绑定
    let mut cmd = common::novel_at(&dir);
    cmd.args([
        "thread", "bind",
        &thread_id,
        "--chapter", &ch_id,
        "--type", "develop",
        "--note", "首次发展",
    ]);
    cmd.assert().success();

    // 查询验证
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "query", "--chapter", &ch_id, "--json"]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);
    let bindings = json["bindings"].as_array().unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0]["threadId"], thread_id);
    assert_eq!(bindings[0]["chapterId"], ch_id);
    assert_eq!(bindings[0]["bindingType"], "develop");
    assert_eq!(bindings[0]["note"], "首次发展");
}

#[test]
fn create_duplicate_name_returns_thread_name_conflict() {
    let (dir, vid) = common::setup_world_with_volume();

    // 创建第一条
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "重名线索", "--json"]);
    cmd.assert().success();

    // 同卷创建重名
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "重名线索", "--json"]);
    cmd.assert().failure().code(4); // thread_name_conflict → category 4
}

#[test]
fn bind_nonexistent_chapter_returns_chapter_not_found() {
    let (dir, vid, _ch) = common::setup_world_with_volume_and_chapter();

    // 创建线索
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "线索", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 绑定到不存在的章节
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "bind", &thread_id, "--chapter", "ch-999", "--type", "mention"]);
    cmd.assert().failure().code(6); // chapter_not_found → category 6
}

#[test]
fn bind_duplicate_binding_returns_binding_already_exists() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    // 创建线索
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "线索", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 首次绑定
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch_id, "--type", "mention"]);
    cmd.assert().success();

    // 重复绑定
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch_id, "--type", "develop"]);
    cmd.assert().failure().code(4); // binding_already_exists → category 4
}

#[test]
fn list_threads_returns_all() {
    let (dir, vid) = common::setup_world_with_volume();

    for name in ["线索A", "线索B", "线索C"] {
        let mut cmd = common::novel_at(&dir);
        cmd.args(["thread", "create", &vid.to_string(), "--name", name, "--json"]);
        cmd.assert().success();
    }

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "list", &vid.to_string(), "--json"]);
    let output = cmd.assert().success().get_output().clone();
    let json = parse_json(&output.stdout);
    assert_eq!(json["threads"].as_array().unwrap().len(), 3);
}

#[test]
fn update_thread_status_active_to_paused_to_resolved() {
    let (dir, vid) = common::setup_world_with_volume();

    // 创建线索
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "状态流转线索", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // active → paused
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "update", &thread_id, "--status", "paused"]);
    cmd.assert().success();

    // 验证
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "list", &vid.to_string(), "--json"]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    let threads = json["threads"].as_array().unwrap();
    assert_eq!(threads[0]["status"], "paused");

    // paused → resolved
    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "update", &thread_id, "--status", "resolved"]);
    cmd.assert().success();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "list", &vid.to_string(), "--json"]);
    let json = parse_json(&cmd.assert().success().get_output().stdout);
    let threads = json["threads"].as_array().unwrap();
    assert_eq!(threads[0]["status"], "resolved");
}

#[test]
fn update_thread_invalid_status_returns_usage() {
    let (dir, vid) = common::setup_world_with_volume();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "线索", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "update", &thread_id, "--status", "invalid"]);
    cmd.assert().failure().code(2); // usage
}

#[test]
fn bind_invalid_type_returns_usage() {
    let (dir, vid, ch_id) = common::setup_world_with_volume_and_chapter();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "create", &vid.to_string(), "--name", "线索", "--json"]);
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut cmd = common::novel_at(&dir);
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch_id, "--type", "invalid"]);
    cmd.assert().failure().code(2); // usage
}
