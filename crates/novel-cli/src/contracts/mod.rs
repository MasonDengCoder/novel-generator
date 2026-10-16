//! 命令契约注册表（contract-registry-v2-03-command-contract-registry）。
//!
//! 为全部 CLI 叶子命令建立唯一结构化 `CommandContract` 注册表，作为后续契约生成器
//! （slice 04 full-contract-generation）的命令语义唯一输入。
//!
//! 边界（CLAUDE.md §6）：本模块属纯声明 + 校验，不访问 storage / repository / services，
//! 不读 SQLite / `.novel/content`。只消费两个既有类型化源：
//! - clap 命令树（`crate::commands::build_cli()`）——语法唯一来源；
//! - `ErrorCatalog`（`crate::errors::catalog`）——错误 ID 唯一来源。
//!
//! 设计决策（见 openspec/changes/contract-registry-v2-03-command-contract-registry/design.md）：
//! - D1 静态 Rust 数据 + 顶层注册函数 `register_contracts()`，编译期可被构建期校验遍历。
//! - D2 记录含 command_path / summary / inputs / outputs / errors / guard。
//! - D3 交叉一致性校验核心为单一函数 `validate_registry()`，被单元测试与 spec-gen 复用。
//! - D4 错误集合为显式声明 + 目录合法校验锚定（运行时完备断言归 05 治理）。
//! - D5 输出定义引用 schema 名而非复制 JSON 结构。

use crate::errors::ErrorCode;

/// 机器可验证的输入语义槽位（非复制语法——语法由 clap 裁决）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputContract {
    /// 语义槽位名（如 `target-status`、`chapter-id`）。
    pub slot: &'static str,
    /// 语义约束描述（如「目标状态 ∈ {outline,drafting,review,approved}」）。
    pub constraint: &'static str,
}

/// 机器可验证的输出形态定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputContract {
    /// 引用 `contracts/cli/json-schemas/<schema>.json` 的稳定 JSON 输出形态。
    JsonSchema(&'static str),
    /// 人读输出（无稳定 JSON schema）。
    HumanReadable,
    /// 静默成功（无输出）。
    SilentSuccess,
}

/// 状态机或 guard 引用（稳定引用，不依赖 CLI 内部实现模块名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardRef {
    /// 引用目标（如 `state-machine.md#chapter-status` 或守卫名）。
    pub target: &'static str,
}

/// 单条命令的结构化契约记录。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandContract {
    /// 完整命令路径（如 `volume goal set`，不含 `novel`），与 clap 叶子命令名一一对应。
    pub command_path: &'static str,
    /// 面向契约消费者的行为摘要（人读，中文，与 `commands.md` 行为语义段概要一致）。
    pub summary: &'static str,
    /// 机器可验证输入定义（语义槽位）。
    pub inputs: &'static [InputContract],
    /// 机器可验证输出形态。
    pub outputs: &'static [OutputContract],
    /// 可返回的 ErrorCatalog 错误 ID 集合（引用 `ErrorCode` 判别式）。
    pub errors: &'static [ErrorCode],
    /// 适用的状态机或 guard 引用。
    pub guard: &'static [GuardRef],
}

