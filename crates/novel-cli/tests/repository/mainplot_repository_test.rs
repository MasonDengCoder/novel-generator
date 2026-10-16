// 单元测试：mainplot_repository（task 11.1）
// 测试 create_node、list_nodes、get_node、update_node_achieved
//
// 说明：volumes.id 是 INTEGER AUTOINCREMENT，因此 volume_id 在实体与 SQL 中均为 i64。

use novel_cli::repositories::mainplot_repository;
use novel_cli::storage::connection::{open, run_migrations};
use rusqlite::{Connection, params};
use tempfile::tempdir;

/// 建立已迁移的临时数据库，并插入一个测试卷，返回 (conn, volume_id)。
fn setup_db() -> (Connection, i64) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    // 保留 dir：tempdir 在 drop 时删除目录；让连接生命周期与 dir 解耦会提前清理，
    // 因此把 dir 泄漏以保证文件在测试期间存活。
    std::mem::forget(dir);
    let conn = open(&db_path).unwrap();
    run_migrations(&conn, "2026-01-01T00:00:00Z").unwrap();

    conn.execute(
        "INSERT INTO volumes (name, volume_order, created_at, updated_at) VALUES ('测试卷', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    let volume_id: i64 = conn.last_insert_rowid();

    (conn, volume_id)
}

#[test]
fn create_node_inserts_record() {
    let (conn, vid) = setup_db();

    let node = mainplot_repository::create_node(&conn, vid, "主线节点1", Some("描述信息"), None).unwrap();
    assert_eq!(node.volume_id, vid);
    assert_eq!(node.title, "主线节点1");
    assert_eq!(node.description, Some("描述信息".to_string()));
    assert_eq!(node.achieved, 0);
    assert!(node.target_chapters.is_none());
    assert_eq!(node.sequence_number, 1);
}

#[test]
fn create_node_with_target_chapters() {
    let (conn, vid) = setup_db();

    // 创建测试章节
    conn.execute(
        "INSERT INTO chapters (id, title, content_path, content_hash, volume_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params!["ch-001", "第一章", "chapters/ch-001.md", "hash001", vid],
    )
    .unwrap();

    let target_chapters_json = r#"["ch-001"]"#;
    let node = mainplot_repository::create_node(&conn, vid, "主线节点2", None, Some(target_chapters_json)).unwrap();
    assert_eq!(node.target_chapters, Some(vec!["ch-001".to_string()]));
}

#[test]
fn create_node_assigns_increasing_sequence() {
    let (conn, vid) = setup_db();

    let n1 = mainplot_repository::create_node(&conn, vid, "节点A", None, None).unwrap();
    let n2 = mainplot_repository::create_node(&conn, vid, "节点B", None, None).unwrap();

    assert_eq!(n1.sequence_number, 1);
    assert_eq!(n2.sequence_number, 2);
    assert_ne!(n1.id, n2.id);
}

#[test]
fn list_nodes_returns_all_nodes_in_volume() {
    let (conn, vid) = setup_db();

    mainplot_repository::create_node(&conn, vid, "节点A", None, None).unwrap();
    mainplot_repository::create_node(&conn, vid, "节点B", None, None).unwrap();

    let nodes = mainplot_repository::list_nodes(&conn, vid).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].title, "节点A");
    assert_eq!(nodes[1].title, "节点B");
}

#[test]
fn list_nodes_empty_volume_returns_empty_vec() {
    let (conn, vid) = setup_db();

    let nodes = mainplot_repository::list_nodes(&conn, vid).unwrap();
    assert!(nodes.is_empty());
}

#[test]
fn get_node_returns_correct_node() {
    let (conn, vid) = setup_db();

    let created = mainplot_repository::create_node(&conn, vid, "目标节点", Some("详情"), None).unwrap();

    let fetched = mainplot_repository::get_node(&conn, &created.id).unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "目标节点");
    assert_eq!(fetched.description, Some("详情".to_string()));
}

#[test]
fn get_node_not_found_returns_error() {
    let (conn, _vid) = setup_db();

    let result = mainplot_repository::get_node(&conn, "non-existent-id");
    assert!(result.is_err());
    let err = result.unwrap_err();
    // get_node 在查不到时返回 MainplotNodeNotFound（退出码 43）
    assert_eq!(err.exit_code(), novel_cli::errors::ExitCode::MainplotNodeNotFound);
}

#[test]
fn update_node_achieved_changes_status() {
    let (conn, vid) = setup_db();

    let node = mainplot_repository::create_node(&conn, vid, "节点", None, None).unwrap();
    assert_eq!(node.achieved, 0);

    mainplot_repository::update_node_achieved(&conn, &node.id, 1).unwrap();

    let updated = mainplot_repository::get_node(&conn, &node.id).unwrap();
    assert_eq!(updated.achieved, 1);
}

#[test]
fn update_node_achieved_not_found_returns_error() {
    let (conn, _vid) = setup_db();

    let result = mainplot_repository::update_node_achieved(&conn, "no-such-node", 1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().exit_code(),
        novel_cli::errors::ExitCode::MainplotNodeNotFound
    );
}

#[test]
fn update_node_achieved_updates_timestamp() {
    let (conn, vid) = setup_db();

    let node = mainplot_repository::create_node(&conn, vid, "节点", None, None).unwrap();
    let original_updated_at = node.updated_at.clone();

    mainplot_repository::update_node_achieved(&conn, &node.id, 1).unwrap();

    let updated = mainplot_repository::get_node(&conn, &node.id).unwrap();
    assert_ne!(updated.updated_at, original_updated_at);
}
