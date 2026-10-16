// volume_service 模块：卷级业务逻辑。
//
// 职责（CLAUDE.md §6 / 不变量 12）：service 持有 DB-open，command 层仅传 `target: &Path`。
// 公开函数接收 `target: &Path`，内部首行 `open_existing_at(target)?`，再把 `&Connection` / `&mut Connection`
// 传给 repository / validator / 内部 helper（advance_to_sealed / generate_monitoring_report）。

use crate::errors::CliError;
use crate::repositories::volume_repository::{self, Volume, VolumeGoal};
use crate::storage::connection::open_existing_at;
use crate::validators::{volume_validator, volume_seal_validator};
use rusqlite::Connection;
use std::path::Path;

/// 创建新卷。
pub fn create_volume(
    target: &Path,
    name: &str,
    description: Option<&str>,
    volume_order: i64,
) -> Result<i64, CliError> {
    let conn = open_existing_at(target)?;

    // 参数验证
    volume_validator::validate_volume_order(volume_order)?;
    volume_validator::validate_unique_name(&conn, name)?;

    // 调用 repository 创建
    volume_repository::create_volume(&conn, name, description, volume_order)
}

/// 获取单个卷信息。
pub fn get_volume(target: &Path, volume_id: i64) -> Result<Volume, CliError> {
    let conn = open_existing_at(target)?;
    volume_repository::get_volume_by_id(&conn, volume_id)
}

/// 列举所有卷。
pub fn list_volumes(target: &Path) -> Result<Vec<Volume>, CliError> {
    let conn = open_existing_at(target)?;
    volume_repository::list_volumes(&conn)
}

/// 封卷（dry_run 模式只检查不修改状态）。
pub fn seal_volume(
    target: &Path,
    volume_id: i64,
    dry_run: bool,
) -> Result<Vec<volume_seal_validator::SealCheckResult>, CliError> {
    let conn = open_existing_at(target)?;
    // 获取卷信息
    let volume = volume_repository::get_volume_by_id(&conn, volume_id)?;

    // 检查卷状态
    if volume.status == "sealed" || volume.status == "archived" {
        return Err(CliError::volume_already_sealed(format!(
            "卷 {} 状态为 {}，无法重复封卷",
            volume_id, volume.status
        )));
    }

    // 执行封卷前置条件检查
    let check_results = volume_seal_validator::can_seal(&conn, volume_id)?;

    // 判断是否所有检查通过
    let all_passed = check_results.iter().all(|r| r.passed);

    // 干运行模式：只返回检查结果，不修改状态（无论检查通过与否）
    if dry_run {
        return Ok(check_results);
    }

    // 非干运行模式：检查失败时返回错误
    if !all_passed {
        // 收集失败原因
        let failed: Vec<String> = check_results
            .iter()
            .filter(|r| !r.passed)
            .flat_map(|r| {
                let mut lines = vec![format!("【{}】{}", r.check_type, r.message)];
                lines.extend(r.details.iter().map(|d| format!("  - {}", d)));
                lines
            })
            .collect();

        return Err(CliError::chapters_not_approved(format!(
            "封卷前置条件未满足：\n{}",
            failed.join("\n")
        )));
    }

    // 更新卷状态到 sealing（中间态，s07 修正 s05 直推 sealed）
    volume_repository::update_volume_status(&conn, volume_id, "sealing")?;

    // 记录封卷时间
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;
    volume_repository::set_volume_sealed_at(&conn, volume_id, &timestamp)?;

    Ok(check_results)
}

