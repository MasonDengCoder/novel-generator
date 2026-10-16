// 验证 init 成功路径（task 4.4）。
// 用 assert_cmd + tempfile 跑 `novel init demo --path <tempdir>`，
// 断言退出 0 且 `.novel/novel.db` 存在，目录结构齐全。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn init_ok_creates_world() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "demo", "--path"]).arg(dir.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("已初始化"));

    let world = dir.path().join(".novel");
    assert!(world.is_dir(), ".novel/ 应被创建");
    assert!(world.join("novel.db").is_file(), "novel.db 应存在");
    assert!(world.join("content/chapters").is_dir());
    assert!(world.join("content/residues").is_dir());
    assert!(world.join("content/reports").is_dir());
    assert!(world.join("tmp").is_dir());
    assert!(world.join("locks").is_dir());
}

#[test]
fn init_ok_json_output() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "demo", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["projectName"], "demo");
    assert_eq!(json["dbPath"], ".novel/novel.db");
    assert_eq!(json["schemaVersion"], CURRENT_SCHEMA_VERSION.to_string());
    assert!(json["createdAt"].as_str().unwrap().len() > 0);
}
