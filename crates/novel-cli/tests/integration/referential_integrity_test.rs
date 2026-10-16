// 引用完整性测试（task 18.1-18.7）
//
// 说明：CLI 当前未提供 `volume delete` / `chapter delete` / `thread delete` 命令，
// 引用完整性由 SQLite FOREIGN KEY 约束在数据库层强制（V007__destiny_weaving.sql）。
// 因此本测试通过直接 SQL 验证 FK 约束拒绝非法引用，覆盖：
// - mainplot_nodes.volume_id 引用 volumes(id)
// - threads.volume_id 引用 volumes(id)
// - thread_bindings.thread_id 引用 threads(id)、chapter_id 引用 chapters(id)
// - emergences.volume_id 引用 volumes(id)、chapter_id 引用 chapters(id)
// - ON DELETE 行为：删除被引用的章节/卷时相关约束生效

#[path = "../common/mod.rs"]
mod common;

use novel_cli::repositories::{emergence_repository, mainplot_repository, thread_repository};
use novel_cli::domain::{BindingType, EmergenceStatus};
use rusqlite::params;

#[test]
fn mainplot_node_invalid_volume_id_rejected_by_fk() {
    let (conn, _target, _vid) = common::setup_db();

    // 不存在的 volume_id：FK 应拒绝
    let result = conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title) VALUES (?1, ?2, 1, '节点')",
        params!["mp-999-1", 9999],
    );
    assert!(result.is_err(), "不存在 volume_id 的插入应被 FK 拒绝");
}

#[test]
fn thread_invalid_volume_id_rejected_by_fk() {
    let (conn, _target, _vid) = common::setup_db();

    let result = conn.execute(
        "INSERT INTO threads (id, volume_id, name) VALUES (?1, ?2, '线索')",
        params!["th-999-1", 9999],
    );
    assert!(result.is_err(), "不存在 volume_id 的插入应被 FK 拒绝");
}

#[test]
fn thread_binding_invalid_thread_id_rejected_by_fk() {
    let (conn, _target, vid, ch1) = common::setup_db_with_chapter();

    // 章节存在，但 thread_id 不存在
    let result = conn.execute(
        "INSERT INTO thread_bindings (thread_id, chapter_id, binding_type) VALUES (?1, ?2, 'mention')",
        params!["th-nonexistent", ch1],
    );
    assert!(result.is_err(), "不存在 thread_id 的绑定应被 FK 拒绝");
    let _ = vid;
}

#[test]
fn thread_binding_invalid_chapter_id_rejected_by_fk() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    // 创建线索
    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();

    // thread 存在，但 chapter_id 不存在
    let result = conn.execute(
        "INSERT INTO thread_bindings (thread_id, chapter_id, binding_type) VALUES (?1, ?2, 'mention')",
        params![thread.id, "ch-nonexistent"],
    );
    assert!(result.is_err(), "不存在 chapter_id 的绑定应被 FK 拒绝");
}

#[test]
fn emergence_invalid_volume_id_rejected_by_fk() {
    let (conn, _target, _vid) = common::setup_db();

    let result = conn.execute(
        "INSERT INTO emergences (id, volume_id, title, description, status) VALUES (?1, ?2, '涌现', '描述', 'pending')",
        params!["em-999-1", 9999],
    );
    assert!(result.is_err(), "不存在 volume_id 的插入应被 FK 拒绝");
}

#[test]
fn emergence_invalid_chapter_id_rejected_by_fk() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    // volume 存在，但 chapter_id 不存在
    let result = conn.execute(
        "INSERT INTO emergences (id, volume_id, chapter_id, title, description, status) VALUES (?1, ?2, ?3, '涌现', '描述', 'pending')",
        params!["em-999-1", vid, "ch-nonexistent"],
    );
    assert!(result.is_err(), "不存在 chapter_id 的插入应被 FK 拒绝");
}

#[test]
fn emergence_valid_null_chapter_allowed() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    // chapter_id 为 NULL 是允许的（涌现可不关联章节）
    let result = conn.execute(
        "INSERT INTO emergences (id, volume_id, chapter_id, title, description, status) VALUES (?1, ?2, NULL, '涌现', '描述', 'pending')",
        params!["em-vid-1", vid],
    );
    assert!(result.is_ok(), "NULL chapter_id 应被允许");
}

#[test]
fn deleting_volume_with_mainplot_nodes_prevented_by_fk() {
    let (conn, _target, vid) = common::setup_db();

    // 创建主线节点引用该卷
    mainplot_repository::create_node(&conn, vid, "节点", None, None).unwrap();

    // 尝试删除该卷：FK 应阻止（mainplot_nodes.volume_id 无 ON DELETE CASCADE）
    let result = conn.execute("DELETE FROM volumes WHERE id = ?1", params![vid]);
    assert!(result.is_err(), "删除有主线节点的卷应被 FK 拒绝");
}

