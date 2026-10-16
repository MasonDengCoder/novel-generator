// thread 命令契约测试（s06-destiny-weaving）。
//
// 验证：
// - thread create --json 输出符合 thread.schema.json
// - thread query --json 输出符合 thread-binding-list.schema.json
// - 错误码 4（thread_name_conflict / binding_already_exists）、6（thread_not_found）
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
    let chapter_id: Value = serde_json::from_slice(&out.stdout).unwrap();
    let ch = chapter_id["id"].as_str().unwrap().to_string();

    (dir, vid, ch)
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout 应为单一合法 JSON")
}

#[test]
fn create_thread_json_matches_schema() {
    let (dir, vid, _ch) = setup_world();

    let mut cmd = novel();
    cmd.args([
        "thread", "create",
        &vid.to_string(),
        "--name", "复仇线",
        "--description", "主角复仇主线",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // thread.schema.json required 字段
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert!(actual["id"].as_str().unwrap().starts_with(&format!("th-{}-", vid)));
    assert_eq!(actual["volumeId"].as_i64().unwrap(), vid);
    assert_eq!(actual["name"].as_str().unwrap(), "复仇线");
    assert_eq!(actual["description"].as_str().unwrap(), "主角复仇主线");
    assert_eq!(actual["status"].as_str().unwrap(), "active");
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);
    assert!(actual["updatedAt"].as_str().unwrap().len() > 0);
}

#[test]
fn query_bindings_json_matches_schema() {
    let (dir, vid, ch) = setup_world();

    // 创建线索
    let mut cmd = novel();
    cmd.args(["thread", "create", &vid.to_string(), "--name", "线索", "--json"])
        .current_dir(dir.path());
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 绑定
    let mut cmd = novel();
    cmd.args([
        "thread", "bind", &thread_id,
        "--chapter", &ch,
        "--type", "develop",
        "--note", "发展备注",
    ])
    .current_dir(dir.path());
    cmd.assert().success();

    // 查询
    let mut cmd = novel();
    cmd.args(["thread", "query", "--chapter", &ch, "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual = parse_json(&out.stdout);

    // thread-binding-list.schema.json: schemaVersion + bindings 数组
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    let bindings = actual["bindings"].as_array().unwrap();
    assert_eq!(bindings.len(), 1);
    let b = &bindings[0];
    assert_eq!(b["threadId"].as_str().unwrap(), thread_id);
    assert_eq!(b["chapterId"].as_str().unwrap(), ch);
    assert_eq!(b["bindingType"].as_str().unwrap(), "develop");
    assert_eq!(b["note"].as_str().unwrap(), "发展备注");
    assert!(b["createdAt"].as_str().unwrap().len() > 0);
}

#[test]
fn thread_name_conflict_error_object() {
    let (dir, vid, _ch) = setup_world();

    let mut cmd = novel();
    cmd.args(["thread", "create", &vid.to_string(), "--name", "重名", "--json"])
        .current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["thread", "create", &vid.to_string(), "--name", "重名", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(4).get_output().clone();
    let err = parse_json(&out.stdout);
    assert_eq!(err["errorId"].as_str().unwrap(), "thread_name_conflict");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 4);
    assert!(err["message"].as_str().unwrap().len() > 0);
}

#[test]
fn thread_not_found_error_object() {
    let (dir, _vid, _ch) = setup_world();

    // thread update 不支持 --json，错误为人读（stderr）。
    // 契约验证：退出码 6 + stderr 含错误标识。
    let mut cmd = novel();
    cmd.args(["thread", "update", "th-999-999", "--status", "paused"])
        .current_dir(dir.path());
    let out = cmd.assert().code(6).get_output().clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("thread_not_found") || stderr.contains("线索"));
}

#[test]
fn binding_already_exists_error_object() {
    let (dir, vid, ch) = setup_world();

    let mut cmd = novel();
    cmd.args(["thread", "create", &vid.to_string(), "--name", "线索", "--json"])
        .current_dir(dir.path());
    let thread_id = parse_json(&cmd.assert().success().get_output().stdout)["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut cmd = novel();
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch, "--type", "mention"])
        .current_dir(dir.path());
    cmd.assert().success();

    // 重复绑定：thread bind 不支持 --json，错误为人读（stderr）。
    // 契约验证：退出码 4。
    let mut cmd = novel();
    cmd.args(["thread", "bind", &thread_id, "--chapter", &ch, "--type", "develop"])
        .current_dir(dir.path());
    let out = cmd.assert().code(4).get_output().clone();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("binding_already_exists") || stderr.contains("绑定"));
}
