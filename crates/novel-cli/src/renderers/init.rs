// init 渲染：成功时人读或 JSON 输出。冲突等错误由 renderers::error 处理。
//
// 约定（contracts/cli/json-output.md）：`--json` 模式输出纯 JSON，不含 ANSI/旁白；
// 非_json 输出人读键值。JSON 成功对象含 schemaVersion。

use serde::Serialize;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;

use crate::services::init::InitOutput;

/// `novel init --json` 成功输出对象。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitJsonOutput {
    pub schema_version: String,
    pub project_name: String,
    pub db_path: String,
    pub created_at: String,
    pub world_root: String,
}

pub fn render_success(out: &InitOutput, json: bool) {
    if json {
        let obj = InitJsonOutput {
            schema_version: out.schema_version.clone(),
            project_name: out.project_name.clone(),
            db_path: out.db_path.clone(),
            created_at: out.created_at.clone(),
            world_root: out.world_root.display().to_string(),
        };
        let s = serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".to_string());
        println!("{s}");
        return;
    }

    println!("✅ 已初始化 novel 世界");
    println!("  项目名称 : {}", out.project_name);
    println!("  世界根目录: {}", out.world_root.display());
    println!("  数据库   : {}", out.db_path);
    println!("  创建时间 : {}", out.created_at);
    println!("  schema版本: {}", out.schema_version);
    if out.repo_warning {
        eprintln!(
            "⚠️  检测到目标路径疑似 novel-generator 仓库根。novel-generator 不应保存具体小说项目的 \
             .novel/ 数据，建议在目标小说项目目录初始化。"
        );
    }
}