/// 顶层注册表：全部 46 个 clap 叶子命令各一条记录。
///
/// TODO(05 治理)：逐命令运行时错误集完备性审计归 contract-v2-rollout-governance 强化。
pub const REGISTRY: &'static [CommandContract] = &[
    // ── init ────────────────────────────────────────────────
    CommandContract {
        command_path: "init",
        summary: "创建新世界：在目标目录下创建 .novel/ 目录结构与 SQLite 数据库，写入项目级元数据与 schemaVersion。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("info")],
        errors: &[ErrorCode::Usage, ErrorCode::ResourceConflict, ErrorCode::Storage],
        guard: &[],
    },
    // ── info ────────────────────────────────────────────────
    CommandContract {
        command_path: "info",
        summary: "输出当前世界基本信息与数据概况。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("info")],
        errors: &[ErrorCode::Usage, ErrorCode::WorldNotInitialized, ErrorCode::Storage],
        guard: &[],
    },
    // ── setting ─────────────────────────────────────────────
    CommandContract {
        command_path: "setting set",
        summary: "设置项目配置项：在 SQLite settings 表中插入或更新键值对配置（UPSERT 语义）。",
        inputs: &[],
        outputs: &[OutputContract::SilentSuccess],
        errors: &[ErrorCode::Usage, ErrorCode::WorldNotInitialized, ErrorCode::Storage],
        guard: &[],
    },
    CommandContract {
        command_path: "setting get",
        summary: "查询指定配置项的值。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("setting-get")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::SettingNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "setting list",
        summary: "列出所有已定义的配置项。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("setting-list")],
        errors: &[ErrorCode::WorldNotInitialized, ErrorCode::Storage],
        guard: &[],
    },
    CommandContract {
        command_path: "setting render",
        summary: "按受控 view 渲染设定注入视图。只读操作，受控 --view 名引用 views-scopes.md §A 组 1 冻结枚举。",
        inputs: &[InputContract {
            slot: "view",
            constraint: "受控视图名 ∈ {intro,four-poles,world,style,summary,full,llm}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("setting-render")],
        errors: &[ErrorCode::Usage, ErrorCode::WorldNotInitialized, ErrorCode::Storage],
        guard: &[],
    },
    // ── constraint ──────────────────────────────────────────
    CommandContract {
        command_path: "constraint define",
        summary: "定义创作约束规则：在 SQLite constraints 表中插入新约束。",
        inputs: &[InputContract {
            slot: "severity",
            constraint: "严重等级 ∈ {info,warning,hard}",
        }],
        outputs: &[OutputContract::SilentSuccess],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ConstraintAlreadyExists,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "constraint render",
        summary: "查询约束规则。支持单条精确查询与受控批量两种参数互斥形态，均只读。",
        inputs: &[InputContract {
            slot: "view/scope",
            constraint: "受控形态 view ∈ {llm,human}、scope ∈ {world,volume,chapter}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("constraint-render")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ConstraintNotFound,
            ErrorCode::VolumeNotFound,
            ErrorCode::ChapterNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "constraint list",
        summary: "列出所有或指定 scope/severity 的约束规则。",
        inputs: &[InputContract {
            slot: "severity",
            constraint: "严重等级 ∈ {info,warning,hard}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("constraint-list")],
        errors: &[ErrorCode::Usage, ErrorCode::WorldNotInitialized, ErrorCode::Storage],
        guard: &[],
    },
    CommandContract {
        command_path: "constraint check",
        summary: "规则检查结果管理：批量写入检查结果或查询指定章节的检查结果（含 stale 检测）。",
        inputs: &[InputContract {
            slot: "severity",
            constraint: "检查结果 severity ∈ {info,warning,hard}",
        }],
        outputs: &[
            OutputContract::SilentSuccess,
            OutputContract::JsonSchema("constraint-check-write"),
            OutputContract::JsonSchema("constraint-check-list"),
        ],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::ContentHashMismatch,
            ErrorCode::StaleRuleChecks,
            ErrorCode::InvalidInput,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── chapter ─────────────────────────────────────────────
    CommandContract {
        command_path: "chapter create",
        summary: "创建新章节：在 SQLite chapters 表中插入记录，生成唯一 chapter-id。可选 --volume 将章节归属到指定卷。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("chapter")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::VolumeAlreadySealed,
            ErrorCode::VolumeAlreadyArchived,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "volume-freeze-guard" }],
    },
    CommandContract {
        command_path: "chapter update",
        summary: "写入章节正文：读取输入文件，计算 hash 和字数，原子写入正文文件并更新 SQLite 元数据。",
        inputs: &[],
        outputs: &[
            OutputContract::SilentSuccess,
            OutputContract::JsonSchema("chapter"),
        ],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::FileNotFound,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "chapter-approval-freeze" }],
    },
    CommandContract {
        command_path: "chapter get",
        summary: "读取章节元数据和正文内容，重新计算正文 hash 与存储值对比检测外部修改。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("chapter")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::FileNotFound,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "chapter list",
        summary: "列出所有章节的元数据摘要。--verify-hash 时逐个读取正文文件并计算 hash 检测外部修改。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("chapter")],
        errors: &[
            ErrorCode::WorldNotInitialized,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "chapter reimport",
        summary: "重新导入外部修改的正文：读取当前正文文件，重新计算 hash 和 word_count，更新 SQLite 元数据。",
        inputs: &[],
        outputs: &[OutputContract::SilentSuccess],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::FileNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "chapter status",
        summary: "推进章节状态至目标状态，在状态机不允许该转换时硬阻断。",
        inputs: &[InputContract {
            slot: "target-status",
            constraint: "目标状态 ∈ {outline,drafting,review,approved}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("chapter")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::PreconditionBlocked,
            ErrorCode::InvalidStateTransition,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#chapter-status" }],
    },
    CommandContract {
        command_path: "chapter context",
        summary: "渲染 CLI canonical Chapter Context Packet：聚合 P0/P1 上下文片段，按 --budget 装填压缩省略。只读，不调用 LLM。",
        inputs: &[InputContract {
            slot: "view",
            constraint: "受控视图名 ∈ {writer,verifier,repair}（views-scopes.md §A 组 4）",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("chapter-context")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::ContextBudgetExceeded,
            ErrorCode::FileNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── verdict ─────────────────────────────────────────────
    CommandContract {
        command_path: "verdict set-mainplot-landing",
        summary: "记录章节对主线节点的落地裁决，绑定写入时刻的正文 hash。",
        inputs: &[InputContract {
            slot: "achieved",
            constraint: "取值 ∈ {0,1}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("verdict")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::PreconditionBlocked,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "chapter-approval-freeze" }],
    },
    // ── volume ──────────────────────────────────────────────
    CommandContract {
        command_path: "volume create",
        summary: "创建新卷：在 SQLite volumes 表中插入新记录，状态为 active。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ResourceConflict,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "volume info",
        summary: "查询单个卷的详细信息。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("volume-info")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "volume list",
        summary: "列举所有卷，按 volume_order 升序排序。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("volume-list")],
        errors: &[ErrorCode::WorldNotInitialized, ErrorCode::Storage],
        guard: &[],
    },
    CommandContract {
        command_path: "volume seal",
        summary: "执行封卷前置条件检查，通过后将卷状态推进到 sealing。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("volume-seal-check")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::ChaptersNotApproved,
            ErrorCode::VolumeAlreadySealed,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#volume-seal" }],
    },
    CommandContract {
        command_path: "volume archive",
        summary: "将 sealing 或 sealed 状态的卷归档到 archived。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::VolumeNotSealed,
            ErrorCode::VolumeAlreadyArchived,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#volume-archive" }],
    },
    CommandContract {
        command_path: "volume goal set",
        summary: "为卷绑定主线节点目标。默认创建必达目标，--optional 标记为可选。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::VolumeAlreadyArchived,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "volume goal get",
        summary: "查询卷的所有目标。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("volume-goals")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "volume goal remove",
        summary: "删除卷目标。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::VolumeAlreadySealed,
            ErrorCode::VolumeAlreadyArchived,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── mainplot ────────────────────────────────────────────
    CommandContract {
        command_path: "mainplot node create",
        summary: "创建主线节点：在指定卷内创建一个主线情节锚点。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("mainplot-node")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::ChapterNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "mainplot node list",
        summary: "列出卷内所有主线节点。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("mainplot-node-list")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "mainplot node get",
        summary: "获取单个主线节点详情。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("mainplot-node")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::MainplotNodeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "mainplot node update",
        summary: "更新主线节点的达成状态。",
        inputs: &[InputContract {
            slot: "achieved",
            constraint: "取值 ∈ {0,1}",
        }],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::MainplotNodeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── thread ──────────────────────────────────────────────
    CommandContract {
        command_path: "thread create",
        summary: "创建线索：在指定卷内创建一个跨章节情节线。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("thread")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::ThreadNameConflict,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "thread bind",
        summary: "绑定线索到章节：建立线索与章节的关联关系。",
        inputs: &[InputContract {
            slot: "type",
            constraint: "绑定类型 ∈ {mention,develop,resolve}（默认 mention）",
        }],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ThreadNotFound,
            ErrorCode::ChapterNotFound,
            ErrorCode::BindingAlreadyExists,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "thread list",
        summary: "列出卷内所有线索。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("thread-list")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "thread query",
        summary: "查询章节的所有线索绑定。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("thread-binding-list")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "thread update",
        summary: "更新线索状态。",
        inputs: &[InputContract {
            slot: "status",
            constraint: "线索状态 ∈ {active,paused,resolved}",
        }],
        outputs: &[OutputContract::HumanReadable],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ThreadNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── emergence ───────────────────────────────────────────
    CommandContract {
        command_path: "emergence create",
        summary: "创建涌现：记录卷内出现的非规划内容机会。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::ChapterNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "emergence rate",
        summary: "评分涌现：为 pending 状态的涌现打分并推进到 rated。",
        inputs: &[InputContract {
            slot: "rating",
            constraint: "评分 ∈ {0..=10}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::EmergenceNotFound,
            ErrorCode::InvalidStateTransition,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#emergence-state" }],
    },
    CommandContract {
        command_path: "emergence decide",
        summary: "决策涌现：对已评分的涌现做出接受/延期/拒绝决策并推进到 decided。",
        inputs: &[InputContract {
            slot: "decision",
            constraint: "决策 ∈ {accept,defer,reject}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::EmergenceNotFound,
            ErrorCode::InvalidStateTransition,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#emergence-state" }],
    },
    CommandContract {
        command_path: "emergence execute",
        summary: "执行涌现：将已决策为 accept 的涌现标记为已执行并推进到 executed。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::EmergenceNotFound,
            ErrorCode::ChapterNotFound,
            ErrorCode::InvalidStateTransition,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#emergence-state" }],
    },
    CommandContract {
        command_path: "emergence archive",
        summary: "归档涌现：将已执行的涌现标记为已归档，冻结其记录。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::EmergenceNotFound,
            ErrorCode::InvalidStateTransition,
            ErrorCode::EmergenceArchived,
            ErrorCode::Storage,
        ],
        guard: &[GuardRef { target: "state-machine.md#emergence-state" }],
    },
    CommandContract {
        command_path: "emergence list",
        summary: "列出卷内所有涌现。",
        inputs: &[InputContract {
            slot: "status",
            constraint: "状态过滤 ∈ {pending,rated,decided,executed,archived}",
        }],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence-list")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "emergence get",
        summary: "获取单个涌现详情。",
        inputs: &[],
        outputs: &[OutputContract::HumanReadable, OutputContract::JsonSchema("emergence")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::EmergenceNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── export ──────────────────────────────────────────────
    CommandContract {
        command_path: "export volume",
        summary: "导出单个卷的世界状态快照：组合卷信息与卷内下属实体，正文以字段嵌入，面向机器消费。",
        inputs: &[],
        outputs: &[OutputContract::JsonSchema("export-volume")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::VolumeNotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "export chapter",
        summary: "导出单个章节的快照：章节元数据 + 正文 + 裁决 + 规则检查结果。",
        inputs: &[],
        outputs: &[OutputContract::JsonSchema("export-chapter")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ChapterNotFound,
            ErrorCode::FileNotFound,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    CommandContract {
        command_path: "export full",
        summary: "导出整个世界的全量快照：遍历所有卷及其下属实体，逐实体校验 hash，输出单一 JSON 对象。",
        inputs: &[],
        outputs: &[OutputContract::JsonSchema("export-full")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
    // ── verify ──────────────────────────────────────────────
    CommandContract {
        command_path: "verify integrity",
        summary: "对世界状态做完整性验证：正文 hash 校验、引用完整性、状态一致性，输出可机读分层报告，hard 违规非零退出。",
        inputs: &[],
        outputs: &[OutputContract::JsonSchema("verify-integrity")],
        errors: &[
            ErrorCode::Usage,
            ErrorCode::WorldNotInitialized,
            ErrorCode::ContentModifiedExternally,
            ErrorCode::FileNotFound,
            ErrorCode::StateConflict,
            ErrorCode::NotFound,
            ErrorCode::Storage,
        ],
        guard: &[],
    },
];

/// 注册表访问入口：返回全部命令契约记录的静态切片。
pub const fn register_contracts() -> &'static [CommandContract] {
    REGISTRY
}

/// 显式声明「不关联特定命令」的目录公开错误 ID（spec「目录公开错误可被追踪到命令」的
/// 反向追踪豁免）。这些 ID 属目录登记的协议/前置条件类错误，当前无命令注册记录引用，
/// 由契约维护者显式标注以免反向校验误报：
/// - `external_modification`：V1 遗留通用完整性码，命令层实际经 `state_conflict`（Integrity）
///   表达，保留目录登记供协议信封兼容。
/// - `invalid_content`：内容校验类协议错误，当前无命令直接返回。
/// - `emergences_not_handled` / `goals_not_achieved`：卷封卷前置条件错误，尚无命令接线
///   （对应 `volume seal` 的 run 前置尚未实现），保持目录登记。
pub const EXPLICIT_UNASSOCIATED_ERRORS: &[ErrorCode] = &[
    ErrorCode::ExternalModification,
    ErrorCode::InvalidContent,
    ErrorCode::EmergencesNotHandled,
    ErrorCode::GoalsNotAchieved,
];

/// 单一校验核心：检查 clap 叶子命令、注册表与 ErrorCatalog 的交叉一致性。
///
/// 返回不一致报告（每条为可打印的说明）；空 Vec 表示一致。被 `crates/novel-cli` 内
/// `#[cfg(test)]` 单元测试与 dev-only 生成器 `spec-gen` 复用（design D3），MUST NOT
/// 存在第二份独立实现。
pub fn validate_registry() -> Vec<String> {
    let mut issues = Vec::new();

    // 1. 叶子命令全覆盖：clap 叶子路径集 vs 注册表 command_path 集一一对应。
    let leaf_paths = collect_clap_leaf_paths();
    let mut registered: Vec<&str> = REGISTRY.iter().map(|c| c.command_path).collect();
    registered.sort_unstable();
    let mut leaves = leaf_paths.clone();
    leaves.sort_unstable();

    // 注册表重复登记检查。
    let mut seen: Vec<&str> = Vec::new();
    for c in REGISTRY {
        if seen.contains(&c.command_path) {
            issues.push(format!("重复登记命令: {}", c.command_path));
        }
        seen.push(c.command_path);
    }

    // 遗漏/多余命令（差集双向）。
    for leaf in &leaves {
        if !registered.iter().any(|r| *r == leaf) {
            issues.push(format!("clap 叶子命令未登记: {}", leaf));
        }
    }
    for path in &registered {
        if !leaves.iter().any(|l| l == path) {
            issues.push(format!("注册表含非 clap 叶子命令（组命令或不存在）: {}", path));
        }
    }

    // 2. 错误 ID 合法：每条记录 errors 引用的 ErrorCode 均须在 ErrorCatalog 登记。
    for c in REGISTRY {
        for e in c.errors {
            let id = e.as_str();
            if crate::errors::ErrorCatalog::get(id).is_none() {
                issues.push(format!(
                    "命令 {} 引用了目录外错误 ID: {}",
                    c.command_path, id
                ));
            }
        }
    }

    // 3. guard 引用可解析：引用目标必须非空且以稳定引用形态表达。
    for c in REGISTRY {
        for g in c.guard {
            if g.target.trim().is_empty() {
                issues.push(format!("命令 {} 的 guard 引用为空", c.command_path));
            }
        }
    }

    // 4. 反向追踪：目录公开错误 ID 须被至少一条命令引用，或显式声明不关联特定命令。
    let mut referenced: Vec<&str> = Vec::new();
    for c in REGISTRY {
        for e in c.errors {
            referenced.push(e.as_str());
        }
    }
    for declared in EXPLICIT_UNASSOCIATED_ERRORS {
        referenced.push(declared.as_str());
    }
    for entry in crate::errors::ErrorCatalog::ENTRIES {
        if !referenced.contains(&entry.id) {
            issues.push(format!(
                "目录公开错误 ID 未被命令引用且未显式声明不关联命令: {}",
                entry.id
            ));
        }
    }

    issues
}

/// 遍历 clap 命令树收集全部叶子命令路径（与 spec-gen `commands_source::collect_commands()`
/// 同构；此处为校验核心内的复用实现，保持只读命令树）。
fn collect_clap_leaf_paths() -> Vec<String> {
    let root = crate::commands::build_cli();
    let mut out = Vec::new();
    walk_leaves(&root, "", &mut out);
    out
}

fn walk_leaves(cmd: &clap::Command, prefix: &str, out: &mut Vec<String>) {
    for sub in cmd.get_subcommands() {
        let name = if prefix.is_empty() {
            sub.get_name().to_string()
        } else {
            format!("{prefix} {}", sub.get_name())
        };
        if sub.get_subcommands().next().is_some() {
            walk_leaves(sub, &name, out);
        } else {
            out.push(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 注册表覆盖全部 clap 叶子命令、错误 ID 全部合法、guard 可解析，且无重复/组命令记录。
    #[test]
    fn registry_passes_cross_consistency_validation() {
        let issues = validate_registry();
        assert!(
            issues.is_empty(),
            "注册表交叉一致性校验失败:\n{}",
            issues.join("\n")
        );
    }

    /// 注册表记录数等于 clap 叶子命令数（46）。
    #[test]
    fn registry_matches_leaf_command_count() {
        let leaf_count = collect_clap_leaf_paths().len();
        assert_eq!(REGISTRY.len(), leaf_count, "注册表记录数必须等于 clap 叶子命令数");
    }

    /// 注册表只含叶子命令路径，无组命令记录。
    #[test]
    fn registry_contains_no_group_command_paths() {
        let leaves = collect_clap_leaf_paths();
        for c in REGISTRY {
            assert!(
                leaves.contains(&c.command_path.to_string()),
                "注册表记录 {} 不是 clap 叶子命令（组命令不登记）",
                c.command_path
            );
        }
    }
}
