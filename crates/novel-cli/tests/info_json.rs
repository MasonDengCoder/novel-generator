// 验证 info JSON 契约（task 5.4）。
// 断言：`novel info --json` 输出符合 info.schema.json 的结构与类型约束，
// 且匹配 info.golden.json（忽略 createdAt 易变字段）。

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

fn init_world(dir: &std::path::Path) {
    novel()
        .args(["init", "demo", "--path"])
        .arg(dir)
        .assert()
        .success();
}

/// 轻量 schema 校验：校验 info.schema.json 中声明的必填字段、类型与枚举。
/// 本切片未引入完整 JSON Schema 引擎（design 假设：machine-verifiable 校验为可选验收），
/// 此函数以字段级断言兑现"输出符合 schema"。
fn assert_conforms_schema(v: &Value) {
    let obj = v.as_object().expect("顶层应为 JSON 对象");
    for key in ["schemaVersion", "projectName", "dbPath", "createdAt", "counts"] {
        assert!(obj.contains_key(key), "缺少必填字段 {key}");
    }
    assert!(obj["schemaVersion"].is_string() && !obj["schemaVersion"].as_str().unwrap().is_empty());
    assert!(obj["projectName"].is_string() && !obj["projectName"].as_str().unwrap().is_empty());
    assert!(obj["dbPath"].is_string() && !obj["dbPath"].as_str().unwrap().is_empty());
    assert!(obj["createdAt"].is_string() && !obj["createdAt"].as_str().unwrap().is_empty());

    let counts = &obj["counts"];
    let counts = counts.as_object().expect("counts 应为对象");
    for key in ["chapters", "volumes", "settings", "constraints", "mainplotNodes"] {
        assert!(counts.contains_key(key), "counts 缺少字段 {key}");
        assert!(counts[key].is_i64(), "counts.{key} 应为整数");
        assert!(counts[key].as_i64().unwrap() >= 0, "counts.{key} 应 >= 0");
    }
}

#[test]
fn info_json_conforms_schema_and_matches_golden() {
    let dir = tempdir().unwrap();
    init_world(dir.path());

    let mut cmd = novel();
    cmd.args(["info", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();

    let actual: Value = serde_json::from_slice(&out.stdout).expect("stdout 应为单一合法 JSON");
    assert_conforms_schema(&actual);

    // 比对 golden（忽略 createdAt）。
    let golden: Value =
        serde_json::from_str(include_str!("../../../contracts/cli/examples/info.golden.json"))
            .unwrap();
    assert_eq!(actual["schemaVersion"], golden["schemaVersion"]);
    assert_eq!(actual["projectName"], golden["projectName"]);
    assert_eq!(actual["dbPath"], golden["dbPath"]);
    assert_eq!(actual["counts"], golden["counts"]);
    // createdAt 易变，仅校验非空（已在 schema 校验中覆盖）。
}

#[test]
fn info_json_is_single_object_no_trailing() {
    let dir = tempdir().unwrap();
    init_world(dir.path());

    let mut cmd = novel();
    cmd.args(["info", "--json", "--path"]).arg(dir.path());
    let out = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    // 整段必须可解析为单一 JSON 对象（无多余文本）。
    let _: Value = serde_json::from_str(stdout.trim()).expect("应为单一合法 JSON 对象");
}