#[test]
fn deleting_volume_with_threads_prevented_by_fk() {
    let (conn, _target, vid) = common::setup_db();

    thread_repository::create_thread(&conn, vid, "线索", None).unwrap();

    let result = conn.execute("DELETE FROM volumes WHERE id = ?1", params![vid]);
    assert!(result.is_err(), "删除有线索的卷应被 FK 拒绝");
}

#[test]
fn deleting_volume_with_emergences_prevented_by_fk() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    emergence_repository::create(&conn, vid, None, "涌现", "描述").unwrap();

    let result = conn.execute("DELETE FROM volumes WHERE id = ?1", params![vid]);
    assert!(result.is_err(), "删除有涌现的卷应被 FK 拒绝");
}

#[test]
fn deleting_chapter_with_thread_bindings_prevented_by_fk() {
    let (conn, _target, vid, ch1) = common::setup_db_with_chapter();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();
    thread_repository::bind_chapter(&conn, &thread.id, &ch1, BindingType::Mention, None).unwrap();

    // 删除章节：thread_bindings.chapter_id 无 ON DELETE CASCADE，应被 FK 拒绝
    let result = conn.execute("DELETE FROM chapters WHERE id = ?1", params![ch1]);
    assert!(result.is_err(), "删除有线索绑定的章节应被 FK 拒绝");
}

#[test]
fn deleting_chapter_referenced_by_emergence_prevented_by_fk() {
    let (conn, _target, vid, ch1) = common::setup_db_with_chapter();

    // 创建引用该章节的涌现
    emergence_repository::create(&conn, vid, Some(&ch1), "涌现", "描述").unwrap();

    // 删除章节：emergences.chapter_id 无 ON DELETE CASCADE，应被 FK 拒绝
    let result = conn.execute("DELETE FROM chapters WHERE id = ?1", params![ch1]);
    assert!(result.is_err(), "删除被涌现引用的章节应被 FK 拒绝");
}

#[test]
fn deleting_thread_with_bindings_prevented_by_fk() {
    let (conn, _target, vid, ch1) = common::setup_db_with_chapter();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();
    thread_repository::bind_chapter(&conn, &thread.id, &ch1, BindingType::Mention, None).unwrap();

    // 删除线索：thread_bindings.thread_id 无 ON DELETE CASCADE，应被 FK 拒绝
    let result = conn.execute("DELETE FROM threads WHERE id = ?1", params![thread.id]);
    assert!(result.is_err(), "删除有线索绑定的线索应被 FK 拒绝");
}

#[test]
fn emergence_lifecycle_status_constraint_rejects_invalid_value() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    // CHECK(status IN (...)) 应拒绝非法状态
    let result = conn.execute(
        "INSERT INTO emergences (id, volume_id, title, description, status) VALUES (?1, ?2, '涌现', '描述', 'invalid_status')",
        params!["em-test-1", vid],
    );
    assert!(result.is_err(), "非法 status 值应被 CHECK 约束拒绝");
}

#[test]
fn thread_status_constraint_rejects_invalid_value() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    let result = conn.execute(
        "INSERT INTO threads (id, volume_id, name, status) VALUES (?1, ?2, '线索', 'invalid_status')",
        params!["th-test-1", vid],
    );
    assert!(result.is_err(), "非法 status 值应被 CHECK 约束拒绝");
}

#[test]
fn binding_type_constraint_rejects_invalid_value() {
    let (conn, _target, vid, ch1) = common::setup_db_with_chapter();

    let thread = thread_repository::create_thread(&conn, vid, "线索", None).unwrap();

    let result = conn.execute(
        "INSERT INTO thread_bindings (thread_id, chapter_id, binding_type) VALUES (?1, ?2, 'invalid_type')",
        params![thread.id, ch1],
    );
    assert!(result.is_err(), "非法 binding_type 值应被 CHECK 约束拒绝");
}

#[test]
fn emergence_status_machine_states_all_valid() {
    let (conn, _target, vid, ch1) = common::setup_db_with_chapter();

    // 验证所有合法状态都能被写入
    for (i, status) in ["pending", "rated", "decided", "executed", "archived"].iter().enumerate() {
        let result = conn.execute(
            "INSERT INTO emergences (id, volume_id, title, description, status) VALUES (?1, ?2, '涌现', '描述', ?3)",
            params![format!("em-valid-{}", i), vid, status],
        );
        assert!(result.is_ok(), "合法 status '{}' 应被接受", status);
    }
    let _ = ch1;
}

#[test]
fn mainplot_achieved_constraint_rejects_invalid_value() {
    let (conn, _target, vid, _) = common::setup_db_with_chapter();

    // CHECK(achieved IN (0, 1)) 应拒绝 2
    let result = conn.execute(
        "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved) VALUES (?1, ?2, 1, '节点', 2)",
        params!["mp-test-1", vid],
    );
    assert!(result.is_err(), "非法 achieved 值 2 应被 CHECK 约束拒绝");
}

// 确认 EmergenceStatus 仍被引用（避免 unused import 警告）
#[test]
fn _emergence_status_variants() {
    assert_eq!(EmergenceStatus::Pending.as_str(), "pending");
}
