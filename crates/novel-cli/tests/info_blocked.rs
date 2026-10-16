// 验证 info 未初始化阻断（task 5.6）。
// 在空 tempdir 跑 `novel info`，断言退出码 3（前置条件阻断）。

use assert_cmd::Command;
use predicates;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn info_uninitialized_exits_3() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["info", "--path"]).arg(dir.path());
    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicates::str::contains("未初始化"));
}

#[test]
fn info_uninitialized_json_exits_3_with_error_object() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["info", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().failure().code(3).get_output().clone();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["errorId"], "world_not_initialized");
    assert_eq!(v["exitCode"], 3);
}

#[test]
fn info_path_not_exist_exits_2() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("no-such-dir");
    let mut cmd = novel();
    cmd.args(["info", "--path"]).arg(&missing);
    cmd.assert().failure().code(2);
}
