# CLI Controlled View / Scope Names Contract

> **本文件是 `novel` CLI 受控 view/scope 名称及其机器语义的唯一事实来源（SSOT）。** 仓库内其余文档（含 `docs/` 与 `reference/`）不得重复定义受控 view/scope 名称表，只能引用本文件。
> 受控 view/scope 名称是稳定契约。来源：`CLAUDE.md` §9，`contracts/cli/README.md` §稳定契约，cli-contract spec。
> 任何受控 view/scope 名的机器输出 MUST 使用稳定实体 ID 与 sourceRefs，不得要求调用方解析内部文件路径。

受控 view/scope 名称分两层：**§A 生效契约名**（名称已冻结为契约意图）与 **§B 预留或未固化标注**（登记可能随切片演进的视图/作用域方向，未固化前 Spec 不得依赖）。§A 名按承载命令实现状态再分两类：组 1/2/3 承载命令 `setting render` / `constraint render` 与组 4 承载命令 `chapter context` 均已实现并通过契约测试（`setting-constraint-render` 切片 2026-07-21 归档、`chapter-context-packet` 切片 2026-07-22 实现），Spec **现在可调用**。

> **关于 §A 的实现状态分层（本文件与 `exit-codes.md`/`error-codes.md` 的关键差异）。** `exit-codes.md`/`error-codes.md` 的 §A 生效码有 `errors/mod.rs` 的 `ExitCode`/`ErrorCode` enum + golden example + 契约测试背书，Spec **现在就能依赖**。本文件的 §A 名中，组 1/2/3 的承载命令已由 `setting-constraint-render` 切片实现并通过契约测试（`crates/novel-cli/src/commands/{setting,constraint}.rs` + `contracts/cli/contract-tests/{setting,constraint}_commands.rs` + golden examples），组 4 `chapter context` 承载命令已由 `chapter-context-packet` 切片实现并通过契约测试（`crates/novel-cli/src/commands/chapter.rs` 的 `context` 子命令 + `services/chapter_context_service.rs` + `renderers/chapter_context_renderer.rs` + `contracts/cli/contract-tests/chapter_context_commands.rs` + golden examples），Spec **现在可调用**。组 1/2/3/4 名均升格为"已实现可调用"（同节内更新标注，不移动分组）。

> 本文件与 `contracts/cli/exit-codes.md`、`contracts/cli/error-codes.md` 并列对称，形成四块 SSOT：`exit-codes.md` 管 `exitCode` 数值、`error-codes.md` 管 `code` 字符串、`json-output.md` 管输出结构、本文件管受控 `--view`/`--scope` 名称。本文件不重复定义退出码/错误码语义，也不定义各 view/scope 的输出 JSON 结构（属各命令切片的 `json-schemas/` 职责）。

## §A 生效契约名

> 以下名称已冻结为契约意图，按命令 × 参数四组枚举。**组 1/2/3 的承载命令（`setting render` / `constraint render`）已由 `setting-constraint-render` 切片实现并通过契约测试，组 4 的承载命令（`chapter context`）已由 `chapter-context-packet` 切片实现并通过契约测试，Spec 现在可调用**。后续切片不得各自发明或漂移受控名称。

### 组 1：`setting render --view`

> 归属切片方向：`setting-constraint-import`（由 `setting-constraint-render` 切片实现）。**已实现可调用**——`novel setting render --view` 已实现并通过契约测试（`contracts/cli/contract-tests/setting_commands.rs` + `setting-render-llm/full.golden.json`），Spec 现在可调用。

| view 名 | 机器语义 |
|---|---|
| `intro` | 世界简介、核心设定摘要。 |
| `four-poles` | 四极边界与不可越界的硬约束。 |
| `world` | 世界观细节，不含长篇风格样例。 |
| `style` | 文风、叙事质感、禁用表达等规则。 |
| `summary` | 压缩摘要，context 紧张时的兜底视图。 |
| `full` | 完整设定视图，人读审查、导出或离线核对。 |
| `llm` | 面向当前任务的预算内组合视图，Spec 默认写前入口。 |

### 组 2：`constraint render --view`

> 归属切片方向：`setting-constraint-import`（由 `setting-constraint-render` 切片实现）。**已实现可调用**——`novel constraint render --view` 已实现并通过契约测试（`contracts/cli/contract-tests/constraint_commands.rs` + `constraint-render-view-chapter.golden.json`），Spec 现在可调用。本组是 `constraint render` 的 `--view` 受控名（控制输出面向 LLM 还是人类），与组 3 的 `--scope`（控制规则作用范围）是同一命令的两个正交受控参数。

| view 名 | 机器语义 |
|---|---|
| `llm` | 面向 LLM 的规则视图，机器可消费。 |
| `human` | 人读规则视图。 |