/// 推进 `sealing → sealed`：生成卷末监测报告写入 `.novel/content/reports/<volume-id>.md`，
/// SQLite 记录报告路径引用，推进状态到 `sealed`（design D1/D8）。
///
/// 内部 helper：由 `archive_volume` 在归档流程内调用，不单独对外暴露为独立命令。
/// DB-open 由调用方 `archive_volume` 持有，故本函数仍收 `&mut Connection`（事务需要）。
fn advance_to_sealed(
    conn: &mut Connection,
    target_dir: &Path,
    volume_id: i64,
) -> Result<(), CliError> {
    use std::io::Write;

    let volume = volume_repository::get_volume_by_id(conn, volume_id)?;

    // 卷状态须为 sealing（或已 sealed，则跳过报告生成）。
    if volume.status != "sealing" && volume.status != "sealed" {
        return Err(CliError::volume_not_sealed(format!(
            "卷 {} 状态为 {}，仅 sealing/sealed 状态可推进到 sealed",
            volume_id, volume.status
        )));
    }

    if volume.status == "sealing" {
        // 生成卷末监测报告并写入正文文件（design D8：正文文件 + SQLite 路径引用）。
        let report_content = generate_monitoring_report(conn, volume_id)?;
        let novel_dir = target_dir.join(".novel");
        let reports_dir = novel_dir.join("content/reports");
        std::fs::create_dir_all(&reports_dir)
            .map_err(|e| CliError::storage(format!("创建报告目录失败：{} - {}", reports_dir.display(), e)))?;
        let report_path_rel = format!("content/reports/volume-{}.md", volume_id);
        let report_path_full = novel_dir.join(&report_path_rel);

        let mut file = std::fs::File::create(&report_path_full)
            .map_err(|e| CliError::storage(format!("创建监测报告文件失败：{} - {}", report_path_full.display(), e)))?;
        file.write_all(report_content.as_bytes())
            .map_err(|e| CliError::storage(format!("写入监测报告失败：{} - {}", report_path_full.display(), e)))?;

        // 记录报告路径引用（V008 新增列 monitoring_report_path）并推进到 sealed。
        let now = time::OffsetDateTime::now_utc();
        let timestamp = now
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

        let tx = conn
            .transaction()
            .map_err(|e| CliError::storage(format!("开启事务失败：{}", e)))?;
        tx.execute(
            "UPDATE volumes SET status = 'sealed', monitoring_report_path = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![report_path_rel, timestamp, volume_id],
        )
        .map_err(|e| CliError::storage(format!("更新卷状态到 sealed 失败：{}", e)))?;
        tx.commit()
            .map_err(|e| CliError::storage(format!("提交事务失败：{}", e)))?;
    }

    Ok(())
}

