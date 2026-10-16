# CLI State Machine Contract

> **本文件是 `novel` CLI 章节状态机的唯一事实来源（SSOT）。** 记录合法状态值、合法转换路径、终态约束与进入 `review`/`approved` 的门禁清单。
> 状态机语义是稳定契约（来源：`CLAUDE.md` §10，s04-state-machine-and-verdicts spec）。Spec 运行层只能通过 `novel chapter status <id> <target-status> --json` 推进状态并消费结果，不得自行判断状态可推进性。
> 任何状态机硬阻断 MUST 以非零退出码（`3` 前置条件阻断）+ 结构化错误对象返回，不得静默降级。

## 合法状态值

章节状态取以下四值之一，存储于 `chapters.status`（`TEXT NOT NULL DEFAULT 'outline'`）：

| 状态值 | 含义 |
|---|---|
| `outline` | 大纲阶段，正文尚未开始撰写。新建章节的初始状态。 |
| `drafting` | 撰写阶段，正文已写入或撰写中。 |
| `review` | 评审阶段，已通过进入 review 的全部前置门禁。 |
| `approved` | 已定稿终态。正文、裁决、规则检查均已固化，不可再修改。 |

非四态之一的目标状态值在参数解析层拒绝（退出码 `2`，错误码 `usage`）。

## 合法前向转换

| 转换 | 触发命令 | 门禁 |
|---|---|---|
| `outline → drafting` | `novel chapter status <id> drafting` | 无额外门禁（外部修改检测仍执行） |
| `drafting → review` | `novel chapter status <id> review` | 全部 review 门禁（见下） |
| `review → approved` | `novel chapter status <id> approved` | 当前必须为 `review` + 全部 review 门禁仍满足 + 无外部修改 |

## 合法回退转换

| 转换 | 触发命令 | 门禁 |
|---|---|---|
| `review → drafting` | `novel chapter status <id> drafting` | 跳过 review/approved 门禁，但仍检测外部修改 |
| `review → outline` | `novel chapter status <id> outline` | 跳过 review/approved 门禁，但仍检测外部修改 |

回退不要求门禁：回退本就是「打回重做」，门禁只在向 `review`/`approved` 前进时强制；重做后再次推进时门禁会重新校验。

## 终态约束

- `approved` 为终态：`approved` 不可转换为任何其他状态（含 `outline`/`drafting`/`review`/`approved` 自身）。
- 试图从 `approved` 推进至任意目标状态时返回退出码 `3`，错误码 `precondition_blocked`。
- `approved` 后不得修改正文、裁决、残余、四相日志或 thread binding（`CLAUDE.md` §7）。裁决命令在章节 `approved` 时拒绝写入（退出码 `3`）。

## 非法路径

合法值但非法路径在 guard 层拒绝（退出码 `3`，错误码 `precondition_blocked`），错误对象给出当前状态与允许的目标状态。典型非法路径：

- `outline → approved`（跳过 `drafting` 与 `review`）
- `outline → review`（跳过 `drafting`）
- `drafting → approved`（跳过 `review`）
- `approved → <任意>`（终态不可转换）

## 进入 `review` 的门禁清单

系统 SHALL 在 `drafting → review` 前验证以下全部条件，任一不满足则硬阻断（退出码 `3`，错误码 `precondition_blocked`）。门禁收集全部失败项一次性报告，不短路：

1. **正文已写入**：`chapters.content_hash` 为非空字符串 hash。
2. **无 hard 违规**：不存在绑定当前正文 hash 且 `severity='hard'` 且 `passed=0` 的规则检查结果（复用 `novel constraint check` 的检查结果）。
3. **主线落地裁决已设置且 `achieved=1`**：存在绑定当前正文 hash 的 `mainplot-landing` 裁决且 `achieved=1`。
4. **裁决与检查绑定 hash 与当前正文 hash 一致**（无 stale）：裁决的 `content_hash` 与检查结果的 `content_hash` 均等于当前 `chapters.content_hash`。
5. **正文未被外部修改**：正文文件重算 SHA-256 等于 `chapters.content_hash`（退出码 `21` 优先于门禁，独立检测，不混入门禁 details）。

正文经 `chapter update` 改变 hash 后，旧规则检查在成功重检前保持 stale 并阻断前向推进。针对当前 hash 的 `constraint check --input` 成功提交完整检查集后，CLI 原子替换该章旧检查集；旧检查不再造成 stale 阻断，门禁继续按新检查集和当前裁决判定。

## 进入 `approved` 的门禁清单

系统 SHALL 在 `review → approved` 前验证以下条件：

1. **当前状态为 `review`**（非法路径门禁，非 `review` 时退出码 `3`）。
2. **进入 `review` 的全部门禁仍满足**：检查与裁决未变为 stale、无 hard 违规、裁决 `achieved=1`。
3. **正文未被外部修改**（退出码 `21`，独立检测）。

