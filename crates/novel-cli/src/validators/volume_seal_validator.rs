// volume_seal_validator 模块：封卷前置条件检查。

use crate::errors::CliError;
use crate::repositories::volume_repository;
use rusqlite::Connection;

/// 封卷检查结果详情。
#[derive(Debug, Clone)]
pub struct SealCheckResult {
    pub check_type: String,
    pub passed: bool,
    pub message: String,
    pub details: Vec<String>,
}

/// 检查卷是否可以封卷。
///
/// 检查项：
/// 1. 卷内所有章节状态必须为 `approved`
/// 2. 卷内所有涌现记录必须已处置（真实查询 `emergences` 表，状态须 `executed`/`archived`）
/// 3. 卷的所有必达目标必须已达成（真实查询 `mainplot_nodes.achieved`，必达须 `1`）
///
/// 表查询异常返回 70（内部错误），不降级为 warn 通过（CLAUDE.md §10）。
pub fn can_seal(conn: &Connection, volume_id: i64) -> Result<Vec<SealCheckResult>, CliError> {
    let mut results = Vec::new();

    // 检查 1: 章节状态
    let chapter_check = check_chapters_approved(conn, volume_id)?;
    results.push(chapter_check);

    // 检查 2: 涌现处置状态（真实检查，s07 升级）
    let emergence_check = check_emergences_handled(conn, volume_id)?;
    results.push(emergence_check);

    // 检查 3: 卷目标达成（真实检查，s07 升级）
    let goal_check = check_goals_achieved(conn, volume_id)?;
    results.push(goal_check);

    Ok(results)
}

