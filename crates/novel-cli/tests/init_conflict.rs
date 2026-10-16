// 验证 init 冲突路径（task 4.5）。
// 两次 init 到同一 tempdir，第二次断言退出码 4 且输出结构化错误。
// 同时校验 `--json` 冲突输出匹配 examples/init-conflict.error.golden.json 字段。

use assert_cmd::Command;
use predicates;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn init_conflict_second_run_fails_exit4() {
    let dir = tempdir().unwrap();

    // 第一次：成功。
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 第二次：冲突，退出码 4。人读模式错误进 stderr。
    let mut cmd = novel();
    cmd.args(["init", "demo", "--path"]).arg(dir.path());
    cmd.assert()
        .failure()
        .code(4)
        .stderr(predicates::str::contains("resource_conflict"));
}

#[test]
fn init_conflict_json_output_matches_error_contract() {
    let dir = tempdir().unwrap();

    // 先建一个世界。
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 第二次冲突，带 --json：stdout 为结构化错误对象。
    let mut cmd = novel();
    cmd.args(["init", "demo", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().failure().code(4).get_output().clone();

    let json: serde_json::Value = serde_json::from_slice(&out.stdout)
        .expect("冲突时 stdout 应为单一合法 JSON");
    assert_eq!(json["errorId"], "resource_conflict");
    assert_eq!(json["exitCode"], 4);
    assert!(json["message"].as_str().unwrap().contains(".novel/"));

    // 比对 golden：errorId/exitCode 一致（message 可不同）。
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("../../../contracts/cli/examples/init-conflict.error.golden.json"))
            .unwrap();
    assert_eq!(json["errorId"], golden["errorId"]);
    assert_eq!(json["exitCode"], golden["exitCode"]);
}

#[test]
fn init_conflict_does_not_overwrite() {
    // 冲突后已有世界不应被破坏。
    let dir = tempdir().unwrap();
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    let db_before = dir.path().join(".novel/novel.db");
    let size_before = std::fs::metadata(&db_before).unwrap().len();

    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(4);

    let size_after = std::fs::metadata(&db_before).unwrap().len();
    assert_eq!(size_before, size_after, "冲突时不应修改已有 novel.db");
}
