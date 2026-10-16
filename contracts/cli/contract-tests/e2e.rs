// 端到端契约测试：init → info 全链路（task 6.1）。
//
// 这是跨命令的契约验证：从空目录 init 一个世界，再用 info 读回，断言：
// - init 与 info 均退出 0
// - `.novel/novel.db` 存在
// - info --json 输出符合 info.schema.json 结构且匹配 info.golden.json（忽略 createdAt）
// - info 人读输出含项目名
//
// 通过 crates/novel-cli/Cargo.toml 的 [[test]] 目标纳入 `cargo test`。

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use predicates;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn e2e_init_then_info_full_chain() {
    let dir = tempdir().unwrap();

    // init：成功创建世界。
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    assert!(dir.path().join(".novel/novel.db").is_file());

    // info 人读：退出 0，含项目名。
    novel()
        .args(["info", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("demo"));

    // info --json：退出 0，符合 schema 且匹配 golden（忽略 createdAt）。
    let mut cmd = novel();
    cmd.args(["info", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");

    let golden: Value =
        serde_json::from_str(include_str!("../examples/info.golden.json")).unwrap();
    assert_eq!(actual["schemaVersion"], golden["schemaVersion"]);
    assert_eq!(actual["projectName"], golden["projectName"]);
    assert_eq!(actual["dbPath"], golden["dbPath"]);
    assert_eq!(actual["counts"], golden["counts"]);
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);
}

#[test]
fn e2e_init_then_info_separate_directories_independent() {
    // 两个独立世界互不影响。
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    novel().args(["init", "alpha", "--path"]).arg(dir1.path()).assert().success();
    novel().args(["init", "beta", "--path"]).arg(dir2.path()).assert().success();

    let out1 = novel()
        .args(["info", "--json", "--path"])
        .arg(dir1.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let v1: Value = serde_json::from_slice(&out1.stdout).unwrap();
    assert_eq!(v1["projectName"], "alpha");

    let out2 = novel()
        .args(["info", "--json", "--path"])
        .arg(dir2.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let v2: Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(v2["projectName"], "beta");
}

#[test]
fn e2e_chapter_lifecycle_create_to_approved() {
    // 端到端：创建章节 → 写正文 → 写裁决 → 规则检查 → 推进 review → 推进 approved
    let dir = tempdir().unwrap();

    // init
    let mut cmd = novel();
    cmd.args(["init", "demo"]).current_dir(dir.path());
    cmd.assert().success();

    // chapter create
    let mut cmd = novel();
    cmd.args(["chapter", "create", "第一章"]).current_dir(dir.path());
    cmd.assert().success();

    // 缺陷3 修复：set-mainplot-landing 现在会校验主线节点存在，
    // 因此必须先建卷 1 与真实节点 mp-1-001（对齐 3 位补零 ID）。
    let mut cmd = novel();
    cmd.args(["volume", "create", "第一卷"]).current_dir(dir.path());
    cmd.assert().success();

    let mut cmd = novel();
    cmd.args(["mainplot", "node", "create", "1", "--title", "节点A"])
        .current_dir(dir.path());
    cmd.assert().success();

    // chapter update（写正文）
    let content_file = dir.path().join("content.md");
    std::fs::write(&content_file, "# 第一章\n\n这是正文内容。").unwrap();
    let mut cmd = novel();
    cmd.args(["chapter", "update", "chapter-001", "--content"])
        .arg(&content_file)
        .current_dir(dir.path());
    cmd.assert().success();

    // 读取 contentHash
    let mut cmd = novel();
    cmd.args(["chapter", "get", "chapter-001", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(chapter["status"].as_str().unwrap(), "outline");
    let hash = chapter["contentHash"].as_str().unwrap();

    // 写入通过的 hard 检查
    let check_input = serde_json::json!({
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
    let check_file = dir.path().join("check.json");
    std::fs::write(&check_file, check_input.to_string()).unwrap();
    let mut cmd = novel();
    cmd.args(["constraint", "check", "--chapter", "chapter-001", "--input"])
        .arg(&check_file)
        .current_dir(dir.path());
    cmd.assert().success();

    // 写入已达成裁决
    let mut cmd = novel();
    cmd.args([
        "verdict",
        "set-mainplot-landing",
        "chapter-001",
        "--node-id",
        "mp-1-001",
        "--achieved",
        "1",
    ])
    .current_dir(dir.path());
    cmd.assert().success();

    // 推进 outline → drafting → review → approved
    for target in ["drafting", "review", "approved"] {
        let mut cmd = novel();
        cmd.args(["chapter", "status", "chapter-001", target])
            .current_dir(dir.path());
        cmd.assert().success();
    }

    // 验证最终状态为 approved
    let mut cmd = novel();
    cmd.args(["chapter", "get", "chapter-001", "--json"])
        .current_dir(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let final_chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(final_chapter["status"].as_str().unwrap(), "approved");
}

/// e2e：setting render / constraint render 受控形态全链路（只读不推进状态机）。
///
/// 初始化世界 → 定义 setting/constraint → setting render 各 view → constraint render
/// 受控形态（world/volume/chapter）→ 验证只读：info counts 不变、chapter 状态不变。
#[test]
fn e2e_setting_constraint_render_readonly_chain() {
    let dir = tempdir().unwrap();
    // 1. 初始化世界
    novel()
        .args(["init", "render-e2e", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 2. 定义 setting（覆盖各 view 语义类）
    for (n, v) in [
        ("intro", "奇幻世界简介"),
        ("world-setting", "元素魔法体系"),
        ("style", "叙事冷峻"),
        ("four-poles-limit", "不可穿越时空"),
    ] {
        novel()
            .args(["setting", "set", n, v, "--path"])
            .arg(dir.path())
            .assert()
            .success();
    }

    // 3. 创建卷 + 章节 + 三层约束
    novel()
        .args(["volume", "create", "第一卷", "--order", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args(["chapter", "create", "开篇", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "world",
               "--name", "时间线一致", "--content", "全局不可矛盾", "--severity", "hard", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "volume:1",
               "--name", "卷一节奏", "--content", "卷一引入主线", "--severity", "warning", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 4. 渲染前快照：info counts
    let before = novel()
        .args(["info", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let before_info: Value = serde_json::from_slice(&before.stdout).unwrap();

    // 5. setting render 各 view（全受控名）
    for v in ["intro", "four-poles", "world", "style", "summary", "full", "llm"] {
        novel()
            .args(["setting", "render", "--view", v, "--json", "--path"])
            .arg(dir.path())
            .assert()
            .success();
    }

    // 6. constraint render 受控形态三种 scope
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "world", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "volume", "--volume-id", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "chapter", "--chapter-id", "chapter-001", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 7. 渲染后快照：info counts 不变（只读，不推进状态机）
    let after = novel()
        .args(["info", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let after_info: Value = serde_json::from_slice(&after.stdout).unwrap();
    assert_eq!(before_info["counts"], after_info["counts"], "render 应只读，counts 不变");

    // 8. chapter 状态仍为初始 outline（render 不推进状态机）
    let ch = novel()
        .args(["chapter", "get", "chapter-001", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let chapter: Value = serde_json::from_slice(&ch.stdout).unwrap();
    assert_eq!(chapter["status"].as_str().unwrap(), "outline", "render 不应推进章节状态");
}

/// chapter-context-packet 切片 e2e：初始化世界→建卷/章节/约束/裁决/检查/线索/主线/卷目标
/// → chapter context 三 view→只读不推进状态机全链路。
///
/// 章卷绑定通过 `chapter create --volume 1` 完成（chapter-create-volume change 后
/// CLI 原生支持章卷绑定，无需 rusqlite seed）。绑定后可验证 P1 装填
/// （mainplot/volume_goal/previous-residue 三层）。
#[test]
fn e2e_chapter_context_readonly_chain() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    novel().args(["init", "demo", "--path"]).arg(path).assert().success();

    // 1. 建卷 + 章节（--volume 绑定章卷）+ 写正文（先建章节，再建引用它的主线节点）。
    novel().args(["volume", "create", "第一卷", "--order", "1", "--path"]).arg(path).assert().success();
    let content_file = path.join("content.md");
    std::fs::write(&content_file, "# 第一章\n\n这是正文内容，用于 context 装填测试。").unwrap();
    novel().args(["chapter", "create", "第一章", "--volume", "1", "--path"]).arg(path).assert().success();
    novel()
        .args(["chapter", "update", "chapter-001", "--content"])
        .arg(&content_file)
        .args(["--path"])
        .arg(path)
        .assert()
        .success();

    // 2. 建主线节点（target-chapters 含 chapter-001）+ 卷目标 + 约束。
    novel()
        .args(["mainplot", "node", "create", "1", "--title", "启程", "--target-chapters", "chapter-001", "--path"])
        .arg(path)
        .assert()
        .success();
    novel()
        .args(["volume", "goal", "set", "1", "mp-1-001", "--description", "必达启程", "--path"])
        .arg(path)
        .assert()
        .success();
    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "world", "--name", "时间线一致", "--content", "时间线必须一致", "--severity", "hard", "--path"])
        .arg(path)
        .assert()
        .success();

    // 读 contentHash，写通过检查 + 已达成裁决。
    let out = novel()
        .args(["chapter", "get", "chapter-001", "--json", "--path"])
        .arg(path)
        .assert()
        .success()
        .get_output()
        .clone();
    let chapter: Value = serde_json::from_slice(&out.stdout).unwrap();
    let hash = chapter["contentHash"].as_str().unwrap();

    let check_input = serde_json::json!({
        "contentHash": hash,
        "checks": [{
            "constraintType": "plot",
            "constraintScope": "world",
            "constraintName": "时间线一致",
            "severity": "hard",
            "passed": true,
            "description": null,
            "location": null
        }]
    });
    let check_file = path.join("check.json");
    std::fs::write(&check_file, check_input.to_string()).unwrap();
    novel()
        .args(["constraint", "check", "--chapter", "chapter-001", "--input"])
        .arg(&check_file)
        .args(["--path"])
        .arg(path)
        .assert()
        .success();

    novel()
        .args(["verdict", "set-mainplot-landing", "chapter-001", "--node-id", "mp-1-001", "--achieved", "1", "--path"])
        .arg(path)
        .assert()
        .success();

    // 线索绑定（CLI 建卷内线索 + 绑章节）。
    novel()
        .args(["thread", "create", "1", "--name", "复仇线", "--description", "主线复仇", "--path"])
        .arg(path)
        .assert()
        .success();
    novel()
        .args(["thread", "bind", "th-1-001", "--chapter", "chapter-001", "--type", "develop", "--note", "推进复仇", "--path"])
        .arg(path)
        .assert()
        .success();

    // 3. 渲染前三 view → 断言只读：info counts 不变、chapter 状态仍 outline。
    let before = novel()
        .args(["info", "--json", "--path"])
        .arg(path)
        .assert()
        .success()
        .get_output()
        .clone();
    let before_info: Value = serde_json::from_slice(&before.stdout).unwrap();

    for view in ["writer", "verifier", "repair"] {
        let out = novel()
            .args(["chapter", "context", "chapter-001", "--view", view, "--budget", "60000", "--json", "--path"])
            .arg(path)
            .assert()
            .success()
            .get_output()
            .clone();
        let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");
        assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
        assert_eq!(actual["chapterId"].as_str().unwrap(), "chapter-001");
        assert_eq!(actual["view"].as_str().unwrap(), view);
        assert!(actual["fragments"].is_array());
        assert!(!actual["fragments"].as_array().unwrap().is_empty(), "P0/P1 应有装填 fragment");
        assert!(actual["overflow"]["omitted"].as_array().unwrap().iter().any(|o| o["reason"] == "no_data_backing_table"));
    }

    // 5. 只读断言：info counts 不变、chapter 状态仍 outline。
    let after = novel()
        .args(["info", "--json", "--path"])
        .arg(path)
        .assert()
        .success()
        .get_output()
        .clone();
    let after_info: Value = serde_json::from_slice(&after.stdout).unwrap();
    assert_eq!(before_info["counts"], after_info["counts"], "context 应只读，counts 不变");

    let ch = novel()
        .args(["chapter", "get", "chapter-001", "--json", "--path"])
        .arg(path)
        .assert()
        .success()
        .get_output()
        .clone();
    let chapter: Value = serde_json::from_slice(&ch.stdout).unwrap();
    assert_eq!(chapter["status"].as_str().unwrap(), "outline", "context 不应推进章节状态");
}
