// constraint 命令契约测试：constraint define/render/list（s01-project-bootstrap 切片）。
//
// 验证：
// - constraint define: 成功定义退出 0，重复定义退出 4 且错误码 "constraint_already_exists"
// - constraint render: 成功查询退出 0，约束不存在退出 6 且错误码 "constraint_not_found"
// - constraint render --json: 符合 constraint-render.schema.json，匹配 golden
// - constraint list: 空列表/过滤结果符合 constraint-list.schema.json
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
fn constraint_define_success() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // constraint define: 成功定义，静默成功，退出 0
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "时间线一致性",
            "--content",
            "所有章节的时间线必须保持一致",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn constraint_define_already_exists() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "时间线一致性",
            "--content",
            "规则内容",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // constraint define: 重复定义，退出 4，返回人读错误（该命令无 --json）
    let out = novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "时间线一致性",
            "--content",
            "另一个规则",
            "--severity",
            "warning",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .code(4)
        .get_output()
        .clone();

    let stderr = String::from_utf8(out.stderr).expect("stderr 应为 UTF-8");
    assert!(stderr.contains("constraint_already_exists") || stderr.contains("[constraint_already_exists]"));
    assert!(stderr.contains("退出码 4") || stderr.contains("4"));
}

#[test]
fn constraint_define_invalid_severity() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // constraint define: --severity 非法值，退出 2
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "规则",
            "--content",
            "内容",
            "--severity",
            "invalid-severity",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_success_json() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "时间线一致性",
            "--content",
            "所有章节的时间线必须保持一致，不得出现矛盾或逻辑冲突。",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // constraint render --json: 退出 0，符合 schema
    let out = novel()
        .args([
            "constraint",
            "render",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "时间线一致性",
            "--json",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");

    // 验证 schema 必填字段
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["constraintType"].as_str().unwrap(), "plot");
    assert_eq!(actual["scope"].as_str().unwrap(), "world");
    assert_eq!(actual["name"].as_str().unwrap(), "时间线一致性");
    assert_eq!(actual["severity"].as_str().unwrap(), "hard");
    assert!(actual["content"].as_str().unwrap().len() > 0);
    assert!(actual["createdAt"].as_str().unwrap().len() > 0);

    // 验证字段集合完整
    assert_eq!(actual.as_object().unwrap().keys().len(), 7);
}

