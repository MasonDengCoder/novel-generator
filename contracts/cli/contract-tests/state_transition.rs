// state_transition 契约测试：novel chapter status（s04-state-machine-and-verdicts 切片）。
//
// 验证：
// - 前向推进成功：outline→drafting→review→approved（门禁满足）
// - 非法路径：outline→approved，退出码 3
// - 终态不可转换：approved→drafting，退出码 3
// - hard 违规阻断：退出码 3
// - 裁决未达成阻断：退出码 3
// - stale 阻断：退出码 3
// - 外部修改阻断：退出码 21
// - 回退成功：review→drafting，退出码 0
// - 多项失败同时报告
//
// 注意：chapter create/update/get 命令使用当前工作目录（无 --path），
// 故用 assert_cmd 的 current_dir 在项目目录内执行。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::{json, Value};
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

/// 初始化世界并创建已写入正文的章节，返回 (章节id, 正文hash)。
fn setup_chapter(dir: &tempfile::TempDir) -> (String, String) {
    let mut cmd = novel();
    cmd.args(["init", "test"]).current_dir(dir.path());
    cmd.assert().success();

    // 缺陷3 修复：set-mainplot-landing 现在会校验主线节点存在，
    // 因此 setup 必须创建卷 1 与真实节点 mp-1-001（对齐 3 位补零 ID）。
    let mut cmd = novel();
    cmd.args(["volume", "create", "第一卷"]).current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["mainplot", "node", "create", "1", "--title", "节点A"])
        .current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["chapter", "create", "测试章节"]).current_dir(dir.path());
    cmd.assert().success();

    let content_file = dir.path().join("content.md");
    std::fs::write(&content_file, "# 第一章\n\n这是测试正文内容。").unwrap();
    let mut cmd = novel();
    cmd.args(["chapter", "update", "chapter-001", "--content"])
        .arg(&content_file)
        .current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["chapter", "get", "chapter-001", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    ("chapter-001".to_string(), chapter["contentHash"].as_str().unwrap().to_string())
}

/// 写入检查结果（passed 控制）。
fn write_check(dir: &tempfile::TempDir, chapter_id: &str, hash: &str, passed: bool) {
    let input = json!({
        "contentHash": hash,
        "checks": [{
            "constraintType": "plot",
            "constraintScope": "world",
            "constraintName": "时间线一致性",
            "severity": "hard",
            "passed": passed,
            "description": null,
            "location": null
        }]
    });
    let input_file = dir.path().join("check-input.json");
    std::fs::write(&input_file, input.to_string()).unwrap();
    let mut cmd = novel();
    cmd.args(["constraint", "check", "--chapter", chapter_id, "--input"])
        .arg(&input_file)
        .current_dir(dir.path());
    cmd.assert().success();
}

/// 写入已达成裁决。
fn write_verdict(dir: &tempfile::TempDir, chapter_id: &str, achieved: &str) {
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        achieved,
    ])
    .current_dir(dir.path());
    cmd.assert().success();
}

/// 推进章节状态（封装）。
fn transition(dir: &tempfile::TempDir, chapter_id: &str, target: &str) {
    let mut cmd = novel();
    cmd.args(["chapter", "status", chapter_id, target])
        .current_dir(dir.path());
    cmd.assert().success();
}

/// 构造满足全部门禁的章节并推进到 drafting。
fn prepare_passing_chapter(dir: &tempfile::TempDir) -> (String, String) {
    let (chapter_id, hash) = setup_chapter(dir);
    write_check(dir, &chapter_id, &hash, true);
    write_verdict(dir, &chapter_id, "1");
    // outline → drafting
    transition(dir, &chapter_id, "drafting");
    (chapter_id, hash)
}

#[test]
fn transition_forward_full_chain_ok() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = prepare_passing_chapter(&dir);

    // drafting → review
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "review", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let golden: Value =
        serde_json::from_str(include_str!("../examples/chapter-status-ok.golden.json")).unwrap();
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["id"].as_str().unwrap(), chapter_id);
    assert_eq!(actual["status"].as_str().unwrap(), "review");
    assert_eq!(actual["schemaVersion"], golden["schemaVersion"]);
    assert_eq!(actual["id"], golden["id"]);
    assert_eq!(actual["status"], golden["status"]);
    assert!(!actual["updatedAt"].as_str().unwrap().is_empty());
    assert_eq!(actual.as_object().unwrap().keys().count(), 4);

    // review → approved
    transition(&dir, &chapter_id, "approved");
}

#[test]
fn transition_illegal_path_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter(&dir);

    // outline → approved（跳过 drafting 与 review）：退出码 3
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "approved", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["status"], "failed");
    assert_eq!(err["errorId"].as_str().unwrap(), "precondition_blocked");
    assert_eq!(err["category"], "precondition");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 3);
    assert!(err["schemaVersion"].is_string());
}

