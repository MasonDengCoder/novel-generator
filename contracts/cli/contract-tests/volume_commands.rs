// 契约测试：验证 `novel volume` 命令输出符合 CLI Contract。
//
// 覆盖任务：
// - 10.1: volume create 成功创建与重复卷名报错
// - 10.2: volume info 人读与 JSON 输出
// - 10.3: volume list 空列表与多卷排序
// - 10.4: volume seal 封卷成功、章节阻断、干运行模式
// - 10.5: volume archive 归档成功与未封卷拒绝
// - 10.6: volume goals 目标设置、查询、删除

use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::tempdir;

fn novel() -> Command {
    Command::cargo_bin("novel").unwrap()
}

/// schemaVersion MUST 为十进制整数字符串（contracts/cli/json-output.md §2.2），
/// 不绑定具体版本号以跟随 PRAGMA user_version 单一数据源演进。
fn assert_decimal_schema_version(json: &Value) {
    let sv = json["schemaVersion"]
        .as_str()
        .expect("schemaVersion 必须为字符串");
    assert!(
        sv.chars().all(|c| c.is_ascii_digit()) && !sv.is_empty(),
        "schemaVersion 必须为非空十进制整数字符串，实际：{sv}"
    );
}

/// 轻量 schema 校验：校验 volume-seal-check.schema.json 声明的必填字段、类型与枚举。
/// 遵循 info_json.rs 的字段级断言模式（本切片未引入完整 JSON Schema 引擎）。
fn assert_volume_seal_check_conforms_schema(v: &Value) {
    let obj = v.as_object().expect("seal check 顶层应为 JSON 对象");
    for key in ["schemaVersion", "volumeId", "dryRun", "canSeal", "checkResults"] {
        assert!(obj.contains_key(key), "seal check 缺少必填字段 {key}");
    }
    assert_decimal_schema_version(v);
    assert!(v["volumeId"].is_i64(), "volumeId 应为整数");
    assert!(v["dryRun"].is_boolean(), "dryRun 应为布尔");
    assert!(v["canSeal"].is_boolean(), "canSeal 应为布尔");
    let checks = v["checkResults"]
        .as_array()
        .expect("checkResults 应为数组");
    for c in checks {
        let c = c.as_object().expect("检查项应为对象");
        for key in ["checkType", "passed", "message", "details"] {
            assert!(c.contains_key(key), "检查项缺少字段 {key}");
        }
        assert!(c["checkType"].is_string(), "checkType 应为字符串");
        assert!(c["passed"].is_boolean(), "passed 应为布尔");
        assert!(c["message"].is_string(), "message 应为字符串");
        assert!(c["details"].is_array(), "details 应为数组");
    }
}

// 10.1: 创建卷成功与重复卷名报错
#[test]
fn test_volume_create_success() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args(["volume", "create", "第一卷", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("创建卷成功"));
}

