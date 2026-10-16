# 术语词典

## 核心概念

### 硬层与软层

- **硬层（CLI）**：`novel` CLI 程序，负责世界状态日志的唯一入口、状态机、权限边界、事务一致性、错误码、封卷门禁、归档与防绕过。技术栈：Rust + SQLite。
- **软层（Spec 运行层）**：创作流程、阶段编排、提示词、规则注入、LLM/代理调用、验证流程。只能通过子进程调用 `novel` 命令访问世界状态。

### CLI Contract

软硬层唯一稳定边界，包括：
- 命令语法与参数约定
- 稳定 JSON 输出 schema
- 结构化错误对象 schema
- 退出码语义
- 实体 ID 与引用规则
- golden examples
- fixture worlds
- contract tests
- schema/version 演进策略

**错误协议（Error Protocol）**：
- **默认 V2**：CLI 默认错误协议为 V2，`--json` 失败输出为 `V2ErrorEnvelope`（`errorId`/`category`/`exitCode`/`message`，可选 `details`），进程退出码与信封 `exitCode` 由 `ErrorCatalog` 类别派生（类别退出码）。类型化权威源见 `crates/novel-cli/src/errors/catalog.rs`。错误码/退出码取值以 `contracts/cli/error-codes.md`/`exit-codes.md` 为 SSOT。
- **V1 弃用兼容**：V1 `ErrorObject`（`code`/`message`/`exitCode`/`suggestion`）与 per-entity 退出码仅在显式选择 V1 时输出，逐字保持 V1 回归基线，不破坏既有 V1 消费者；其移除由独立变更按 `repository/decisions.md` 三条件齐备准则执行。
- **命令契约注册表**：`CommandContract`（`contracts/mod.rs`）为命令语义唯一声明源，`ErrorCatalog` 为公开错误 ID 唯一声明源；`spec-gen` 从二者 + clap 树生成 `contracts/cli/` 受管产物，手改生成物即漂移红（`scripts/check-generated-drift.sh`）。

**非稳定实现细节**：SQLite schema、`.novel` 内部路径、正文文件布局、Rust module 名称、repository/storage 实现、migration 具体文件名。

### 世界状态（World State）

存储在目标小说项目 `.novel/novel.db` SQLite 数据库中的权威元数据与状态机状态，包括：
- 项目级元数据、设置、约束
- 卷、章节、涌现、事件、节点等实体
- 状态机状态（章节状态、涌现状态、卷状态）
- 主线落地裁决与规则检查结果
- 正文文件引用、hash、字数、更新时间
- 归档/冻结状态
- schemaVersion

### 章节正文存储策略

**混合存储**：SQLite 元数据索引 + 文件存储正文
- SQLite 记录：正文文件路径引用、hash、字数、更新时间、归档/冻结状态、状态机状态
- 正文文件：使用稳定、人可读的 Markdown 或纯文本格式，存储在 `.novel/content/chapters/<chapter-id>.md`
- 写入正文必须通过受控命令（如 `novel chapter update`），在同一用例中更新 SQLite 元数据和内容 hash
- hash 检测：防止用户绕过 CLI 直接编辑正文文件；外部变更时拒绝继续或要求显式重新导入

### 状态机

#### 章节状态机
`outline → drafting → review → approved`

- `review` 可回退到 `drafting` 或 `outline`
- 进入 `review` / `approved` 的**生效门禁清单**以 `contracts/cli/state-machine.md` §进入 review 门禁清单 / §进入 approved 门禁清单 为唯一事实来源（SSOT），含：正文已写入、无 hard 违规、主线落地裁决 `achieved=1`、裁决与检查绑定当前正文 hash（无 stale）、正文未被外部修改。
- **规划态门禁**（尚未生效，Spec 不得依赖）：残余已写入或显式为空、四相日志已写入。二者属语义上进入 `review` 的前置条件，待对应切片落地后并入 `state-machine.md` 门禁清单方成为生效门禁。规划态门禁对应实体清单见 `repository/dev-progress.md` §7.2，门禁生效状态见同文件 §4。
- `approved` 是章节内容状态终态；已归档是冻结/归档维度

#### 涌现状态机
必须按评分、决策、执行、归档顺序推进，不能跳步。

#### 卷状态机
卷末关闭大轮回前必须处置 pending 涌现、验证卷目标、处理卷内事件终态并写入监测报告。

