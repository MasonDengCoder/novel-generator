// 集成测试公共辅助：用 assert_cmd 跑真实 CLI 子进程，初始化世界并创建卷。
// 被 s06 集成测试复用。

use assert_cmd::Command;
use tempfile::TempDir;

/// 初始化世界并创建一个卷，返回 (TempDir, volume_id)。
/// TempDir 需保持存活以维护目录，调用方须保留它。
pub fn setup_world_with_volume() -> (TempDir, i64) {
    let dir = tempfile::tempdir().unwrap();

    // novel init demo --path <dir> --json
    let mut cmd = Command::cargo_bin("novel").unwrap();
    cmd.args(["init", "demo", "--json", "--path"]).arg(dir.path());
    cmd.assert().success();

    // novel volume create 卷一 --path <dir>
    // volume create 不支持 --json，从人读输出解析 "ID = N"。
    let mut cmd = Command::cargo_bin("novel").unwrap();
    cmd.args(["volume", "create", "卷一", "--path"]).arg(dir.path());
    let output = cmd.assert().success().get_output().clone();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let volume_id = parse_volume_id(&stdout);

    (dir, volume_id)
}

/// 初始化世界、创建卷与一个章节，返回 (TempDir, volume_id, chapter_id)。
pub fn setup_world_with_volume_and_chapter() -> (TempDir, i64, String) {
    let (dir, volume_id) = setup_world_with_volume();

    // novel chapter create 第一章 --json
    // chapter create 不支持 --path，使用 current_dir 指向临时目录。
    let mut cmd = Command::cargo_bin("novel").unwrap();
    cmd.args(["chapter", "create", "第一章", "--json"]).current_dir(dir.path());
    let output = cmd.assert().success().get_output().clone();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let chapter_id = json["id"].as_str().unwrap().to_string();

    (dir, volume_id, chapter_id)
}

/// 构造一个 novel Command，已设置 current_dir 指向世界目录。
pub fn novel_at(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("novel").unwrap();
    cmd.current_dir(dir.path());
    cmd
}

/// 从 volume create 的人读输出中解析 volume id。
/// 输出形如 "✓ 创建卷成功：ID = 1"。
fn parse_volume_id(stdout: &str) -> i64 {
    let line = stdout
        .lines()
        .find(|l| l.contains("ID ="))
        .expect("volume create 输出应包含 ID =");
    let id_str = line
        .split("ID =")
        .nth(1)
        .expect("应能按 'ID =' 分割")
        .trim();
    id_str.parse::<i64>().expect("volume id 应为整数")
}