/// 检查卷内所有章节状态为 `approved`。
fn check_chapters_approved(conn: &Connection, volume_id: i64) -> Result<SealCheckResult, CliError> {
    // 检查 chapters 表是否存在
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='chapters'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查 chapters 表失败：{}", e)))?;

    if !table_exists {
        // 章节表不存在：返回错误（无法验证章节状态）
        return Err(CliError::storage(
            "章节表不存在，无法验证章节状态。请确保 s02-chapter-content-core 切片已实现。".to_string(),
        ));
    }

    // 查询卷内章节状态
    let mut stmt = conn
        .prepare("SELECT id, status FROM chapters WHERE volume_id = ?1")
        .map_err(|e| CliError::storage(format!("查询章节状态失败：{}", e)))?;

    let chapters: Vec<(String, String)> = stmt
        .query_map([volume_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CliError::storage(format!("查询章节失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析章节数据失败：{}", e)))?;

    if chapters.is_empty() {
        // 空卷：输出警告但通过
        eprintln!("警告：卷 {} 内无章节", volume_id);
        return Ok(SealCheckResult {
            check_type: "章节状态".to_string(),
            passed: true,
            message: "卷内无章节（已警告）".to_string(),
            details: vec![],
        });
    }

    // 收集未 approved 的章节
    let mut not_approved = Vec::new();
    for (chapter_id, status) in &chapters {
        if status != "approved" {
            not_approved.push(format!("章节 {} 状态为 {}", chapter_id, status));
        }
    }

    if not_approved.is_empty() {
        Ok(SealCheckResult {
            check_type: "章节状态".to_string(),
            passed: true,
            message: format!("所有 {} 个章节已 approved", chapters.len()),
            details: vec![],
        })
    } else {
        Ok(SealCheckResult {
            check_type: "章节状态".to_string(),
            passed: false,
            message: format!("{} 个章节未 approved", not_approved.len()),
            details: not_approved,
        })
    }
}

/// 检查涌现处置状态（真实检查，s07 升级）。
///
/// 查询 `emergences` 表，卷内涌现状态须为 `executed` 或 `archived`（已处置）；
/// `pending`/`rated`/`decided` 为未处置终态前的中间状态，必须阻断封卷。
/// 表查询异常返回 70（内部错误），**不**降级为 warn 通过（CLAUDE.md §10 hard 阻断原则）。
fn check_emergences_handled(conn: &Connection, volume_id: i64) -> Result<SealCheckResult, CliError> {
    // 查询卷内所有涌现的状态。s06 已创建 emergences 表，表缺失属迁移异常 → hard 70。
    let mut stmt = conn
        .prepare("SELECT id, status FROM emergences WHERE volume_id = ?1")
        .map_err(|e| CliError::storage(format!("涌现表查询失败，无法验证涌现处置状态：{}", e)))?;

    let emergences: Vec<(String, String)> = stmt
        .query_map([volume_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CliError::storage(format!("涌现表查询失败，无法验证涌现处置状态：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("涌现表查询失败，无法验证涌现处置状态：{}", e)))?;

    if emergences.is_empty() {
        // 无涌现记录：通过检查。
        return Ok(SealCheckResult {
            check_type: "涌现处置".to_string(),
            passed: true,
            message: "卷内无涌现记录".to_string(),
            details: vec![],
        });
    }

    // 收集未处置的涌现（状态非 executed/archived）。
    let mut not_handled = Vec::new();
    for (emergence_id, status) in &emergences {
        if status != "executed" && status != "archived" {
            not_handled.push(format!("涌现 {} 状态为 {}", emergence_id, status));
        }
    }

    if not_handled.is_empty() {
        Ok(SealCheckResult {
            check_type: "涌现处置".to_string(),
            passed: true,
            message: format!("所有 {} 个涌现已处置", emergences.len()),
            details: vec![],
        })
    } else {
        Ok(SealCheckResult {
            check_type: "涌现处置".to_string(),
            passed: false,
            message: format!("{} 个涌现未处置（需推进到 executed 或 archived）", not_handled.len()),
            details: not_handled,
        })
    }
}

/// 检查卷目标达成状态（真实检查，s07 升级）。
///
/// 查询 `mainplot_nodes` 表的 `achieved` 字段，必达目标对应主线节点须 `achieved=1`；
/// 可选目标未达成仅提示不阻断。表查询异常返回 70（内部错误），**不**降级为 warn 通过。
fn check_goals_achieved(conn: &Connection, volume_id: i64) -> Result<SealCheckResult, CliError> {
    // 获取卷的所有目标（必达 + 可选）。
    let goals = volume_repository::get_goals_by_volume(conn, volume_id)?;
    let mandatory_goals: Vec<_> = goals.iter().filter(|g| g.is_mandatory).collect();

    if mandatory_goals.is_empty() {
        // 无必达目标：通过检查（可选目标即便未达成也不阻断）。
        let optional_unachieved = collect_unachieved_optional(conn, &goals)?;
        let mut message = "无必达目标".to_string();
        if !optional_unachieved.is_empty() {
            message = format!("无必达目标；可选目标未达成（不阻断）：{}", optional_unachieved.join("、"));
        }
        return Ok(SealCheckResult {
            check_type: "卷目标达成".to_string(),
            passed: true,
            message,
            details: vec![],
        });
    }

    // 对每个必达目标，查询对应主线节点的 achieved 字段。
    // mainplot_nodes 表由 s06 创建，表缺失属迁移异常 → hard 70。
    let mut unachieved_mandatory = Vec::new();
    for goal in &mandatory_goals {
        let achieved: i64 = conn
            .query_row(
                "SELECT achieved FROM mainplot_nodes WHERE id = ?1",
                [&goal.mainplot_node_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    // 必达目标引用的主线节点不存在 → mainplot_node_not_found（退出码 43），
                    // 比 storage(70) 更准确；该情况在正常流程已被 goal set 的前置校验挡住，
                    // 仅在数据库被手工篡改时才可能触达。
                    CliError::mainplot_node_not_found(format!(
                        "必达目标引用的主线节点 {} 不存在（卷目标指向了悬空节点）",
                        goal.mainplot_node_id
                    ))
                }
                _ => CliError::storage(format!("主线节点表查询失败，无法验证卷目标达成：{}", e)),
            })?;

        if achieved != 1 {
            unachieved_mandatory.push(format!(
                "必达目标 {} (mainplot-node {}) 未达成",
                goal.id, goal.mainplot_node_id
            ));
        }
    }

    let optional_unachieved = collect_unachieved_optional(conn, &goals)?;

    if unachieved_mandatory.is_empty() {
        // 必达全部达成：通过。可选未达成仅提示。
        let mut details = vec![];
        let mut message = format!("所有 {} 个必达目标已达成", mandatory_goals.len());
        if !optional_unachieved.is_empty() {
            message = format!("{}；可选目标未达成（不阻断）：{}", message, optional_unachieved.join("、"));
            details = optional_unachieved;
        }
        Ok(SealCheckResult {
            check_type: "卷目标达成".to_string(),
            passed: true,
            message,
            details,
        })
    } else {
        Ok(SealCheckResult {
            check_type: "卷目标达成".to_string(),
            passed: false,
            message: format!("{} 个必达目标未达成", unachieved_mandatory.len()),
            details: unachieved_mandatory,
        })
    }
}

/// 收集未达成的可选目标（仅提示，不阻断）。
fn collect_unachieved_optional(conn: &Connection, goals: &[volume_repository::VolumeGoal]) -> Result<Vec<String>, CliError> {
    let mut unachieved = Vec::new();
    for goal in goals.iter().filter(|g| !g.is_mandatory) {
        let achieved: i64 = conn
            .query_row(
                "SELECT achieved FROM mainplot_nodes WHERE id = ?1",
                [&goal.mainplot_node_id],
                |row| row.get(0),
            )
            .unwrap_or(0); // 可选目标引用的节点缺失不阻断（降级为"未达成"提示）
        if achieved != 1 {
            unachieved.push(format!("可选目标 {} ({})", goal.id, goal.mainplot_node_id));
        }
    }
    Ok(unachieved)
}

/// 检查卷是否可以归档（s07：接受 sealing 或 sealed 状态）。
///
/// 已归档的卷阻断（`volume_already_archived`）；非 sealing/sealed 阻断（`volume_not_sealed`）。
pub fn can_archive(conn: &Connection, volume_id: i64) -> Result<(), CliError> {
    let volume = volume_repository::get_volume_by_id(conn, volume_id)?;

    if volume.status == "archived" {
        return Err(CliError::volume_already_archived(format!(
            "卷 {} 已归档，无法重复归档",
            volume_id
        )));
    }

    if volume.status != "sealing" && volume.status != "sealed" {
        return Err(CliError::volume_not_sealed(format!(
            "卷 {} 状态为 {}，只有 sealing/sealed 状态的卷可以归档",
            volume_id, volume.status
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::volume_repository;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;
    use tempfile::tempdir;

    fn setup() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn, "2026-06-30T00:00:00Z").unwrap();
        (conn, dir)
    }

    #[test]
    fn test_can_archive_sealed_volume() {
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();
        volume_repository::update_volume_status(&conn, volume_id, "sealed").unwrap();

        assert!(can_archive(&conn, volume_id).is_ok());
    }

    #[test]
    fn test_can_archive_active_volume_fails() {
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();

        let result = can_archive(&conn, volume_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_goals_achieved_no_goals() {
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();

        let result = check_goals_achieved(&conn, volume_id).unwrap();
        assert_eq!(result.check_type, "卷目标达成");
        assert!(result.passed);
    }

    #[test]
    fn test_check_goals_achieved_mandatory_node_missing_fails_hard() {
        // s07 真实检查：必达目标引用的 mainplot_node 不存在 → 缺陷修复后返回
        // mainplot_node_not_found（退出码 43，属引用/前置问题），不再以 storage(70) 崩溃。
        // 正常流程中该悬空引用已被 goal set 的前置校验挡住，此处仅验证封卷时的清晰错误类别。
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();
        volume_repository::add_goal(&conn, volume_id, "node-001", None, true).unwrap();

        let result = check_goals_achieved(&conn, volume_id);
        assert!(result.is_err(), "必达目标引用不存在的节点应 hard 失败");
        let err = result.unwrap_err();
        assert_eq!(err.exit_code(), crate::errors::ExitCode::MainplotNodeNotFound);
    }

    #[test]
    fn test_check_goals_achieved_mandatory_node_achieved_passes() {
        // s07 真实检查：必达目标引用的节点 achieved=1 → 通过。
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();
        volume_repository::add_goal(&conn, volume_id, "node-001", None, true).unwrap();
        // 插入对应主线节点并标记 achieved=1。
        conn.execute(
            "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) \
             VALUES ('node-001', ?1, 1, '主线节点A', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [volume_id],
        ).unwrap();

        let result = check_goals_achieved(&conn, volume_id).unwrap();
        assert!(result.passed);
        assert!(result.message.contains("已达成"), "消息应反映达成：{}", result.message);
    }

    #[test]
    fn test_check_goals_achieved_mandatory_node_unachieved_blocks() {
        // s07 真实检查：必达目标引用的节点 achieved=0 → 阻断（passed=false）。
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();
        volume_repository::add_goal(&conn, volume_id, "node-001", None, true).unwrap();
        conn.execute(
            "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) \
             VALUES ('node-001', ?1, 1, '主线节点A', 0, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [volume_id],
        ).unwrap();

        let result = check_goals_achieved(&conn, volume_id).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("未达成"));
        assert!(!result.details.is_empty());
    }

    #[test]
    fn test_check_emergences_handled_no_records_passes() {
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();

        let result = check_emergences_handled(&conn, volume_id).unwrap();
        assert!(result.passed);
        assert!(result.message.contains("无涌现"));
    }

    #[test]
    fn test_check_emergences_handled_pending_blocks() {
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();
        // 直接插入一个 pending 涌现（绕过 service 仅用于检查测试）。
        conn.execute(
            "INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at) \
             VALUES ('em-1-1', ?1, '涌现A', '描述', '2026-07-01T00:00:00Z', 'pending', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [volume_id],
        ).unwrap();

        let result = check_emergences_handled(&conn, volume_id).unwrap();
        assert!(!result.passed);
        assert!(result.message.contains("未处置"));
        assert!(!result.details.is_empty());
    }

    #[test]
    fn test_check_emergences_handled_executed_passes() {
        let (conn, _dir) = setup();

        let volume_id = volume_repository::create_volume(&conn, "第一卷", None, 1).unwrap();
        conn.execute(
            "INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at) \
             VALUES ('em-1-1', ?1, '涌现A', '描述', '2026-07-01T00:00:00Z', 'executed', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z')",
            [volume_id],
        ).unwrap();

        let result = check_emergences_handled(&conn, volume_id).unwrap();
        assert!(result.passed);
        assert!(result.message.contains("已处置"));
    }
}

