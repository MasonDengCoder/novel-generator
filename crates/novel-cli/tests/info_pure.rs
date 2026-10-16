// 验证 info JSON 纯净性（task 5.5）。
// 断言：stdout 为单一合法 JSON，无 ANSI 转义序列。

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn info_json_stdout_is_pure_json() {
    let dir = tempdir().unwrap();
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    let mut cmd = novel();
    cmd.args(["info", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout).unwrap();

    // 可被 JSON 解析器无报错解析。
    let _: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout 应为单一合法 JSON");

    // 不含 ANSI 转义（CSI 序列 \x1b[...）。
    assert!(
        !predicates::str::contains("\u{1b}").eval(&stdout),
        "JSON 输出不得包含 ANSI 转义序列"
    );
}

#[test]
fn info_json_has_no_human_verbosity() {
    // 确认 JSON 顶层键不含人读旁白字段。
    let dir = tempdir().unwrap();
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    let mut cmd = novel();
    cmd.args(["info", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let obj = v.as_object().unwrap();
    // 仅允许契约字段，不允许任意旁白键。
    for allowed in [
        "schemaVersion", "projectName", "dbPath", "createdAt", "counts",
    ] {
        let _ = allowed;
    }
    let extra: Vec<&String> = obj
        .keys()
        .filter(|k| !["schemaVersion", "projectName", "dbPath", "createdAt", "counts"].contains(&k.as_str()))
        .collect();
    assert!(extra.is_empty(), "JSON 含非契约字段: {:?}", extra);
}