### Spec 运行层

规划中的创作流程引擎（目录 `spec/`），负责：
- 流程编排与阶段推进
- 提示词构建与规则注入
- LLM 调用与代理调度
- 验证流程与质量门禁
- 通过 `novel ... --json` 子进程调用访问世界状态

约束：
- 不得直接读取或修改 SQLite 或 `.novel/content`
- 不得 import/require/link CLI 内部模块
- 不得依赖 SQLite 表结构、内部路径或 Rust 模块名
- 不得自行判断状态机推进条件
- 必须处理非零退出码并停止推进

### 章节四相日志（Four-Phase Log）

章节创作过程的结构化记录，包括：设定吸收、主线推进、涌现响应、节奏控制四个**语义维度**的日志。用于追踪创作意图执行情况。属语义上进入 `review` 的规划态门禁（生效状态见 §章节状态机 规划态门禁说明与 `repository/dev-progress.md` §7.2/§4）。

> **命名分层说明**：本条描述的是四相的**语义维度**（设定吸收/主线推进/涌现响应/节奏控制）。`docs/CLI架构设计.md` §章节后更新的 `phase-log add` 命令使用 `birth/dwell/decay/death` 作为**可配置的默认阶段名**（并允许"项目定义的阶段摘要"自定义）。二者是不同抽象层——语义维度描述四相各自记录什么创作意图，阶段名是 phase 字段的可选默认取值——不冲突。命令实现切片落地时须同时尊重语义维度与可配置阶段名。

### 主线落地裁决（Mainplot Landing Verdict）

记录章节对主线节点的推进情况：
- `node_id`：关联的主线节点 ID
- `achieved`：是否达成（0/1）
- 必须绑定适用的正文 hash
- 正文修复或重写后，旧裁决不得继续作为状态推进依据
- `achieved=1` 是章节进入 `review` 状态的必要条件

### 规则检查结果（Rule Check Result）

记录章节正文对世界设定与约束的符合性检查：
- 违规级别（检查结果）：`info` / `warning` / `hard` 三级；`hard` 为硬阻断，进入 `review` 前必须无 `hard` 违规
- 注意区分：约束定义（`constraints` 表的 severity）与检查结果（规则检查结果的 severity）是不同实体。经 V009 migration 三档化后，二者共用同一套 `info`/`warning`/`hard` 枚举（见 `domain/constraint.rs` `ConstraintSeverity`、`contracts/cli/json-schemas/` 相关 schema）。早期 `soft`/`hard` 两级已废弃。
- 必须绑定适用的正文 hash
- 正文修复或重写后，旧检查结果不得继续作为状态推进依据
- 存在 hard 违规时不得进入 `review` 状态

### 封卷（Volume Sealing）

卷结束后的冻结操作，确保：
- 所有 pending 涌现已处置
- 卷目标已验证
- 卷内事件终态已处理
- 监测报告已写入
- 后续不可修改卷内章节、涌现、事件状态

### 归档（Archive）

已完成实体的冻结状态：
- 章节 `approved` 后允许执行受控归档
- 归档后不得修改正文、裁决、残余、四相日志或 thread binding
- 归档是冻结/归档维度，与章节状态机状态（`approved`）正交

### Hash 检测与 Stale 保护

- **Hash 检测**：防止用户绕过 CLI 直接编辑正文文件。CLI 在读取、推进状态、归档或校验时必须通过 hash 检测外部变更，并拒绝继续或要求显式重新导入/确认。
- **Stale 保护**：规则检查结果和主线落地裁决必须绑定适用的正文 hash。正文修复或重写后，旧检查结果和旧裁决标记为 stale，不得继续作为状态推进依据。

### 残余（Residue）

章节创作过程中产生的长残余文本（可选），与章节正文分离存储在 `.novel/content/residues/<chapter-id>.md`。

### Chapter Context Packet (CCP)

CLI 生成的章节上下文数据包，用于 Spec 运行层构建 prompt。包含：
- P0 片段：必需上下文
- P1 片段：重要上下文
- audit 信息：上下文选择依据
- overflow 标记：预算超限提示

Spec 不得自行重排、删除、重压缩或补全 CLI canonical CCP 的 P0/P1 片段。

### schemaVersion

SQLite 数据库的 schema 版本号，用于追踪数据结构演进。CLI 负责 migration 管理与版本兼容性检查。
