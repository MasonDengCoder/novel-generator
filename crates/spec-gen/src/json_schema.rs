//! JSON Schema 类型化生成源（contract-governance-03-struct-schema-gen）。
//!
//! 把 `novel-cli` 的 `Serialize` struct（带 `JsonSchema` derive）提升为 `contracts/cli/json-schemas/*.json`
//! 的结构契约源，对账手写版。与 `enum_source`（错误码/退出码从 enum 生成）对称：struct 生成 schema。
//!
//! 生成链路：
//!   novel_cli 的 JsonSchema struct  ← 结构契约唯一类型化源
//!     → SchemaGenerator::schema_for::<T>() 导出 Schema（draft07 settings，$schema 默认即 draft-07）
//!     → inject additionalProperties:false（对齐手写版封闭对象语义）
//!     → 写盘 contracts/cli/json-schemas/<name>.schema.json
//!
//! 映射表用函数指针：Rust 无反射，struct 路径字符串不能动态分发，每个 struct 经显式
//! `fn gen_xxx(g) = g.schema_for::<Xxx>()` 注册。
//!
//! 边界（CLAUDE.md §3/§8）：spec-gen 是 dev-only xtask，以 lib 形式消费 novel-cli 的 struct 源，
//! 不读 SQLite、不读 `.novel/content`，不绕过 CLI。本模块只读 struct 的 JsonSchema 派生结果，
//! 属编译期类型化数据，无运行期世界状态。

