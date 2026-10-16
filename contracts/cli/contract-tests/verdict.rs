// verdict 命令契约测试：novel verdict set-mainplot-landing（s04-state-machine-and-verdicts 切片）。
//
// 验证：
// - 写入成功：退出 0，--json 含正确 contentHash、achieved、schemaVersion
// - 同节点覆盖：重复写入仍退出 0
// - 外部修改拒绝：退出 21，错误码 content_modified_externally
// - 章节不存在：退出 20，错误码 chapter_not_found
// - 参数缺失/非法 achieved：退出 2
// - approved 章节拒绝：退出 3，错误码 precondition_blocked
// - golden/schema 一致性：--json 输出符合 verdict.schema.json
//
// 注意：chapter create/update/get 命令使用当前工作目录（无 --path），
// 故用 assert_cmd 的 current_dir 在项目目录内执行；verdict/status 命令同样用 current_dir。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

/// 初始化世界并创建一个已写入正文的章节，返回 (章节id, 正文hash)。
fn setup_chapter_with_content(dir: &tempfile::TempDir) -> (String, String) {
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

    // 写入正文
    let content_file = dir.path().join("content.md");
    std::fs::write(&content_file, "# 第一章\n\n这是测试正文内容。").unwrap();

    let mut cmd = novel();
    cmd.args(["chapter", "update", "chapter-001", "--content"])
        .arg(&content_file)
        .current_dir(dir.path());
    cmd.assert().success();

    // 读取章节获取 contentHash
    let mut cmd = novel();
    cmd.args(["chapter", "get", "chapter-001", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    let hash = chapter["contentHash"].as_str().unwrap().to_string();

    ("chapter-001".to_string(), hash)
}

#[test]
fn verdict_set_success_json() {
    let dir = tempdir().unwrap();
    let (chapter_id, hash) = setup_chapter_with_content(&dir);

    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();

    let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");
    let golden: Value = serde_json::from_str(include_str!("../examples/verdict-set-ok.json")).unwrap();

    // 验证 verdict.schema.json 必填字段
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["chapterId"].as_str().unwrap(), chapter_id);
    assert_eq!(actual["nodeId"].as_str().unwrap(), "mp-1-001");
    assert_eq!(actual["achieved"].as_i64().unwrap(), 1);
    assert_eq!(actual["contentHash"].as_str().unwrap(), hash);
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);
    assert_eq!(actual["schemaVersion"], golden["schemaVersion"]);
    assert_eq!(actual["chapterId"], golden["chapterId"]);
    assert_eq!(actual["nodeId"], golden["nodeId"]);
    assert_eq!(actual["achieved"], golden["achieved"]);

    // 验证字段集合完整（无多余字段）
    assert_eq!(actual.as_object().unwrap().keys().len(), 6);
}

#[test]
fn verdict_set_not_achieved_json() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter_with_content(&dir);

    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "0",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["achieved"].as_i64().unwrap(), 0);
}

#[test]
fn verdict_set_overwrite_same_node() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter_with_content(&dir);

    // 第一次 achieved=0
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "0",
    ])
    .current_dir(dir.path());
    cmd.assert().success();

    // 同节点覆盖为 achieved=1
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
        "--json",
    ])
    .current_dir(dir.path());
    cmd.assert().success();
}