### 组 3：`constraint render --scope`

> 归属切片方向：`setting-constraint-import`（由 `setting-constraint-render` 切片实现）。**已实现可调用**——`novel constraint render --scope` 已实现并通过契约测试，Spec 现在可调用。`--scope` 用于避免全量规则注入；`chapter` scope 需 `--chapter-id` 参数，`volume` scope 需 `--volume-id` 或 `--chapter-id` 之一（取章节所属卷）。受控 `--scope` 是三枚举 `world|volume|chapter`，与既有 `--type` 单条形态的自由字符串 scope 语义不同（受控形态由 `--view` 出现触发）。

| scope 名 | 机器语义 |
|---|---|
| `world` | 全局不变式。 |
| `volume` | 当前卷相关约束。 |
| `chapter` | 本章相关约束，需要 `--chapter-id`。 |

### 组 4：`chapter context --view`

> 归属切片方向：`chapter-context-packet`。**已实现可调用**——`novel chapter context --view` 已实现并通过契约测试（`crates/novel-cli/src/commands/chapter.rs` 的 `context` 子命令 + `contracts/cli/contract-tests/chapter_context_commands.rs` + `chapter-context-writer.golden.json`），Spec 现在可调用。该命令是只读命令，输出 CLI canonical Chapter Context Packet（P0/P1/audit/overflow）。本切片实现有数据支撑的 P0/P1 子集（章契约/Snapshot 层无表输出 omitted），后续 change 补表后填。

| view 名 | 机器语义 |
|---|---|
| `writer` | 写作前上下文。 |
| `verifier` | 验证前上下文。 |
| `repair` | 修复前上下文。 |

### 同名异义消歧

受控名按"命令 × 参数"分组确定语义，同名在不同组下语义独立，不可跨组混用：

- `llm` 在 `setting render --view`（组 1，设定预算内组合视图）与 `constraint render --view`（组 2，面向 LLM 的规则视图）语义不同。前者是设定注入视图、后者是规则注入视图，分属不同命令。
- `world` 在 `setting render --view`（组 1，世界观细节视图）与 `constraint render --scope`（组 3，全局不变式作用域）语义不同。前者是设定内容视图、后者是规则作用范围，分属不同命令不同参数。

> 查询某受控名的机器语义时，必须同时给出命令与参数（如"`setting render --view` 的 `world`"），不可只说"world"——同名异义按分组消歧。

## §B 预留或未固化标注

> 以下名登记语义方向意图，**对应切片实现并通过测试前不生效**，Spec MUST NOT 依赖其语义。预留名不是当前生效契约，连名称都尚未固化（实现时可调整）。预留节登记的是方向不是承诺，非穷举，后续切片可增补。

| 受控参数 | 预留名 | 方向意图 | 归属切片方向 |
|---|---|---|---|
| `chapter context --view` | `audit` | 审计视图，呼应 `docs/CLI架构设计.md` §9 的 `audit` 字段 | `chapter-context-packet` 的审计场景 |
| `chapter context --view` | `export` | 导出视图，呼应 `CLAUDE.md` §7 的导出命令 | `chapter-context-packet` 的导出场景 |
| `setting render --view` | `diff` | 设定差异视图，呼应 hash 检测场景（正文/设定被外部修改时给出差异） | `setting-constraint-import` 的 hash 检测场景 |

> 预留名登记的是方向意图，不是承诺。对应切片实现时可调整方向后再升格。预留名不阻塞任何当前行为（承载命令均未实现）。

### 升格路径

两条升格路径并存：

1. **§B 预留名 → §A 生效契约名**：某预留名（如 `chapter context --view audit`）对应的切片实现并通过契约测试后，在归档该切片时把该名从 §B 移至 §A，标注为生效契约名，Spec 方可依赖其语义。升格时须在 `crates/novel-cli/` 实现承载命令、补充 golden example 与契约测试。升格前 §B 标注的方向意图可在实现时调整。

2. **§A 冻结名 → 已实现可调用**：§A 某冻结名（如 `setting render --view llm`）对应的承载命令实现并通过契约测试后，在归档该命令切片时把该名从"冻结为契约意图"升格为"已实现可调用"标注（更新 §A 标注，不移动分组），Spec 调用该命令不再失败。升格时须在 `crates/novel-cli/` 实现该命令、补充 golden example 与契约测试。

> 区分：路径 1 是"名称从无到有"（§B 未固化名升格为 §A 生效名，跨节移动）；路径 2 是"命令从无到有"（§A 名早冻结，承载命令实现后从"冻结意图"升格为"已实现可调用"，同节内更新标注）。两者升格前提相同——承载命令实现并通过契约测试。
