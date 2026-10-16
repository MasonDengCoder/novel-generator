// info 渲染：`--json` 输出含 schemaVersion 的稳定 JSON；无 `--json` 输出人读键值。
//
// 约定（contracts/cli/json-output.md）：JSON 模式输出纯 JSON，不含 ANSI/旁白，可被解析器
// 直接解析；结构对应 contracts/cli/json-schemas/info.schema.json。

use crate::domain::info::WorldInfo;

pub fn render(info: &WorldInfo, json: bool) {
    if json {
        let s = serde_json::to_string_pretty(info).unwrap_or_else(|_| "{}".to_string());
        println!("{s}");
        return;
    }

    println!("📖 novel 世界概况");
    println!("  项目名称  : {}", info.project_name);
    println!("  数据库    : {}", info.db_path);
    println!("  创建时间  : {}", info.created_at);
    println!("  schema版本: {}", info.schema_version);
    println!("  计数:");
    println!("    chapters      : {}", info.counts.chapters);
    println!("    volumes       : {}", info.counts.volumes);
    println!("    settings      : {}", info.counts.settings);
    println!("    constraints   : {}", info.counts.constraints);
    println!("    mainplotNodes : {}", info.counts.mainplot_nodes);
}
