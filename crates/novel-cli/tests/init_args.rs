// 验证 init 参数错误（task 4.6）。
// name 缺失、name 含 `/`、--path 不存在，各断言退出码 2。

use assert_cmd::Command;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn init_missing_name_exits_2() {
    // clap 必填参数缺失：退出码 2。
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "--path"]).arg(dir.path());
    cmd.assert().failure().code(2);
}

#[test]
fn init_name_with_separator_exits_2() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "a/b", "--path"]).arg(dir.path());
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("路径分隔符"));
}

#[test]
fn init_path_not_exist_exits_2() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("does-not-exist");
    let mut cmd = novel();
    cmd.args(["init", "demo", "--path"]).arg(&missing);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("不存在"));
}
