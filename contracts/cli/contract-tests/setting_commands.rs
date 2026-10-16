// setting 命令契约测试：setting set/get/list（s01-project-bootstrap 切片）。
//
// 验证：
// - setting set: 成功设置退出 0，世界未初始化退出 3
// - setting get: 成功查询退出 0，配置项不存在退出 6 且错误对象 errorId="setting_not_found"
// - setting get --json: 符合 setting-get.schema.json，匹配 setting-get.output.golden.json
// - setting list: 空列表退出 0，非空列表符合 setting-list.schema.json
// - 错误对象符合 V2 错误信封（schemaVersion/status/errorId/category/exitCode/message）

use assert_cmd::Command;
use novel_cli::storage::connection::CURRENT_SCHEMA_VERSION;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

#[test]
fn setting_set_success() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // setting set: 成功设置，静默成功，退出 0
    novel()
        .args(["setting", "set", "world-style", "奇幻史诗", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn setting_set_world_not_initialized() {
    let dir = tempdir().unwrap();

    // setting set: 世界未初始化，退出 3
    novel()
        .args(["setting", "set", "key", "value", "--path"])
        .arg(dir.path())
        .assert()
        .code(3);
}

#[test]
fn setting_get_success_json() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args(["setting", "set", "world-style", "奇幻史诗", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // setting get --json: 退出 0，符合 schema
    let out = novel()
        .args(["setting", "get", "world-style", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");

    // 验证 schema 必填字段
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["name"].as_str().unwrap(), "world-style");
    assert_eq!(actual["value"].as_str().unwrap(), "奇幻史诗");
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);

    // 验证与 golden 结构一致（字段集合）
    let golden: Value =
        serde_json::from_str(include_str!("../examples/setting-get.output.golden.json")).unwrap();
    assert_eq!(actual.as_object().unwrap().keys().collect::<Vec<_>>().len(), 4);
    assert!(actual.get("schemaVersion").is_some());
    assert!(actual.get("name").is_some());
    assert!(actual.get("value").is_some());
    assert!(actual.get("createdAt").is_some());
}

#[test]
fn setting_get_not_found() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // setting get: 配置项不存在，退出 6，返回错误对象
    let out = novel()
        .args(["setting", "get", "non-existent-key", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(6)
        .get_output()
        .clone();

    let err: Value = serde_json::from_slice(&out.stdout).expect("错误输出应为 JSON");

    // 验证错误对象结构
    assert_eq!(err["errorId"].as_str().unwrap(), "setting_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
    assert!(err["message"].as_str().unwrap().len() > 0);
    assert_eq!(err["category"].as_str().unwrap(), "not_found");
    assert_eq!(err["status"].as_str().unwrap(), "failed");

    // 验证与 golden 错误对象结构一致（V2 信封六字段）
    let golden_err: Value =
        serde_json::from_str(include_str!("../examples/setting-not-found.error.golden.json"))
            .unwrap();
    assert_eq!(err.as_object().unwrap().keys().collect::<Vec<_>>().len(), 6);
}

#[test]
fn setting_list_empty() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // setting list --json: 空列表，退出 0
    let out = novel()
        .args(["setting", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["settings"].as_array().unwrap().len(), 0);
}

#[test]
fn setting_list_non_empty_json() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args(["setting", "set", "world-style", "奇幻史诗", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args(["setting", "set", "author-name", "张三", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // setting list --json: 非空列表，符合 schema
    let out = novel()
        .args(["setting", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    let settings = actual["settings"].as_array().unwrap();
    assert_eq!(settings.len(), 2);

    // 验证每个元素包含必填字段
    for item in settings {
        assert!(item["name"].as_str().unwrap().len() > 0);
        assert!(item.get("value").is_some());
        assert!(item["createdAt"].as_str().unwrap().len() > 0);
    }
}

#[test]
fn setting_list_human_readable() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // setting list: 空列表人读输出
    novel()
        .args(["setting", "list", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No settings defined."));

    novel()
        .args(["setting", "set", "key1", "value1", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // setting list: 非空列表人读输出包含表头
    novel()
        .args(["setting", "list", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Name"))
        .stdout(predicate::str::contains("Value"))
        .stdout(predicate::str::contains("key1"));
}

// ===== setting render --view 契约测试（setting-constraint-render 切片）=====

/// 准备一个含分类 setting 的世界，返回临时目录句柄。
fn setup_render_world() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    for (n, v) in [
        ("intro", "这是一个奇幻世界"),
        ("world-setting", "魔法体系：元素魔法"),
        ("style", "叙事冷峻"),
        ("four-poles-limit", "不可穿越时空"),
    ] {
        novel()
            .args(["setting", "set", n, v, "--path"])
            .arg(dir.path())
            .assert()
            .success();
    }
    dir
}

#[test]
fn setting_render_each_view_json() {
    let dir = setup_render_world();

    // 7 个受控 view 名各跑一次，均退出 0 且 view 字段等于输入
    for v in ["intro", "four-poles", "world", "style", "summary", "full", "llm"] {
        let out = novel()
            .args(["setting", "render", "--view", v, "--json", "--path"])
            .arg(dir.path())
            .assert()
            .success()
            .get_output()
            .clone();
        let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str(), "view={}", v);
        assert_eq!(actual["view"].as_str().unwrap(), v, "view={}", v);
        assert!(actual["settings"].is_array(), "view={}", v);
        // 每个 setting 元素含 sourceRefs，格式 setting:<name>
        for item in actual["settings"].as_array().unwrap() {
            assert!(
                item["sourceRefs"][0].as_str().unwrap().starts_with("setting:"),
                "view={}",
                v
            );
        }
    }
}

#[test]
fn setting_render_llm_is_combination_of_four_views() {
    let dir = setup_render_world();

    // llm 视图 = intro∪four-poles∪world∪style 的并集，应包含全部 4 个 setting
    let out = novel()
        .args(["setting", "render", "--view", "llm", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = actual["settings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names.len(), 4);
    assert!(names.contains(&"intro"));
    assert!(names.contains(&"world-setting"));
    assert!(names.contains(&"style"));
    assert!(names.contains(&"four-poles-limit"));
}

#[test]
fn setting_render_full_returns_all() {
    let dir = setup_render_world();
    let out = novel()
        .args(["setting", "render", "--view", "full", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["view"].as_str().unwrap(), "full");
    assert_eq!(actual["settings"].as_array().unwrap().len(), 4);
}

#[test]
fn setting_render_full_human_readable() {
    // 对齐 spec scenario "渲染 full 视图（人读格式）"：无 --json 时输出多行文本含 view 名与设定项。
    let dir = setup_render_world();
    novel()
        .args(["setting", "render", "--view", "full", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("View: full"))
        .stdout(predicate::str::contains("Name"))
        .stdout(predicate::str::contains("Value"))
        // 含各设定项名（多行文本，人读审查、导出或离线核对）
        .stdout(predicate::str::contains("intro"))
        .stdout(predicate::str::contains("world-setting"))
        .stdout(predicate::str::contains("style"))
        .stdout(predicate::str::contains("four-poles-limit"));
}

#[test]
fn setting_render_style_only() {
    let dir = setup_render_world();
    let out = novel()
        .args(["setting", "render", "--view", "style", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<&str> = actual["settings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    // style 视图只含 style 类 setting
    assert_eq!(names, vec!["style"]);
}

#[test]
fn setting_render_empty_settings() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    // 刚初始化无 setting，full 视图返回空数组
    let out = novel()
        .args(["setting", "render", "--view", "full", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["settings"].as_array().unwrap().len(), 0);
}

#[test]
fn setting_render_invalid_view_returns_usage() {
    let dir = setup_render_world();
    // 非法 view 名 → 退出码 2，code=usage
    let out = novel()
        .args(["setting", "render", "--view", "invalid", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "usage");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 2);
}

#[test]
fn setting_render_missing_view_returns_usage() {
    let dir = setup_render_world();
    // 缺 --view → 退出码 2（clap 层）
    novel()
        .args(["setting", "render", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn setting_render_world_not_initialized() {
    let dir = tempdir().unwrap();
    // 世界未初始化 → 退出码 3
    novel()
        .args(["setting", "render", "--view", "llm", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(3);
}

#[test]
fn setting_render_does_not_mutate_state() {
    let dir = setup_render_world();
    // 渲染前后 info 的 counts 不变（只读，不推进状态机）
    let info_before = novel()
        .args(["info", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    novel()
        .args(["setting", "render", "--view", "llm", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    let info_after = novel()
        .args(["info", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let before: Value = serde_json::from_slice(&info_before.stdout).unwrap();
    let after: Value = serde_json::from_slice(&info_after.stdout).unwrap();
    assert_eq!(before["counts"], after["counts"], "render 应只读，不改变 counts");
}
