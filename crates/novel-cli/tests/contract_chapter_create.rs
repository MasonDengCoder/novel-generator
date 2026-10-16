// 契约测试：验证 `novel chapter create` 输出符合 JSON schema（task 10.1）。

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn test_chapter_create_json_schema() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 创建章节
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");

    // 验证 JSON schema
    assert!(json.get("schemaVersion").is_some(), "必须包含 schemaVersion");
    assert!(json.get("id").is_some(), "必须包含 id");
    assert!(json.get("title").is_some(), "必须包含 title");
    assert!(json.get("createdAt").is_some(), "必须包含 createdAt");

    // 验证 id 格式
    let id = json["id"].as_str().unwrap();
    assert!(
        id.starts_with("chapter-"),
        "id 应以 chapter- 开头，实际：{}",
        id
    );

    // 验证 title
    assert_eq!(json["title"].as_str().unwrap(), "第一章");

    // 验证 schemaVersion 是字符串
    assert!(json["schemaVersion"].is_string());
}

#[test]
fn test_chapter_create_human_output() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 创建章节（人读输出）
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("测试章节")
        .assert()
        .success()
        .stdout(predicate::str::contains("创建成功"))
        .stdout(predicate::str::contains("chapter-001"))
        .stdout(predicate::str::contains("测试章节"));
}

#[test]
fn test_chapter_create_empty_title() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 创建章节（空标题应失败）
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("")
        .assert()
        .failure()
        .code(2); // E_INVALID_ARGUMENT
}

#[test]
fn test_chapter_create_project_not_initialized() {
    let dir = tempdir().unwrap();

    // 未初始化项目时创建章节应失败
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(dir.path())
        .arg("chapter")
        .arg("create")
        .arg("标题")
        .assert()
        .failure()
        .code(3); // world_not_initialized 返回退出码 3
}

// chapter 组子命令必须支持 --path 显式指向非 CWD 的世界（commands.md `[--path <dir>]`）。
#[test]
fn test_chapter_create_with_explicit_path() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化项目
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 在与项目目录不同的 CWD 下，通过 --path 显式指向该项目世界创建章节
    let other_dir = tempdir().unwrap();
    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(other_dir.path())
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .arg("--path")
        .arg(project_dir)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("chapter-001"));
}

// --path 指向不存在的目录应退出码 2（参数错误）。
#[test]
fn test_chapter_create_path_not_exist_exits_2() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("does-not-exist");

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(dir.path())
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .arg("--path")
        .arg(&missing)
        .assert()
        .failure()
        .code(2);
}

// --volume <id> 成功归属卷：JSON 含 volumeId，chapters.volume_id 已写入。
#[test]
fn test_chapter_create_with_volume_json() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();

    // 初始化 + 建一个 active 卷（ID 自增为 1）
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("volume")
        .arg("create")
        .arg("第一卷")
        .arg("--order")
        .arg("1")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // chapter create --volume 1 --json
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .arg("--volume")
        .arg("1")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert!(json.get("volumeId").is_some(), "传入 --volume 时必须含 volumeId");
    assert_eq!(json["volumeId"].as_str().unwrap(), "1", "volumeId 应为字符串化整数 1");

    // 确认 chapters.volume_id 已写入（chapter get 回显 volumeId）
    let get_output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("get")
        .arg("chapter-001")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let get_json: Value = serde_json::from_slice(&get_output).expect("get 输出应为有效 JSON");
    assert_eq!(get_json["volumeId"].as_str().unwrap(), "1", "chapter get 应回显 volumeId=1");
}

// 不传 --volume 时 JSON 不出现 volumeId 字段（零回归）。
#[test]
fn test_chapter_create_without_volume_has_no_volumeid() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("独立章")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("输出应为有效 JSON");
    assert!(json.get("volumeId").is_none(), "不传 --volume 时不应出现 volumeId 字段");
}

// --volume 指向不存在的卷应退出码 6（volume_not_found）。
#[test]
fn test_chapter_create_volume_not_found_exits_6() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .arg("--volume")
        .arg("999")
        .arg("--json")
        .assert()
        .failure()
        .code(6)
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("错误输出应为有效 JSON");
    assert_eq!(json["errorId"].as_str().unwrap(), "volume_not_found");
    assert_eq!(json["exitCode"].as_i64().unwrap(), 6);
}

// --volume 非整数应退出码 2（参数错误）。
#[test]
fn test_chapter_create_volume_not_integer_exits_2() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("第一章")
        .arg("--volume")
        .arg("abc")
        .arg("--json")
        .assert()
        .failure()
        .code(2);
}


// 卷非 active（sealed）状态时拒绝加章：退出码 3 + volume_already_sealed。
// 用 rusqlite 直接置 sealed 绕过封卷门禁（测试 fixture 非权威方式，同 setup-sealed-volume.sh 模式）。
#[test]
fn test_chapter_create_volume_sealed_exits_3() {
    let dir = tempdir().unwrap();
    let project_dir = dir.path();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("init")
        .arg("test-project")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("volume")
        .arg("create")
        .arg("第一卷")
        .arg("--order")
        .arg("1")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();
    Command::cargo_bin("novel")
        .unwrap()
        .arg("volume")
        .arg("create")
        .arg("第二卷")
        .arg("--order")
        .arg("2")
        .arg("--path")
        .arg(project_dir)
        .assert()
        .success();

    // 直插置卷 2 为 sealed（测试 fixture，绕过封卷门禁）
    let db_path = project_dir.join(".novel/novel.db");
    let conn = rusqlite::Connection::open(&db_path).expect("打开测试 db 失败");
    conn.execute(
        "UPDATE volumes SET status='sealed', sealed_at='2026-07-01T00:00:00Z' WHERE id=2",
        [],
    )
    .expect("置 sealed 失败");
    drop(conn);

    // --volume 2 指向 sealed 卷应拒码3 + volume_already_sealed
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(project_dir)
        .arg("chapter")
        .arg("create")
        .arg("章")
        .arg("--volume")
        .arg("2")
        .arg("--json")
        .assert()
        .failure()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("错误输出应为有效 JSON");
    assert_eq!(json["errorId"].as_str().unwrap(), "volume_already_sealed");
    assert_eq!(json["exitCode"].as_i64().unwrap(), 3);
}

// 世界未初始化 + --volume：应返回 world_not_initialized（码3），而非 volume_not_found。
// 保护"检查顺序"不变量：世界初始化检查先于卷校验。
#[test]
fn test_chapter_create_volume_world_not_initialized_exits_3() {
    let dir = tempdir().unwrap();

    // 未 init 目录传 --volume 1：应码3 world_not_initialized（不因卷不存在返回码6）
    let output = Command::cargo_bin("novel")
        .unwrap()
        .current_dir(dir.path())
        .arg("chapter")
        .arg("create")
        .arg("标题")
        .arg("--volume")
        .arg("1")
        .arg("--json")
        .assert()
        .failure()
        .code(3)
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output).expect("错误输出应为有效 JSON");
    assert_eq!(
        json["errorId"].as_str().unwrap(),
        "world_not_initialized",
        "世界未初始化应先于卷校验返回 world_not_initialized，而非 volume_not_found"
    );
    assert_eq!(json["exitCode"].as_i64().unwrap(), 3);
}