#[test]
fn constraint_render_not_found() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // constraint render: 约束不存在，退出 6，返回错误对象
    let out = novel()
        .args([
            "constraint",
            "render",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "不存在",
            "--json",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .code(6)
        .get_output()
        .clone();

    let err: Value = serde_json::from_slice(&out.stdout).expect("错误输出应为 JSON");

    // 验证错误对象结构
    assert_eq!(err["errorId"].as_str().unwrap(), "constraint_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
    assert!(err["message"].as_str().unwrap().len() > 0);
}

#[test]
fn constraint_list_empty() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // constraint list --json: 空列表，退出 0
    let out = novel()
        .args(["constraint", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["constraints"].as_array().unwrap().len(), 0);
}

#[test]
fn constraint_list_non_empty_json() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "时间线一致性",
            "--content",
            "规则1",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "character",
            "--scope",
            "volume:v1",
            "--name",
            "角色行为一致性",
            "--content",
            "规则2",
            "--severity",
            "warning",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // constraint list --json: 非空列表，符合 schema
    let out = novel()
        .args(["constraint", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    let constraints = actual["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 2);

    // 验证每个元素包含必填字段且不含 content
    for item in constraints {
        assert!(item["constraintType"].as_str().unwrap().len() > 0);
        assert!(item["scope"].as_str().unwrap().len() > 0);
        assert!(item["name"].as_str().unwrap().len() > 0);
        assert!(["info", "warning", "hard"].contains(&item["severity"].as_str().unwrap()));
        assert!(item["createdAt"].as_str().unwrap().len() > 0);
        assert!(item.get("content").is_none()); // list 不含 content 字段
    }
}

#[test]
fn constraint_list_filter_by_scope() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "规则1",
            "--content",
            "内容",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "volume:v1",
            "--name",
            "规则2",
            "--content",
            "内容",
            "--severity",
            "warning",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // constraint list --scope volume: 前缀过滤
    let out = novel()
        .args(["constraint", "list", "--scope", "volume:", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let constraints = actual["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0]["scope"].as_str().unwrap(), "volume:v1");
}

#[test]
fn constraint_list_filter_by_severity() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "规则1",
            "--content",
            "内容",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();
    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "规则2",
            "--content",
            "内容",
            "--severity",
            "warning",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // constraint list --severity hard
    let out = novel()
        .args(["constraint", "list", "--severity", "hard", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();

    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let constraints = actual["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0]["severity"].as_str().unwrap(), "hard");
}

#[test]
fn constraint_list_human_readable() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // constraint list: 空列表人读输出
    novel()
        .args(["constraint", "list", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No constraints found."));

    novel()
        .args([
            "constraint",
            "define",
            "--type",
            "plot",
            "--scope",
            "world",
            "--name",
            "规则1",
            "--content",
            "内容",
            "--severity",
            "hard",
            "--path",
        ])
        .arg(dir.path())
        .assert()
        .success();

    // constraint list: 非空列表人读输出包含表头
    novel()
        .args(["constraint", "list", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Type"))
        .stdout(predicate::str::contains("Scope"))
        .stdout(predicate::str::contains("plot"))
        .stdout(predicate::str::contains("world"));
}

// ===== constraint render 受控形态契约测试（setting-constraint-render 切片）=====

/// 建世界 + 卷1 + chapter-001（不关联卷）+ 三层约束（world / volume:1 / chapter:chapter-001）。
fn setup_constraint_scope_world() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
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
    // world 约束
    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "world",
               "--name", "时间线一致", "--content", "全局时间线不可矛盾", "--severity", "hard", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    // volume:1 约束
    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "volume:1",
               "--name", "卷一节奏", "--content", "卷一需在第三章前引入主线", "--severity", "warning", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    // chapter:chapter-001 约束
    novel()
        .args(["constraint", "define", "--type", "character", "--scope", "chapter:chapter-001",
               "--name", "主角出场", "--content", "第一章必须让主角登场", "--severity", "hard", "--path"])
        .arg(dir.path())
        .assert()
        .success();
    dir
}

#[test]
fn constraint_render_view_world_scope_json() {
    let dir = setup_constraint_scope_world();
    // --scope world：仅全局约束
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "world", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["schemaVersion"].as_str().unwrap(), CURRENT_SCHEMA_VERSION.to_string().as_str());
    assert_eq!(actual["view"].as_str().unwrap(), "llm");
    assert_eq!(actual["scope"].as_str().unwrap(), "world");
    let constraints = actual["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0]["scope"].as_str().unwrap(), "world");
    // sourceRefs 格式 constraint:<type>:<scope>:<name>
    assert!(
        constraints[0]["sourceRefs"][0]
            .as_str()
            .unwrap()
            .starts_with("constraint:")
    );
}

#[test]
fn constraint_render_view_chapter_scope_json() {
    let dir = setup_constraint_scope_world();
    // --scope chapter --chapter-id chapter-001：三层叠加（chapter 未关联 volume，故 world + chapter 两层）
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "chapter",
               "--chapter-id", "chapter-001", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["scope"].as_str().unwrap(), "chapter");
    assert_eq!(actual["chapterId"].as_str().unwrap(), "chapter-001");
    let scopes: Vec<&str> = actual["constraints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scope"].as_str().unwrap())
        .collect();
    assert!(scopes.contains(&"world"));
    assert!(scopes.contains(&"chapter:chapter-001"));
}

#[test]
fn constraint_render_view_volume_scope_json() {
    let dir = setup_constraint_scope_world();
    // --scope volume --volume-id 1：world + volume:1 两层
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "volume",
               "--volume-id", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["scope"].as_str().unwrap(), "volume");
    assert_eq!(actual["volumeId"].as_i64().unwrap(), 1);
    let scopes: Vec<&str> = actual["constraints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["scope"].as_str().unwrap())
        .collect();
    assert!(scopes.contains(&"world"));
    assert!(scopes.contains(&"volume:1"));
}

#[test]
fn constraint_render_view_human_scope_human_readable() {
    let dir = setup_constraint_scope_world();
    // --view human 人读输出
    novel()
        .args(["constraint", "render", "--view", "human", "--scope", "world", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("View: human"))
        .stdout(predicate::str::contains("Scope: world"));
}

#[test]
fn constraint_render_view_scope_chapter_missing_chapter_id() {
    let dir = setup_constraint_scope_world();
    // --scope chapter 缺 --chapter-id → 退出码 2
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "chapter", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_view_scope_volume_missing_locator() {
    let dir = setup_constraint_scope_world();
    // --scope volume 缺 --volume-id 和 --chapter-id → 退出码 2
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "volume", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_view_invalid_view() {
    let dir = setup_constraint_scope_world();
    // 非法 view → 退出码 2，code=usage
    let out = novel()
        .args(["constraint", "render", "--view", "invalid", "--scope", "world", "--json", "--path"])
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
fn constraint_render_view_invalid_scope() {
    let dir = setup_constraint_scope_world();
    // 受控 scope 非法 → 退出码 2
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "invalid", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_view_and_single_form_mutex() {
    let dir = setup_constraint_scope_world();
    // --view 与 --type/--name 互斥 → 退出码 2
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "world",
               "--type", "plot", "--name", "时间线一致", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_view_only_no_scope() {
    let dir = setup_constraint_scope_world();
    // 仅 --view 无 --scope → 退出码 2
    novel()
        .args(["constraint", "render", "--view", "llm", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_view_chapter_id_not_found() {
    let dir = setup_constraint_scope_world();
    // --chapter-id 不存在 → 退出码 6，code=chapter_not_found
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "chapter",
               "--chapter-id", "chapter-999", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(6)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "chapter_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
}

#[test]
fn constraint_render_view_volume_id_not_found() {
    let dir = setup_constraint_scope_world();
    // --volume-id 不存在 → 退出码 6，errorId=volume_not_found
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "volume",
               "--volume-id", "999", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(6)
        .get_output()
        .clone();
    let err: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(err["errorId"].as_str().unwrap(), "volume_not_found");
    assert_eq!(err["exitCode"].as_i64().unwrap(), 6);
}

#[test]
fn constraint_render_view_world_not_initialized() {
    let dir = tempdir().unwrap();
    // 世界未初始化 → 退出码 3
    novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "world", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .code(3);
}

#[test]
fn constraint_render_view_empty_scope() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();
    // world scope 无约束 → 空数组，退出码 0
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "world", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(actual["constraints"].as_array().unwrap().len(), 0);
}

#[test]
fn constraint_render_single_form_unchanged() {
    let dir = setup_constraint_scope_world();
    // 既有单条精确形态行为不变（无 --view）→ 退出码 0，输出单条约束
    let out = novel()
        .args(["constraint", "render", "--type", "plot", "--scope", "world",
               "--name", "时间线一致", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    // 单条形态 JSON 顶层无 view/scope(constraints)，而是 constraintType 等扁平字段
    assert_eq!(actual["constraintType"].as_str().unwrap(), "plot");
    assert_eq!(actual["name"].as_str().unwrap(), "时间线一致");
    assert!(actual.get("view").is_none(), "单条形态不应含 view 字段");
}

// ===== C12: severity 三档枚举（info/warning/hard）契约测试 =====

#[test]
fn constraint_define_severity_info_success() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // --severity info：成功定义，退出 0，静默
    novel()
        .args([
            "constraint", "define", "--type", "plot", "--scope", "world",
            "--name", "info约束", "--content", "提示性规则",
            "--severity", "info", "--path",
        ])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn constraint_define_severity_warning_success() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    // --severity warning：成功定义，退出 0，静默
    novel()
        .args([
            "constraint", "define", "--type", "plot", "--scope", "world",
            "--name", "warning约束", "--content", "建议性规则",
            "--severity", "warning", "--path",
        ])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn constraint_define_severity_soft_rejected() {
    // 旧值 soft 不保留为别名，必须退出码 2（不保留别名决策）
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args([
            "constraint", "define", "--type", "plot", "--scope", "world",
            "--name", "soft约束", "--content", "旧值",
            "--severity", "soft", "--path",
        ])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn constraint_render_severity_values_three_tier() {
    // define info/warning/hard 三条，render 后 severity 字段逐一正确
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    for (name, sev) in [("info规则", "info"), ("warning规则", "warning"), ("hard规则", "hard")] {
        novel()
            .args([
                "constraint", "define", "--type", "plot", "--scope", "world",
                "--name", name, "--content", "x", "--severity", sev, "--path",
            ])
            .arg(dir.path())
            .assert()
            .success();
    }

    // render --view llm --scope world 应列出三档
    let out = novel()
        .args(["constraint", "render", "--view", "llm", "--scope", "world", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let constraints = actual["constraints"].as_array().unwrap();
    let mut severities: Vec<String> = constraints.iter()
        .map(|c| c["severity"].as_str().unwrap().to_string())
        .collect();
    severities.sort();
    assert_eq!(severities, vec!["hard".to_string(), "info".to_string(), "warning".to_string()]);
}

#[test]
fn constraint_list_filter_by_severity_warning() {
    // 两条约束（warning + hard），--severity warning 应仅返回 1 条
    let dir = tempdir().unwrap();
    novel().args(["init", "test", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "world",
               "--name", "w1", "--content", "x", "--severity", "warning", "--path"])
        .arg(dir.path()).assert().success();
    novel()
        .args(["constraint", "define", "--type", "plot", "--scope", "world",
               "--name", "h1", "--content", "x", "--severity", "hard", "--path"])
        .arg(dir.path()).assert().success();

    let out = novel()
        .args(["constraint", "list", "--severity", "warning", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .clone();
    let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
    let constraints = actual["constraints"].as_array().unwrap();
    assert_eq!(constraints.len(), 1);
    assert_eq!(constraints[0]["severity"].as_str().unwrap(), "warning");
}