use schemars::gen::SchemaGenerator;
use schemars::schema::{RootSchema, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;
use serde_json::Value;
use std::path::Path;

use novel_cli::domain::info::WorldInfo;
use novel_cli::renderers::constraint_renderer::{
    CheckResultListJson, CheckSuccessJson, ConstraintListJson, ConstraintViewJson, ConstraintJson,
};
use novel_cli::renderers::emergence_renderer::{EmergenceJson, EmergenceListJson};
use novel_cli::renderers::export_renderer::{ExportChapterJson, ExportFullJson, ExportVolumeJson};
use novel_cli::renderers::mainplot_renderer::{MainplotNodeJson, MainplotNodeListJson};
use novel_cli::renderers::setting_renderer::{SettingJson, SettingListJson, SettingViewJson};
use novel_cli::renderers::thread_renderer::{ThreadBindingListJson, ThreadJson, ThreadListJson};
use novel_cli::renderers::verify_renderer::VerifyReportJson;
use novel_cli::renderers::volume_renderer::{SealCheckResultsJson, VolumeGoalsJson, VolumeInfoJson, VolumeListJson};

/// 一条 struct → schema 映射：schema 文件名（不含 `.schema.json` 后缀）+ struct 全路径标注 + 生成函数。
pub struct Mapping {
    pub name: &'static str,
    pub struct_path: &'static str,
    pub gen: fn(&mut SchemaGenerator) -> RootSchema,
}

// 显式 gen 函数：每个 struct 一个，避免 const 闭包 + 泛型限制。
// 用 `root_schema_for::<T>()` 返回 RootSchema（含 $schema=meta_schema，draft07 settings 默认即 draft-07）。
macro_rules! gen_fn {
    ($ty:path) => {
        |g: &mut SchemaGenerator| g.root_schema_for::<$ty>()
    };
}

/// 全部生成化 schema 的映射表。
///
/// `chapter`/`verdict`/`chapter-context` 三类 renderer 用 `json!` 宏内联构造、无带 Serialize 的
/// struct，保留手写，不在本表（design Decision 6）。`Counts` 等嵌套类型自动进父 struct 的
/// definitions，不单独映射。
pub static MAPPINGS: &[Mapping] = &[
    // setting（renderers/setting_renderer.rs）
    Mapping { name: "setting-get", struct_path: "novel_cli::renderers::setting_renderer::SettingJson", gen: gen_fn!(SettingJson) },
    Mapping { name: "setting-list", struct_path: "novel_cli::renderers::setting_renderer::SettingListJson", gen: gen_fn!(SettingListJson) },
    Mapping { name: "setting-render", struct_path: "novel_cli::renderers::setting_renderer::SettingViewJson", gen: gen_fn!(SettingViewJson) },
    // constraint（renderers/constraint_renderer.rs）
    Mapping { name: "constraint-render", struct_path: "novel_cli::renderers::constraint_renderer::ConstraintJson", gen: gen_fn!(ConstraintJson) },
    Mapping { name: "constraint-list", struct_path: "novel_cli::renderers::constraint_renderer::ConstraintListJson", gen: gen_fn!(ConstraintListJson) },
    Mapping { name: "constraint-render-view", struct_path: "novel_cli::renderers::constraint_renderer::ConstraintViewJson", gen: gen_fn!(ConstraintViewJson) },
    Mapping { name: "constraint-check-write", struct_path: "novel_cli::renderers::constraint_renderer::CheckSuccessJson", gen: gen_fn!(CheckSuccessJson) },
    Mapping { name: "constraint-check-list", struct_path: "novel_cli::renderers::constraint_renderer::CheckResultListJson", gen: gen_fn!(CheckResultListJson) },
    // thread（renderers/thread_renderer.rs）
    Mapping { name: "thread", struct_path: "novel_cli::renderers::thread_renderer::ThreadJson", gen: gen_fn!(ThreadJson) },
    Mapping { name: "thread-list", struct_path: "novel_cli::renderers::thread_renderer::ThreadListJson", gen: gen_fn!(ThreadListJson) },
    Mapping { name: "thread-binding-list", struct_path: "novel_cli::renderers::thread_renderer::ThreadBindingListJson", gen: gen_fn!(ThreadBindingListJson) },
    // mainplot（renderers/mainplot_renderer.rs）
    Mapping { name: "mainplot-node", struct_path: "novel_cli::renderers::mainplot_renderer::MainplotNodeJson", gen: gen_fn!(MainplotNodeJson) },
    Mapping { name: "mainplot-node-list", struct_path: "novel_cli::renderers::mainplot_renderer::MainplotNodeListJson", gen: gen_fn!(MainplotNodeListJson) },
    // emergence（renderers/emergence_renderer.rs）
    Mapping { name: "emergence", struct_path: "novel_cli::renderers::emergence_renderer::EmergenceJson", gen: gen_fn!(EmergenceJson) },
    Mapping { name: "emergence-list", struct_path: "novel_cli::renderers::emergence_renderer::EmergenceListJson", gen: gen_fn!(EmergenceListJson) },
    // volume（renderers/volume_renderer.rs）
    Mapping { name: "volume-info", struct_path: "novel_cli::renderers::volume_renderer::VolumeInfoJson", gen: gen_fn!(VolumeInfoJson) },
    Mapping { name: "volume-list", struct_path: "novel_cli::renderers::volume_renderer::VolumeListJson", gen: gen_fn!(VolumeListJson) },
    Mapping { name: "volume-goals", struct_path: "novel_cli::renderers::volume_renderer::VolumeGoalsJson", gen: gen_fn!(VolumeGoalsJson) },
    Mapping { name: "volume-seal-check", struct_path: "novel_cli::renderers::volume_renderer::SealCheckResultsJson", gen: gen_fn!(SealCheckResultsJson) },
    // export（renderers/export_renderer.rs）
    Mapping { name: "export-volume", struct_path: "novel_cli::renderers::export_renderer::ExportVolumeJson", gen: gen_fn!(ExportVolumeJson) },
    Mapping { name: "export-chapter", struct_path: "novel_cli::renderers::export_renderer::ExportChapterJson", gen: gen_fn!(ExportChapterJson) },
    Mapping { name: "export-full", struct_path: "novel_cli::renderers::export_renderer::ExportFullJson", gen: gen_fn!(ExportFullJson) },
    // verify（renderers/verify_renderer.rs）
    Mapping { name: "verify-integrity", struct_path: "novel_cli::renderers::verify_renderer::VerifyReportJson", gen: gen_fn!(VerifyReportJson) },
    // info（domain/info.rs）
    Mapping { name: "info", struct_path: "novel_cli::domain::info::WorldInfo", gen: gen_fn!(WorldInfo) },
];

/// 生成一份 schema 的 pretty JSON 字符串：导出 RootSchema → 注入 additionalProperties:false。
///
/// `$schema=draft-07` 由 `SchemaGenerator::default()`（draft07 settings）的 `meta_schema` 提供，
/// `root_schema_for` 把它写入 RootSchema.meta_schema，序列化为 `$schema` 字段。
pub fn render_schema_string(mapping: &Mapping) -> String {
    let mut generator = SchemaGenerator::default();
    let mut root = (mapping.gen)(&mut generator);
    inject_additional_properties_false(&mut root.schema);
    // definitions 内的嵌套 struct 同样注入（schemars 把嵌套类型放 definitions 用 $ref 引用，
    // 注入需覆盖此路径，否则被引用的 struct 缺 additionalProperties:false）。
    for (_, def) in root.definitions.iter_mut() {
        let mut def_obj = def.clone().into_object();
        inject_additional_properties_false(&mut def_obj);
        *def = def_obj.into();
    }
    let value = serde_json::to_value(&root).unwrap_or(Value::Null);
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

/// 递归为所有 object 节点注入 `additionalProperties: false`。
///
/// schemars 默认省略该字段（等价 true），手写版为 false。后处理统一注入，避免逐 struct 标注。
fn inject_additional_properties_false(obj: &mut SchemaObject) {
    if let Some(object) = &mut obj.object {
        object.additional_properties = Some(Box::new(Schema::Bool(false)));
        for (_, sub) in object.properties.iter_mut() {
            let mut sub_obj = sub.clone().into_object();
            inject_additional_properties_false(&mut sub_obj);
            *sub = sub_obj.into();
        }
    }
    if let Some(arr) = &mut obj.array {
        if let Some(items) = &mut arr.items {
            match items {
                SingleOrVec::Single(schema) => {
                    let mut items_obj = schema.clone().into_object();
                    inject_additional_properties_false(&mut items_obj);
                    *schema = Box::new(items_obj.into());
                }
                SingleOrVec::Vec(schemas) => {
                    for schema in schemas.iter_mut() {
                        let mut items_obj = schema.clone().into_object();
                        inject_additional_properties_false(&mut items_obj);
                        *schema = items_obj.into();
                    }
                }
            }
        }
    }
}

/// 全部映射清单（供 generate/check 遍历）。
pub fn mappings() -> &'static [Mapping] {
    MAPPINGS
}

/// 对账报告：stdout 输出已生成的 schema 文件名清单与 struct 路径，供人比对。
pub fn report() -> String {
    let mut s = String::new();
    s.push_str(&format!("# json-schema 对账报告（{} 份生成化 schema）\n\n", MAPPINGS.len()));
    s.push_str("| schema 文件名 | struct 源 |\n");
    s.push_str("|---|---|\n");
    for m in MAPPINGS {
        s.push_str(&format!("| {} | `{}` |\n", m.name, m.struct_path));
    }
    s.push_str("\n保留手写（renderer 用 json! 宏无 struct）：chapter / verdict / chapter-context\n");
    s
}

/// drift 检查：内存重新生成 24 份生成化 schema，与仓库 `contracts/cli/json-schemas/` 比对。
///
/// `chapter`/`verdict`/`chapter-context` 三类手写 schema 不在 MAPPINGS，天然豁免（不参与 diff）。
/// 任一生成物与仓库文件 diff 非空即漂移（exit 1）。返回漂移条目清单。
pub fn check_drift(schemas_dir: &Path) -> Vec<String> {
    let mut drifts: Vec<String> = Vec::new();
    for mapping in MAPPINGS {
        let expected = render_schema_string(mapping);
        let path = schemas_dir.join(format!("{}.schema.json", mapping.name));
        let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
        if expected != on_disk {
            drifts.push(format!("{}（{}）", path.display(), mapping.name));
        }
    }
    drifts
}

