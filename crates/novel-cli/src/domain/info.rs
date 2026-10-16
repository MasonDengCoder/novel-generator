// `novel info` 输出的领域对象。
//
// 这是 service 组装、renderer 序列化的稳定结构。字段集合对应
// contracts/cli/json-schemas/info.schema.json。

use serde::Serialize;
// contract-governance-03-struct-schema-gen：JsonSchema derive 让 Serialize struct 成为
// json-schemas 结构契约源，供 dev-only spec-gen 遍历派生 schema。
use schemars::JsonSchema;

/// `novel info --json` 与人读输出共用的世界概况对象。
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    /// 世界状态 schema 版本，单一数据源：SQLite PRAGMA user_version。
    pub schema_version: String,
    /// 项目/世界名称。
    pub project_name: String,
    /// 相对世界目录的数据库路径。
    pub db_path: String,
    /// 世界创建时间（ISO-8601）。
    pub created_at: String,
    /// 各实体计数，反映对应实体表的真实行数。
    pub counts: Counts,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub chapters: u64,
    pub volumes: u64,
    pub settings: u64,
    pub constraints: u64,
    pub mainplot_nodes: u64,
}