#[test]
fn transition_invalid_target_value() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter(&dir);

    // 非四态之一的目标值：退出码 2（value_parser 拒绝）
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "foo"])
        .current_dir(dir.path());
    cmd.assert().code(2);
}

#[test]
fn transition_approved_terminal_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = prepare_passing_chapter(&dir);
    // 推进到 approved
    transition(&dir, &chapter_id, "review");
    transition(&dir, &chapter_id, "approved");

    // approved → drafting：退出码 3
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "drafting", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "precondition_blocked");
}

#[test]
fn transition_hard_violation_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, hash) = setup_chapter(&dir);
    // 写入未通过的 hard 检查 + 已达成裁决
    write_check(&dir, &chapter_id, &hash, false);
    write_verdict(&dir, &chapter_id, "1");
    transition(&dir, &chapter_id, "drafting");

    // drafting → review：hard 违规阻断，退出码 3
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "review", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "precondition_blocked");
    assert!(err["message"].as_str().unwrap().contains("hard"));
}

#[test]
fn transition_verdict_not_achieved_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, hash) = setup_chapter(&dir);
    // 通过的检查但裁决 achieved=0
    write_check(&dir, &chapter_id, &hash, true);
    write_verdict(&dir, &chapter_id, "0");
    transition(&dir, &chapter_id, "drafting");

    // drafting → review：裁决未达成阻断，退出码 3
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "review", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "precondition_blocked");
    assert!(err["message"].as_str().unwrap().contains("裁决"));
}

#[test]
fn transition_stale_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, hash) = setup_chapter(&dir);
    // 写入检查 + 裁决绑定旧 hash
    write_check(&dir, &chapter_id, &hash, true);
    write_verdict(&dir, &chapter_id, "1");
    // 更新正文（hash 改变），旧检查/裁决变 stale
    let content_file = dir.path().join("content2.md");
    std::fs::write(&content_file, "# 更新后的正文\n\n内容已改变。").unwrap();
    let mut cmd = novel();
    cmd.args(["chapter", "update", &chapter_id, "--content"])
        .arg(&content_file)
        .current_dir(dir.path());
    cmd.assert().success();
    transition(&dir, &chapter_id, "drafting");

    // drafting → review：stale 阻断，退出码 3
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "review", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "precondition_blocked");
    assert!(err["message"].as_str().unwrap().contains("stale"));
}

#[test]
fn transition_external_modification_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter(&dir);

    // 外部修改正文
    let content_path = dir.path().join(".novel/content/chapters/chapter-001.md");
    std::fs::write(&content_path, "外部修改").unwrap();

    // 推进 drafting：外部修改阻断，退出码 21
    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "drafting", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(5).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "content_modified_externally");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 5);
}

#[test]
fn transition_backward_ok() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = prepare_passing_chapter(&dir);
    // 推进到 review
    transition(&dir, &chapter_id, "review");

    // review → drafting（回退，跳过门禁）：退出码 0
    transition(&dir, &chapter_id, "drafting");
}

#[test]
fn transition_multiple_failures_reported() {
    let dir = tempdir().unwrap();
    let (chapter_id, hash) = setup_chapter(&dir);
    // hard 违规 + 不写裁决（裁决未达成）：两项同时失败
    write_check(&dir, &chapter_id, &hash, false);
    transition(&dir, &chapter_id, "drafting");

    let mut cmd = novel();
    cmd.args(["chapter", "status", &chapter_id, "review", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    let msg = err["message"].as_str().unwrap();
    // 应同时报告 hard 违规与裁决未达成（不短路）
    assert!(msg.contains("hard"));
    assert!(msg.contains("裁决"));
}

#[test]
fn transition_chapter_not_found() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "test"]).current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["chapter", "status", "chapter-999", "drafting", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(6).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "chapter_not_found");
}

#[test]
fn transition_world_not_initialized() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["chapter", "status", "chapter-001", "drafting", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "world_not_initialized");
}

#[test]
fn transition_approved_blocks_chapter_update() {
    // approved 终态冻结：approved 章节拒绝修改正文（state-machine.md:41、CLAUDE.md §7）。
    // approved 后不得修改正文，chapter update 返回退出码 3（precondition_blocked）。
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = prepare_passing_chapter(&dir);
    // drafting → review → approved（门禁全满足）
    transition(&dir, &chapter_id, "review");
    transition(&dir, &chapter_id, "approved");

    // 尝试修改正文：应被拒（退出码 3）
    let new_content = dir.path().join("content2.md");
    std::fs::write(&new_content, "# 篡改后的正文\n\n内容已改变。").unwrap();
    let mut cmd = novel();
    cmd.args(["chapter", "update", &chapter_id, "--content"])
        .arg(&new_content)
        .arg("--json")
        .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["status"], "failed");
    assert_eq!(err["errorId"], "precondition_blocked");
    assert_eq!(err["exitCode"], 3);
}