/// 生成卷末监测报告（人读 Markdown），含卷内章节终态、涌现处置摘要、卷目标达成摘要。
fn generate_monitoring_report(conn: &Connection, volume_id: i64) -> Result<String, CliError> {
    let volume = volume_repository::get_volume_by_id(conn, volume_id)?;
    let mut report = String::new();
    report.push_str(&format!("# 卷末监测报告：卷 {}（{}）\n\n", volume_id, volume.name));
    report.push_str(&format!("- 状态：{}\n", volume.status));
    if let Some(sealed_at) = &volume.sealed_at {
        report.push_str(&format!("- 封卷时间：{}\n", sealed_at));
    }
    report.push('\n');

    // 章节终态摘要
    report.push_str("## 章节终态\n\n");
    let chapters: Vec<(String, String, i64)> = conn
        .prepare("SELECT id, title, word_count FROM chapters WHERE volume_id = ?1 ORDER BY created_at ASC")
        .map_err(|e| CliError::storage(format!("查询章节摘要失败：{}", e)))?
        .query_map([volume_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| CliError::storage(format!("查询章节摘要失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析章节摘要失败：{}", e)))?;
    if chapters.is_empty() {
        report.push_str("- 无章节\n");
    } else {
        report.push_str("| 章节 | 标题 | 字数 |\n|---|---|---|\n");
        for (id, title, wc) in &chapters {
            report.push_str(&format!("| {} | {} | {} |\n", id, title, wc));
        }
    }
    report.push('\n');

    // 涌现处置摘要
    report.push_str("## 涌现处置摘要\n\n");
    let emergences: Vec<(String, String)> = conn
        .prepare("SELECT id, status FROM emergences WHERE volume_id = ?1 ORDER BY discovered_at ASC")
        .map_err(|e| CliError::storage(format!("查询涌现摘要失败：{}", e)))?
        .query_map([volume_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| CliError::storage(format!("查询涌现摘要失败：{}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::storage(format!("解析涌现摘要失败：{}", e)))?;
    if emergences.is_empty() {
        report.push_str("- 无涌现\n");
    } else {
        report.push_str("| 涌现 | 状态 |\n|---|---|\n");
        for (id, status) in &emergences {
            report.push_str(&format!("| {} | {} |\n", id, status));
        }
    }
    report.push('\n');

    // 卷目标达成摘要
    report.push_str("## 卷目标达成摘要\n\n");
    let goals = volume_repository::get_goals_by_volume(conn, volume_id)?;
    if goals.is_empty() {
        report.push_str("- 无卷目标\n");
    } else {
        report.push_str("| 目标 ID | 主线节点 | 必达 | 达成 |\n|---|---|---|---|\n");
        for goal in &goals {
            let achieved: i64 = conn
                .query_row(
                    "SELECT achieved FROM mainplot_nodes WHERE id = ?1",
                    [&goal.mainplot_node_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            report.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                goal.id,
                goal.mainplot_node_id,
                if goal.is_mandatory { "是" } else { "否" },
                if achieved == 1 { "是" } else { "否" }
            ));
        }
    }

    Ok(report)
}

/// 归档卷（design D1/D3）。
///
/// 接受 `sealing` 或 `sealed` 状态的卷：
/// - `sealing`：先生成卷末监测报告推进到 `sealed`，再推进到 `archived`，记录 `archived_at` 与冻结清单。
/// - `sealed`：直接推进到 `archived`。
/// 非 `sealing`/`sealed` 阻断（退出码 3，`volume_not_sealed`），已归档阻断（退出码 3）。
pub fn archive_volume(
    target: &Path,
    volume_id: i64,
) -> Result<(), CliError> {
    let mut conn = open_existing_at(target)?;

    // 检查卷是否存在与状态。
    let volume = volume_repository::get_volume_by_id(&conn, volume_id)?;

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

    // sealing 态：先生成监测报告并推进到 sealed。
    if volume.status == "sealing" {
        advance_to_sealed(&mut conn, target, volume_id)?;
    }

    // 推进到 archived，记录归档时间。
    let now = time::OffsetDateTime::now_utc();
    let timestamp = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| CliError::storage(format!("格式化时间戳失败：{}", e)))?;

    let tx = conn
        .transaction()
        .map_err(|e| CliError::storage(format!("开启事务失败：{}", e)))?;
    tx.execute(
        "UPDATE volumes SET status = 'archived', archived_at = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![timestamp, timestamp, volume_id],
    )
    .map_err(|e| CliError::storage(format!("更新卷状态到 archived 失败：{}", e)))?;
    tx.commit()
        .map_err(|e| CliError::storage(format!("提交事务失败：{}", e)))?;

    Ok(())
}

/// 设置卷目标。
pub fn set_volume_goal(
    target: &Path,
    volume_id: i64,
    mainplot_node_id: &str,
    description: Option<&str>,
    is_mandatory: bool,
) -> Result<i64, CliError> {
    let conn = open_existing_at(target)?;

    // 检查卷是否存在
    let volume = volume_repository::get_volume_by_id(&conn, volume_id)?;

    // 检查卷状态：archived 时拒绝
    if volume.status == "archived" {
        return Err(CliError::volume_already_archived(format!(
            "卷 {} 已归档，无法设置目标",
            volume_id
        )));
    }

    // 验证节点 ID 格式
    volume_validator::validate_mainplot_node_id(mainplot_node_id)?;

    // 校验主线节点存在（防止卷目标引用悬空节点；若缺失在封卷时才崩溃是更差的体验）。
    // 节点不存在返回 mainplot_node_not_found（退出码 43），与 verdict 的节点校验保持一致。
    let node_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM mainplot_nodes WHERE id = ?1)",
            [mainplot_node_id],
            |row| row.get(0),
        )
        .map_err(|e| CliError::storage(format!("检查主线节点是否存在失败：{}", e)))?;
    if !node_exists {
        return Err(CliError::mainplot_node_not_found(format!(
            "主线节点 '{}' 不存在，无法设置卷目标（请先用 `novel mainplot node create` 创建该节点）",
            mainplot_node_id
        )));
    }

    // 添加目标
    volume_repository::add_goal(&conn, volume_id, mainplot_node_id, description, is_mandatory)
}

/// 获取卷的所有目标。
pub fn get_volume_goals(target: &Path, volume_id: i64) -> Result<Vec<VolumeGoal>, CliError> {
    let conn = open_existing_at(target)?;

    // 检查卷是否存在
    let _ = volume_repository::get_volume_by_id(&conn, volume_id)?;

    // 查询目标
    volume_repository::get_goals_by_volume(&conn, volume_id)
}

/// 删除卷目标。
pub fn remove_volume_goal(target: &Path, volume_id: i64, goal_id: i64) -> Result<(), CliError> {
    let conn = open_existing_at(target)?;

    // 检查卷是否存在
    let volume = volume_repository::get_volume_by_id(&conn, volume_id)?;

    // 检查卷状态：sealed 或 archived 时拒绝
    if volume.status == "sealed" || volume.status == "archived" {
        return Err(CliError::volume_already_sealed(format!(
            "卷 {} 状态为 {}，无法删除目标",
            volume_id, volume.status
        )));
    }

    // 删除目标
    volume_repository::remove_goal(&conn, goal_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::run_migrations;
    use rusqlite::Connection;
    use tempfile::tempdir;

    /// 返回 `(conn, dir)`：在 `dir/.novel/novel.db` 建库并跑迁移，`conn` 供测试直接做状态修改；
    /// 公开 service 函数接收 `dir.path()`（`&Path`），经 `open_existing_at` 解析到同一 `.novel/novel.db`。
    fn setup() -> (Connection, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let novel_dir = dir.path().join(".novel");
        std::fs::create_dir_all(&novel_dir).unwrap();
        let db_path = novel_dir.join("novel.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn, "2026-06-30T00:00:00Z").unwrap();
        (conn, dir)
    }

    #[test]
    fn test_create_volume_validates_order() {
        let (_conn, dir) = setup();

        let result = create_volume(dir.path(), "第一卷", None, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_volume_validates_unique_name() {
        let (_conn, dir) = setup();

        create_volume(dir.path(), "第一卷", None, 1).unwrap();
        let result = create_volume(dir.path(), "第一卷", None, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_seal_volume_already_sealed() {
        let (conn, dir) = setup();

        let volume_id = create_volume(dir.path(), "第一卷", None, 1).unwrap();
        volume_repository::update_volume_status(&conn, volume_id, "sealed").unwrap();

        let result = seal_volume(dir.path(), volume_id, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_volume_not_sealed() {
        let (_conn, dir) = setup();

        let volume_id = create_volume(dir.path(), "第一卷", None, 1).unwrap();

        let result = archive_volume(dir.path(), volume_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_volume_goal_archived_volume() {
        let (conn, dir) = setup();

        let volume_id = create_volume(dir.path(), "第一卷", None, 1).unwrap();
        // 缺陷修复后 set_volume_goal 校验节点存在，需预置 node-001 以便真正触发 archived 卷门禁。
        conn.execute(
            "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) VALUES ('node-001',?1,1,'节点',0,'2026-06-30T00:00:00Z','2026-06-30T00:00:00Z')",
            [volume_id],
        ).unwrap();
        volume_repository::update_volume_status(&conn, volume_id, "archived").unwrap();

        let result = set_volume_goal(dir.path(), volume_id, "node-001", None, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_volume_goal_invalid_node_id() {
        let (_conn, dir) = setup();

        let volume_id = create_volume(dir.path(), "第一卷", None, 1).unwrap();

        let result = set_volume_goal(dir.path(), volume_id, "node@invalid", None, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_volume_goal_sealed_volume() {
        let (conn, dir) = setup();

        let volume_id = create_volume(dir.path(), "第一卷", None, 1).unwrap();
        // 缺陷修复后 set_volume_goal 校验节点存在，需预置 node-001。
        conn.execute(
            "INSERT INTO mainplot_nodes (id, volume_id, sequence_number, title, achieved, created_at, updated_at) VALUES ('node-001',?1,1,'节点',0,'2026-06-30T00:00:00Z','2026-06-30T00:00:00Z')",
            [volume_id],
        ).unwrap();
        let goal_id = set_volume_goal(dir.path(), volume_id, "node-001", None, true).unwrap();

        volume_repository::update_volume_status(&conn, volume_id, "sealed").unwrap();

        let result = remove_volume_goal(dir.path(), volume_id, goal_id);
        assert!(result.is_err());
    }
}