#[test]
fn verdict_set_rebinds_hash_after_content_update() {
    // 场景：正文修改后重写裁决，新裁决的 contentHash SHALL 绑定为更新后的正文 hash，
    // 而非旧 hash（specs/verdict-management/spec.md:34-36）。
    let dir = tempdir().unwrap();
    let (chapter_id, old_hash) = setup_chapter_with_content(&dir);

    // 更新正文（hash 改变）
    let content_file = dir.path().join("content2.md");
    std::fs::write(&content_file, "# 更新后的正文\n\n内容已改变，hash 必然不同。").unwrap();
    let mut cmd = novel();
    cmd.args(["chapter", "update", &chapter_id, "--content"])
        .arg(&content_file)
        .current_dir(dir.path());
    cmd.assert().success();

    // 读取更新后的 contentHash
    let mut cmd = novel();
    cmd.args(["chapter", "get", &chapter_id, "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    let new_hash = chapter["contentHash"].as_str().unwrap();
    assert_ne!(new_hash, old_hash, "更新正文后 hash 必须改变");

    // 写入裁决：contentHash 应绑定新 hash，而非旧 hash
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let verdict: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        verdict["contentHash"].as_str().unwrap(),
        new_hash,
        "裁决应绑定更新后的正文 hash"
    );
    assert_ne!(
        verdict["contentHash"].as_str().unwrap(),
        old_hash,
        "裁决不得绑定旧 hash"
    );
}

#[test]
fn verdict_set_external_modification_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter_with_content(&dir);

    // 外部修改正文文件
    let content_path = dir.path().join(".novel/content/chapters/chapter-001.md");
    std::fs::write(&content_path, "外部修改的内容").unwrap();

    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().code(5).get_output().clone();

    let err: Value = serde_json::from_slice(&out.stdout).expect("错误输出应为 JSON");
    assert_eq!(err["status"], "failed");
    assert_eq!(err["errorId"].as_str().unwrap(), "content_modified_externally");
    assert_eq!(err["category"], "integrity");
    assert_eq!(err["exitCode"], 5);
    assert!(err["schemaVersion"].is_string());
    assert_eq!(err["exitCode"].as_i64().unwrap(), 5);
}

#[test]
fn verdict_set_chapter_not_found() {
    let dir = tempdir().unwrap();
    let mut cmd = novel();
    cmd.args(["init", "test"]).current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        "chapter-999",
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().code(6).get_output().clone();

    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "chapter_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
}

#[test]
fn verdict_set_missing_node_id() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter_with_content(&dir);

    // 缺失 --node-id：退出码 2（clap required）
    let mut cmd = novel();
    cmd.args(["verdict", "set-mainplot-landing", &chapter_id, "--achieved", "1"])
        .current_dir(dir.path());
    cmd.assert().code(2);
}

#[test]
fn verdict_set_invalid_achieved() {
    let dir = tempdir().unwrap();
    let (chapter_id, _hash) = setup_chapter_with_content(&dir);

    // --achieved 非法值（非 0/1）：退出码 2（value_parser 拒绝）
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "2",
    ])
    .current_dir(dir.path());
    cmd.assert().code(2);
}

#[test]
fn verdict_set_approved_blocked() {
    let dir = tempdir().unwrap();
    let (chapter_id, hash) = setup_chapter_with_content(&dir);

    // 写通过的检查 + 已达成裁决，再推进 drafting→review→approved
    write_passing_check(&dir, &chapter_id, &hash);
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
    ])
    .current_dir(dir.path());
    cmd.assert().success();

    // 推进状态：outline→drafting→review→approved
    for target in ["drafting", "review", "approved"] {
        let mut cmd = novel();
        cmd.args(["chapter", "status", &chapter_id, target])
            .current_dir(dir.path());
        cmd.assert().success();
    }

    // approved 后写入裁决：退出码 3，precondition_blocked
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        &chapter_id,
        "--node-id",
        "mp-1-001",
        "--achieved",
        "0",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();

    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "precondition_blocked");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 3);
}

#[test]
fn verdict_set_world_not_initialized() {
    let dir = tempdir().unwrap();

    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        "chapter-001",
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
        "--json",
    ])
    .current_dir(dir.path());
    let out = cmd.assert().code(3).get_output().clone();

    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "world_not_initialized");
}

/// 写入一条通过的 hard 检查结果（用于 approved 推进门禁）。
fn write_passing_check(dir: &tempfile::TempDir, chapter_id: &str, hash: &str) {
    let input = serde_json::json!({
        "contentHash": hash,
        "checks": [{
            "constraintType": "plot",
            "constraintScope": "world",
            "constraintName": "时间线一致性",
            "severity": "hard",
            "passed": true,
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