#[test]
fn test_volume_create_duplicate_name() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 重复卷名应失败，退出码 4（资源冲突）
    novel()
        .args(["volume", "create", "第一卷", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(4);
}

// 10.2: volume info 人读与 JSON 输出
#[test]
fn test_volume_info_human_readable() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    novel()
        .args(["volume", "info", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("第一卷"));
}

#[test]
fn test_volume_info_json_schema() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel()
        .args(["volume", "create", "第一卷", "--description", "测试卷", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    let output = novel()
        .args(["volume", "info", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");

    assert_decimal_schema_version(&json);
    assert_eq!(json["volumeId"], 1);
    assert_eq!(json["name"], "第一卷");
    assert_eq!(json["description"], "测试卷");
    assert_eq!(json["status"], "active");
    assert!(json["createdAt"].is_string());
}

// 10.3: volume list 空列表与多卷排序
#[test]
fn test_volume_list_empty() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["volume", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert_decimal_schema_version(&json);
    assert_eq!(json["volumes"].as_array().unwrap().len(), 0);
}

#[test]
fn test_volume_list_multiple_sorted() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    novel().args(["volume", "create", "第三卷", "--order", "3", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--order", "1", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第二卷", "--order", "2", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["volume", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    let volumes = json["volumes"].as_array().unwrap();
    assert_eq!(volumes.len(), 3);
    assert_eq!(volumes[0]["name"], "第一卷");
    assert_eq!(volumes[1]["name"], "第二卷");
    assert_eq!(volumes[2]["name"], "第三卷");
}

// 10.4: volume seal 干运行模式（空卷检查）
#[test]
fn test_volume_seal_dry_run_no_chapters_table() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 干运行：空卷应通过检查（输出警告）
    novel()
        .args(["volume", "seal", "1", "--dry-run", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("卷内无章节"));
}

// 10.5: volume archive 归档成功与未封卷拒绝
#[test]
fn test_volume_archive_not_sealed() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 未封卷状态下归档应失败
    novel()
        .args(["volume", "archive", "1", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(3); // 前置条件阻断
}

// 10.6: volume goals 目标设置、查询、删除
#[test]
fn test_volume_goal_set_get_remove() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 缺陷4 修复：volume goal set 现在会校验主线节点存在，需先插入 node-001。
    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) VALUES ('node-001', 1, 1, '主线节点A', 0, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();

    // 设置目标
    novel()
        .args([
            "volume", "goal", "set", "1", "node-001",
            "--description", "完成主线A",
            "--path"
        ])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("设置卷目标成功"));

    // 查询目标（JSON）
    let output = novel()
        .args(["volume", "goal", "get", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert_decimal_schema_version(&json);
    assert_eq!(json["volumeId"], 1);
    let goals = json["goals"].as_array().unwrap();
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0]["mainplotNodeId"], "node-001");
    assert_eq!(goals[0]["isMandatory"], true);

    // 删除目标
    novel()
        .args(["volume", "goal", "remove", "1", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("删除卷目标成功"));

    // 验证目标已删除
    let output2 = novel()
        .args(["volume", "goal", "get", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json2: Value = serde_json::from_slice(&output2).unwrap();
    assert_eq!(json2["goals"].as_array().unwrap().len(), 0);
}

#[test]
fn test_volume_goal_invalid_node_id() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 非法节点 ID 格式
    novel()
        .args(["volume", "goal", "set", "1", "node@invalid", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(2); // 参数校验失败
}

// E2E 测试（任务 12.1-12.2）

#[test]
fn test_e2e_volume_crud_flow() {
    // 12.1: init → volume create → volume info → volume list
    let dir = tempdir().unwrap();

    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let output = novel()
        .args(["volume", "info", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["name"], "第一卷");

    let output2 = novel()
        .args(["volume", "list", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json2: Value = serde_json::from_slice(&output2).unwrap();
    assert_eq!(json2["volumes"].as_array().unwrap().len(), 1);
}

#[test]
fn test_e2e_volume_goal_flow() {
    // 12.2: 创建卷 → 设置目标 → 查询目标 → 删除目标
    let dir = tempdir().unwrap();

    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 缺陷4 修复：volume goal set 现在会校验主线节点存在，需先插入 node-001。
    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) VALUES ('node-001', 1, 1, '主线节点A', 0, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();

    novel()
        .args(["volume", "goal", "set", "1", "node-001", "--description", "完成主线A", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    let output = novel()
        .args(["volume", "goal", "get", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["goals"].as_array().unwrap().len(), 1);

    novel()
        .args(["volume", "goal", "remove", "1", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    let output2 = novel()
        .args(["volume", "goal", "get", "1", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json2: Value = serde_json::from_slice(&output2).unwrap();
    assert_eq!(json2["goals"].as_array().unwrap().len(), 0);
}

// E2E 测试：任务 12.3 - 封卷成功 → 归档成功

#[test]
fn test_e2e_volume_seal_with_approved_chapters() {
    let dir = tempdir().unwrap();

    // 初始化世界
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();

    // 创建卷
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 手动插入 approved 章节（fixture）
    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        []
    ).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
         VALUES ('chapter-002', '第二章', 'content/chapters/chapter-002.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        []
    ).unwrap();

    // 封卷应成功
    novel()
        .args(["volume", "seal", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 归档应成功
    novel()
        .args(["volume", "archive", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();
}

// E2E 测试：任务 12.4 - 封卷失败（章节阻断）

#[test]
fn test_e2e_volume_seal_blocked_by_drafting_chapters() {
    let dir = tempdir().unwrap();

    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    // 手动插入 drafting 章节
    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'drafting', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        []
    ).unwrap();

    // 封卷应失败（前置条件阻断，退出码 3）
    novel()
        .args(["volume", "seal", "1", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("章节"));
}

// E2E 测试：任务 12.5 - 干运行 → 修复 → 封卷成功

#[test]
fn test_e2e_volume_seal_dry_run_then_fix() {
    let dir = tempdir().unwrap();

    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();

    // 插入 drafting 章节
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'drafting', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        []
    ).unwrap();

    // 干运行封卷检查，应返回阻断信息（但退出 0）
    let output = novel()
        .args(["volume", "seal", "1", "--dry-run", "--json", "--path"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_volume_seal_check_conforms_schema(&json);
    assert_eq!(json["canSeal"], false);

    // 修复：手动更新章节状态为 approved
    conn.execute("UPDATE chapters SET status = 'approved' WHERE id = 'chapter-001'", []).unwrap();

    // 正式封卷应成功
    novel()
        .args(["volume", "seal", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();
}

// s07：封卷真实检查 — 涌现未处置阻断（退出码 3）

#[test]
fn test_volume_seal_blocked_by_unhandled_emergence() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    // approved 章节
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        [],
    ).unwrap();
    // 未处置涌现（pending）
    conn.execute(
        "INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at) \
         VALUES ('em-1-1', 1, '涌现A', '描述', '2026-07-01T00:00:00Z', 'pending', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();

    // 封卷应被阻断（退出码 3）
    novel()
        .args(["volume", "seal", "1", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(3);
}

// s07：封卷真实检查 — 必达目标未达成阻断（退出码 3）

#[test]
fn test_volume_seal_blocked_by_unachieved_mandatory_goal() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        [],
    ).unwrap();
    // 必达目标 + 主线节点 achieved=0
    conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) \
         VALUES ('node-001', 1, 1, '主线节点A', 0, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO volume_goals (volume_id, mainplot_node_id, description, is_mandatory, created_at) \
         VALUES (1, 'node-001', '必达目标A', 1, '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();

    // 封卷应被阻断（退出码 3）
    novel()
        .args(["volume", "seal", "1", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(3);
}

// s07：封卷全部通过推进到 sealing（退出码 0），并验证状态为 sealing 中间态

#[test]
fn test_volume_seal_all_pass_advances_to_sealing() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        [],
    ).unwrap();

    // 封卷应成功（退出码 0），推进到 sealing
    novel()
        .args(["volume", "seal", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    // 验证卷状态为 sealing（s07 中间态，不再直推 sealed）
    let status: String = conn
        .query_row("SELECT status FROM volumes WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(status, "sealing");
    let sealed_at: Option<String> = conn
        .query_row("SELECT sealed_at FROM volumes WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    assert!(sealed_at.is_some(), "sealing 态应已记录 sealed_at");

    // 随后归档应成功（archive 接受 sealing，生成监测报告后推进到 archived）
    novel()
        .args(["volume", "archive", "1", "--path"])
        .arg(dir.path())
        .assert()
        .success();

    let status2: String = conn
        .query_row("SELECT status FROM volumes WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(status2, "archived");
    // 监测报告文件应已生成
    let report = dir.path().join(".novel/content/reports/volume-1.md");
    assert!(report.exists(), "监测报告文件应已生成：{}", report.display());
}

// s07：归档冻结 — 归档卷后 chapter update 被拒（退出码 3，volume_already_archived）

#[test]
fn test_archived_volume_blocks_chapter_update() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    // approved 章节并写入正文文件（让 update 能走到归档冻结 guard）
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        [],
    ).unwrap();
    let content_path = dir.path().join(".novel/content/chapters/chapter-001.md");
    std::fs::create_dir_all(content_path.parent().unwrap()).unwrap();
    std::fs::write(&content_path, "原始正文").unwrap();

    // 封卷 + 归档
    novel().args(["volume", "seal", "1", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "archive", "1", "--path"]).arg(dir.path()).assert().success();

    // 归档后尝试 update 正文文件应被拒（退出码 3）
    let new_content = dir.path().join("new-content.md");
    std::fs::write(&new_content, "新正文").unwrap();
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "chapter", "update", "chapter-001",
            "--content", new_content.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["status"], "failed");
    assert_eq!(error["errorId"], "volume_already_archived");
    assert_eq!(error["exitCode"], 3);
}

// s07：归档冻结 — 归档卷后 emergence rate 被拒（退出码 3，volume_already_archived）

#[test]
fn test_archived_volume_blocks_emergence_rate() {
    let dir = tempdir().unwrap();
    novel().args(["init", "test-world", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "create", "第一卷", "--path"]).arg(dir.path()).assert().success();

    let db_path = dir.path().join(".novel/novel.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at) \
         VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', 'dummy-hash', 100, 'approved', 1, '2026-06-30T00:00:00Z', '2026-06-30T00:00:00Z')",
        [],
    ).unwrap();
    // pending 涌现（归档前归为 pending，归档后尝试 rate 应被冻结 guard 拒绝）
    conn.execute(
        "INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at) \
         VALUES ('em-1-1', 1, '涌现A', '描述', '2026-07-01T00:00:00Z', 'pending', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
        [],
    ).unwrap();
    // 先把涌现处置完以便能通过封卷检查
    conn.execute("UPDATE emergences SET status = 'executed' WHERE id = 'em-1-1'", []).unwrap();

    novel().args(["volume", "seal", "1", "--path"]).arg(dir.path()).assert().success();
    novel().args(["volume", "archive", "1", "--path"]).arg(dir.path()).assert().success();

    // 归档后尝试 rate 涌现应被拒（退出码 3，volume_already_archived）
    novel()
        .args(["emergence", "rate", "em-1-1", "--rating", "5", "--reason", "评分", "--path"])
        .arg(dir.path())
        .assert()
        .failure()
        .code(3);
}