> 进入 `approved` 复用 review 门禁重新跑一遍，确保 review 期间裁决未被改为未达成、检查未变 stale。

## 事务一致性

状态推进在 SQLite 事务中执行：校验当前状态 → 执行门禁检查 → 更新 `chapters.status` 与 `updated_at` → 提交。任一步失败回滚且不留下中间状态。门禁检查失败时不修改 `chapters.status`，章节状态保持原值。事务提交失败时返回退出码 `70`（错误码 `storage`）。

## 卷状态机

> 卷状态机语义是稳定契约（来源：`CLAUDE.md` §10，volume-sealing / volume-lifecycle spec）。Spec 运行层只能通过 `novel volume seal` / `novel volume archive` 推进卷状态并消费结果，不得自行判断卷可否封卷或归档。
> 退出码与错误码语义以 `error-codes.md` §A、`exit-codes.md` §A 为准，本节不重复定义。

### 合法状态值

卷状态取以下四值之一，存储于 `volumes.status`（`TEXT NOT NULL DEFAULT 'active'`）：

| 状态值 | 含义 |
|---|---|
| `active` | 活动态。卷可创建、修改属性、绑定目标、下属实体可写。新建卷的初始状态。 |
| `sealing` | 封卷中间态。已通过全部封卷前置检查但尚未冻结；为卷末监测报告生成留出窗口。 |
| `sealed` | 已封闭态。已生成卷末监测报告；可执行归档推进。 |
| `archived` | 归档终态。卷内所有实体冻结，拒绝任何修改操作。 |

### 合法前向转换

| 转换 | 触发命令 | 前置检查 |
|---|---|---|
| `active → sealing` | `novel volume seal <id>` | 全部封卷前置检查（章节状态、涌现处置、卷目标达成），见 `volume-sealing` spec 与下方门禁清单 |
| `sealing → sealed` | `novel volume archive <id>`（推进过程中先生成监测报告再推进至 `sealed`，随后推进至 `archived`） | 卷末监测报告已生成并写入；归档推进由同一命令在事务内完成 `sealing → sealed → archived` 的连续推进 |
| `sealed → archived` | `novel volume archive <id>` | 见上行；`archive` 命令从 `sealing` 或 `sealed` 推进至 `archived`，过程中生成/复用监测报告 |

### 封卷前置检查门禁清单

`novel volume seal` SHALL 在 `active → sealing` 前验证以下全部条件，任一不满足则硬阻断（退出码 `3`，对应细分错误码见 `error-codes.md`）。门禁收集全部失败项一次性报告，不短路：

1. **卷内所有章节状态为 `approved`**（不满足→`chapters_not_approved`）。
2. **卷内所有涌现记录已处置**：实际查询 `emergences` 表，状态须为 `executed` 或 `archived`；`pending`/`rated`/`decided` 阻断（不满足→`emergences_not_handled`）。表结构/查询异常返回退出码 `70`（`storage`），**不**降级为 warn 通过。
3. **所有必达卷目标已达成**：实际查询 `mainplot_nodes.achieved`，必达目标对应主线节点须 `achieved=1`；可选目标未达成仅提示不阻断（不满足→`goals_not_achieved`）。表结构/查询异常返回退出码 `70`。
4. **卷当前状态为 `active`**：卷状态为 `sealed`/`archived` 时拒绝重复封卷（退出码 `3`，`volume_already_sealed`）。

### 卷末监测报告窗口

`sealing` 作为"已通过前置检查但尚未冻结"的中间态，在归档命令执行时生成卷末监测报告（卷内章节终态摘要、涌现处置摘要、卷目标达成摘要），报告写入后再推进到 `sealed`，最终推进到 `archived`。报告生成与状态推进在归档命令的同一流程内完成，`sealing` 态本身不对外暴露为长驻状态。

### 终态约束与归档冻结

- `archived` 为卷终态：归档后卷内所有实体（章节正文、裁决、残余、四相日志、thread binding、涌现、主线节点）MUST 进入冻结态，拒绝任何修改操作（退出码 `3`，`volume_already_archived`）。
- 卷状态机单向推进，不提供回退命令：无 `sealing → active`、`sealed → active`、`archived → sealed` 的推进命令。
- 归档冻结在 service 层跨实体统一判定（CLAUDE.md §6/§10）：`novel chapter update`/`novel chapter status`/`novel emergence rate|decide|execute`/`novel verdict set-mainplot-landing`/`novel volume goal set` 等写命令在操作实体所属卷为 `archived` 时，统一返回退出码 `3` 与 `volume_already_archived`，不得散落在各 repository/storage 层。
