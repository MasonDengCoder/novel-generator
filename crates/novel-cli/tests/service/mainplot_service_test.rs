// 单元测试：mainplot_service（task 13.1）
// 测试 service 层业务逻辑与错误处理：create_node、list_nodes、get_node、update_node_achieved

#[path = "../common/mod.rs"]
mod common;

use novel_cli::errors::ExitCode;
use novel_cli::services::mainplot_service;

#[test]
fn create_node_success() {
    let (conn, target, vid) = common::setup_db();

    let node = mainplot_service::create_node(&target, vid, "主线节点", Some("描述"), None).unwrap();
    assert_eq!(node.volume_id, vid);
    assert_eq!(node.title, "主线节点");
}

#[test]
fn create_node_volume_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = mainplot_service::create_node(&target, 9999, "节点", None, None);
    assert!(result.is_err());
    // volume_not_found 走 NotFound 分支（退出码 6）
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::NotFound);
}

#[test]
fn create_node_with_existing_target_chapters() {
    let (conn, target, vid, ch1, ch2) = common::setup_db_with_two_chapters();

    let node = mainplot_service::create_node(
        &target,
        vid,
        "主线",
        None,
        Some(vec![ch1.clone(), ch2.clone()]),
    )
    .unwrap();
    assert_eq!(node.target_chapters, Some(vec![ch1, ch2]));
}

#[test]
fn create_node_with_nonexistent_target_chapter_rejected() {
    let (conn, target, vid, _) = common::setup_db_with_chapter();

    let result = mainplot_service::create_node(&target, vid, "主线", None, Some(vec!["ch-999".to_string()]));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("ch-999"));
}

#[test]
fn list_nodes_returns_all() {
    let (conn, target, vid) = common::setup_db();

    mainplot_service::create_node(&target, vid, "节点A", None, None).unwrap();
    mainplot_service::create_node(&target, vid, "节点B", None, None).unwrap();

    let nodes = mainplot_service::list_nodes(&target, vid).unwrap();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn list_nodes_volume_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = mainplot_service::list_nodes(&target, 9999);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::NotFound);
}

#[test]
fn get_node_returns_correct_node() {
    let (conn, target, vid) = common::setup_db();

    let created = mainplot_service::create_node(&target, vid, "目标", None, None).unwrap();
    let fetched = mainplot_service::get_node(&target, &created.id).unwrap();
    assert_eq!(fetched.id, created.id);
}

#[test]
fn get_node_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = mainplot_service::get_node(&target, "no-such-node");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::MainplotNodeNotFound);
}

#[test]
fn update_node_achieved_success() {
    let (conn, target, vid) = common::setup_db();

    let node = mainplot_service::create_node(&target, vid, "节点", None, None).unwrap();
    mainplot_service::update_node_achieved(&target, &node.id, 1).unwrap();

    let updated = mainplot_service::get_node(&target, &node.id).unwrap();
    assert_eq!(updated.achieved, 1);
}

#[test]
fn update_node_achieved_invalid_value() {
    let (conn, target, vid) = common::setup_db();

    let node = mainplot_service::create_node(&target, vid, "节点", None, None).unwrap();
    let result = mainplot_service::update_node_achieved(&target, &node.id, 5);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::Usage);
}

#[test]
fn update_node_achieved_not_found() {
    let (conn, target, _vid) = common::setup_db();

    let result = mainplot_service::update_node_achieved(&target, "no-such-node", 1);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().exit_code(), ExitCode::MainplotNodeNotFound);
}
