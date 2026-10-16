// domain 层：稳定实体类型、枚举与领域错误码基础集合。
//
// 职责（CLAUDE.md §6、CLI模块对齐清单 §6）：定义 World、SettingDocument、Chapter、Volume、
// Verdict、Emergent 等稳定实体类型与状态枚举；定义 schemaVersion、Context Priority、
// CompressionLevel、EmergentScoreDimension 等。不包含存储路径公开约定，不直接读写 storage。
//
// 本切片为占位：仅定义 init/info 用到的最小领域类型，其余实体随各自切片补入。

pub mod chapter_context;
pub mod constraint;
pub mod emergence;
pub mod info;
pub mod mainplot_node;
pub mod render_views;
pub mod setting;
pub mod thread;

pub use chapter_context::{
    Audit, BudgetAccount, BudgetFailure, ChapterContextView, CompressedRecord, ContextPacket,
    ContextPriority, Fragment, OmittedRecord, Overflow,
};
pub use constraint::{Constraint, ConstraintSeverity, ConstraintType};
pub use emergence::{Decision, Emergence, EmergenceStatus};
pub use info::WorldInfo;
pub use mainplot_node::MainplotNode;
pub use render_views::{ConstraintScope, ConstraintView, SettingView};
pub use setting::Setting;
pub use thread::{BindingType, Thread, ThreadBinding, ThreadStatus};
