# CLI 契约对齐审计报告

> **本报告为参考文档，不构成新契约。** 本报告为 2026-07-24 三基线审计快照，驱动的修复切片 s01-s08 已全部归档（见 `repository/dev-progress.md` §6.2），本报告发现项已闭环，不再作为活跃进度追踪；当前实现状态以 `repository/dev-progress.md` 为准。
>
> - 不引入新的稳定契约；不改变 `schemaVersion`；不改变 `contracts/cli/` 任何 SSOT（`commands.md`/`exit-codes.md`/`error-codes.md`/`json-output.md`/`views-scopes.md`/`entity-ids-sourcerefs.md`）的实际取值。
> - 任何判定若与实际代码或契约 SSOT 冲突，**以代码与契约为准回溯**，报告判定视为待回溯，须在后续修复 slice 中更正。
> - 修复簇为「建议」方向，实际拆分以 `openspec-slices-plan` 评估为准；本报告不越权定义 slice。
>
> **生成基线日期**：2026-07-24
> **三基线快照来源**：
> - 契约基线：`contracts/cli/` 全树结构化梳理（46 命令清单 + 16 退出码 + 错误码 + 26 JSON schema + 29 golden + 契约测试 + view/scope）
> - 实现基线：`crates/novel-cli/src/` 全模块结构化梳理（12 组 ~40 子命令 + 分层 + 三套状态机 + 存储/一致性 + 错误渲染）
> - 测试/规格基线：`crates/novel-cli/tests/` + `contracts/cli/contract-tests/` + `openspec/specs/` + `repository/` 16 条系统设计不变量

---

## 1. 标准契约格式 Rubric

本节复述 `contracts/cli/` 的契约格式标准，作为「逐命令四维矩阵 → 标准格式」维度的判定基准。**本节不复制 SSOT 取值表**（退出码/错误码/view/scope 数值见对应 SSOT 文件），仅描述其结构与判定规则。

### 1.1 SSOT 架构

`contracts/cli/README.md` 声明 CLI Contract 为软硬层唯一稳定边界，由六个 SSOT 文件 + 两格式 SSOT 分担，每个关注点只由一个文件拥有，其余文件只能引用不得重定义：

| 关注点 | SSOT 文件 |
|---|---|
| 命令语法 / 参数 / 每命令 `--json` | `commands.md` |
| `exitCode` 数值 | `exit-codes.md` |
| `code` 错误字符串 | `error-codes.md` |
| `--json` 输出结构 / `schemaVersion` 规则 / 错误对象字段 | `json-output.md` |
| 受控 `--view`/`--scope` 名 | `views-scopes.md` |
| `sourceRefs` 字符串格式 + 实体前缀表 | `entity-ids-sourcerefs.md` |

支撑 SSOT：`state-machine.md`（章节 + 卷状态机）、`verdicts.md`（裁决数据结构）。`examples/`、`json-schemas/`、`contract-tests/`、`fixtures/`、`generated/` 为机器可验证证据层。

### 1.2 两层冻结模型（§A / §B）

`exit-codes.md`/`error-codes.md`/`views-scopes.md`/`entity-ids-sourcerefs.md` 各自分两层：
- **§A 生效契约**：有实现 + golden + 契约测试背书，Spec MAY 依赖。
- **§B 预留**：意图已登记但切片未实现，Spec MUST NOT 依赖。

`views-scopes.md` 额外有第三维度：§A 名再分「已实现可调用」vs「冻结为契约意图（命令未落地）」。

### 1.3 per-command 条目模板

`commands.md` 中每条命令用如下结构（共 46 条 `## \`novel …\`` H2）：

1. `## \`novel <group> <sub>\`` — H2 标题（命令签名 token）
2. 一行用途说明
3. ```` ``` ```` fence 内 canonical 语法行，如 `novel init <name> [--path <dir>] [--json]`
4. `### 参数与选项` — 三列表：`参数/选项 | 必填 | 说明`；位置参数 `<...>`、标志 `--flag <...>`；必填 = 是/否/条件
5. `### 行为语义` — 要点列表：存储写入/只读声明、人读 vs JSON 输出形态、错误条件（内联 `code:` / 退出码引用）、交叉引用 `json-schemas/*.schema.json` 与 `views-scopes.md`
6. `### 退出码` — 两列表：`退出码 | 场景`，列出该命令可返回的所有退出码，常在场景内用反引号标错误码（如 `code: "setting_not_found"`）

互斥形态命令（`constraint render`、`constraint check`）以 `### 形态一`/`### 形态二` 嵌二级标题，各自重列语法 + 参数表，再共用 `### 退出码（两形态共用）`。

**模板示例**（`novel setting get`，最简完整条目）：

```
## `novel setting get`

查询指定配置项的值。

```
novel setting get <name> [--json] [--path <dir>]
```

### 参数与选项
| 参数/选项 | 必填 | 说明 |
|---|---|---|
| `<name>` | 是 | 配置项名称。 |
| `--json` | 否 | 输出稳定结构化 JSON 对象。 |
| `--path <dir>` | 否 | 世界所在目录。默认当前工作目录。 |

### 行为语义
- 人读格式输出：`<name>: <value>`
- JSON 格式输出顶层对象至少含：`schemaVersion`、`name`、`value`、`createdAt`。
- 配置项不存在时返回退出码 6，错误对象 `code: "setting_not_found"`。
- 世界未初始化时返回退出码 3。

### 退出码
| 退出码 | 场景 |
|---|---|
| `0` | 成功查询配置项 |
| `2` | 参数错误：`<name>` 缺失 |
| `3` | 前置条件阻断：世界未初始化 |
| `6` | 数据不存在：配置项不存在 |
| `70` | 内部/存储错误 |
```

> 判定「标准格式」维：一条命令是否具备上述六要素、形态分裂是否规范、退出码表是否覆盖其行为语义所列条件。

### 1.4 判定三态定义

- ✅ 一致：该维无偏离。
- ⚠️ 部分：有小瑕疵但不阻断（如输出风格不一、标志事实上 no-op、形态入口非显性）。
- ❌ 违反：阻断性偏离（违反状态机/一致性/防绕过/分层契约/契约自洽）。

⚠️/❌ 必附证据指针（`文件:行`）。「系统设计一致」维的 ❌/⚠️ 必标注触及的不变量编号（1–16，见 §4 不变量表）。

---

## 2. CLI 实现命令清单

来源：`crates/novel-cli/src/commands/mod.rs` 注册的命令组 + 各组实现文件。所有命令接受 `[--path <dir>]`（世界目录，默认 cwd）。`--json` 列 = 是否在 query 命令上接通；写命令多为人读成功或静默。

| 组 | 子命令 | 签名（节选） | 标志 | `--json` | 实现源文件 |
|---|---|---|---|---|---|
| init | `init` | `init <NAME> [--path] [--json]` | `--path`,`--json` | ✅ | `commands/init.rs` |
| info | `info` | `info [--path] [--json]` | `--path`,`--json` | ✅ | `commands/info.rs` |
| setting | `set` | `setting set <name> <value> [--path]` | `--path` | ❌(静默) | `commands/setting.rs` |
| | `get` | `setting get <name> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `list` | `setting list [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `render` | `setting render --view <受控名> [--path] [--json]` | `--view`(7值枚举),`--path`,`--json` | ✅ | |
| constraint | `define` | `constraint define --type --scope --name --content --severity [--path]` | `--type`,`--scope`,`--name`,`--content`,`--severity`,`--path` | ❌(静默) | `commands/constraint.rs` |
| | `render` | `constraint render`（双形态） | 形态1:`--type/--scope/--name`；形态2:`--view/--scope`；`--chapter-id`,`--volume-id`,`--path`,`--json` | ✅ | |
| | `list` | `constraint list [--scope --severity] [--path] [--json]` | `--scope`,`--severity`,`--path`,`--json` | ✅ | |
| | `check` | `constraint check --chapter <id> (--input <file> \| --list [--verify-hash]) [--path] [--json]` | `--chapter`,`--input`/`--list`,`--verify-hash`,`--json`,`--path` | ✅(list) | |
| chapter | `create` | `chapter create <TITLE> [--path] [--json]` | `--path`,`--json` | ✅ | `commands/chapter.rs` |
| | `update` | `chapter update <ID> --content <FILE> [--path]` | `--content`,`--path` | ❌(静默) | |
| | `get` | `chapter get <ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `list` | `chapter list [--verify-hash] [--path] [--json]` | `--verify-hash`,`--path`,`--json` | ✅ | |
| | `reimport` | `chapter reimport <ID> [--path]` | `--path` | ❌(静默) | |
| | `status` | `chapter status <ID> <TARGET-STATUS> [--path] [--json]` | `--path`,`--json`；TARGET ∈ {outline,drafting,review,approved} | ✅ | |
| | `context` | `chapter context <ID> --view <VIEW> --budget <N> [--path] [--json]` | `--view`(3值),`--budget`(正整数),`--path`,`--json` | ✅ | |
| verdict | `set-mainplot-landing` | `verdict set-mainplot-landing <CHAPTER-ID> --node-id --achieved <0\|1> [--path] [--json]` | `--node-id`,`--achieved`,`--path`,`--json` | ✅ | `commands/verdict.rs` |
| volume | `create` | `volume create <NAME> [--description --order] [--path]` | `--description`,`--order`,`--path` | ❌(人读"ID = N") | `commands/volume.rs` |
| | `info` | `volume info <VOLUME_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `list` | `volume list [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `seal` | `volume seal <VOLUME_ID> [--dry-run] [--path] [--json]` | `--dry-run`,`--path`,`--json` | ✅ | |
| | `archive` | `volume archive <VOLUME_ID> [--path]` | `--path` | ❌(人读) | |
| | `goal set` | `volume goal set <VOLUME_ID> <MAINPLOT-NODE-ID> [--description] [--optional] [--path]` | `--description`,`--optional`,`--path` | ❌ | |
| | `goal get` | `volume goal get <VOLUME_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `goal remove` | `volume goal remove <VOLUME_ID> <GOAL_ID> [--path]` | `--path` | ❌ | |
| mainplot | `node create` | `mainplot node create <VOLUME_ID> --title [--description --target-chapters] [--path] [--json]` | `--title`,`--description`,`--target-chapters`(csv),`--path`,`--json` | ✅ | `commands/mainplot.rs` |
| | `node list` | `mainplot node list <VOLUME_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `node get` | `mainplot node get <NODE_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `node update` | `mainplot node update <NODE_ID> --achieved <0\|1> [--path]` | `--achieved`,`--path` | ❌ | |
| thread | `create` | `thread create <VOLUME_ID> --name [--description] [--path] [--json]` | `--name`,`--description`,`--path`,`--json` | ✅ | `commands/thread.rs` |
| | `bind` | `thread bind <THREAD_ID> --chapter [--type --note] [--path]` | `--chapter`,`--type`,`--note`,`--path` | ❌ | |
| | `list` | `thread list <VOLUME_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `query` | `thread query --chapter <CHAPTER_ID> [--path] [--json]` | `--chapter`,`--path`,`--json` | ✅ | |
| | `update` | `thread update <THREAD_ID> --status <STATUS> [--path]` | `--status`,`--path` | ❌ | |
| emergence | `create` | `emergence create <VOLUME_ID> --title --description [--chapter] [--path] [--json]` | `--title`,`--description`,`--chapter`,`--path`,`--json` | ✅ | `commands/emergence.rs` |
| | `rate` | `emergence rate <EMERGENCE_ID> --rating <0-10> --reason [--path] [--json]` | `--rating`,`--reason`,`--path`,`--json` | ✅ | |
| | `decide` | `emergence decide <EMERGENCE_ID> --decision <accept\|defer\|reject> --reason [--path] [--json]` | `--decision`,`--reason`,`--path`,`--json` | ✅ | |
| | `execute` | `emergence execute <EMERGENCE_ID> --target <CHAPTER_ID> [--note] [--path] [--json]` | `--target`,`--note`,`--path`,`--json` | ✅ | |
| | `archive` | `emergence archive <EMERGENCE_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `list` | `emergence list <VOLUME_ID> [--status] [--path] [--json]` | `--status`,`--path`,`--json` | ✅ | |
| | `get` | `emergence get <EMERGENCE_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| export | `volume` | `export volume <VOLUME_ID> [--path] [--json]` | `--path`,`--json`(默认JSON) | ✅ | `commands/export.rs` |
| | `chapter` | `export chapter <CHAPTER_ID> [--path] [--json]` | `--path`,`--json` | ✅ | |
| | `full` | `export full [--path] [--json]` | `--path`,`--json` | ✅ | |
| verify | `integrity` | `verify integrity [--path] [--json]` | `--path`,`--json` | ✅ | `commands/verify.rs` |

> 注：实现侧 `emergence rate/decide/execute/archive` 实际接通了 `--json`，而契约侧 `commands.md` 把这些标为 **no JSON（人读/静默）**——这是契约↔实现的偏差，详见 §3 矩阵与 §4 发现 F-01。

---

## 3. 逐命令四维矩阵

四维：① 契约一致（对照 `commands.md`/`exit-codes.md`/`error-codes.md`/`json-schemas`/`views-scopes`）② 标准格式（对照 §1 rubric）③ 需求满足（对照 `openspec/specs/**`）④ 系统设计一致（对照 §4 不变量 1–16）。触不变量 = 仅在④为 ⚠️/❌ 时标注。证据指针 = `文件:行`。

> 矩阵行的「触不变量」列填 `N` 指违反第 N 条不变量；多个用 `N,M`；空表示④为 ✅ 不触不变量。不变量编号见 §4 表。

### 3.1 init / info / setting 组（6 命令）

| 命令 | ①契约 | ②格式 | ③需求 | ④系统 | 触不变量 | 证据指针 |
|---|---|---|---|---|---|---|
| `novel init` | ✅ | ✅ | ✅ | ✅ | — | `contracts/cli/json-output.md §5.6`（init 成功输出不纳入机器契约，仅 error golden） |
| `novel info` | ⚠️ | ✅ | ✅ | ⚠️ | — | `crates/novel-cli/src/renderers/info.rs`：JSON 用 `serde_json::to_string`（紧凑）而非 sibling 的 `to_string_pretty`，输出风格不一致（非阻断） |
| `novel setting set` | ✅ | ✅ | ✅ | ✅ | — | `commands/setting.rs`：静默成功，与 `commands.md` 一致 |
| `novel setting get` | ✅ | ✅ | ✅ | ✅ | — | 模板示例命令，§1.3 摘引 |
| `novel setting list` | ✅ | ✅ | ✅ | ✅ | — | — |
| `novel setting render --view` | ✅ | ✅ | ✅ | ✅ | — | `views-scopes.md §A 组1`（7 视图已实现可调用）；`setting-constraint-render` 切片 2026-07-22 归档 |

### 3.2 constraint 组（4 命令）

| 命令 | ①契约 | ②格式 | ③需求 | ④系统 | 触不变量 | 证据指针 |
|---|---|---|---|---|---|---|
| `novel constraint define` | ⚠️ | ✅ | ⚠️ | ✅ | — | `crates/novel-cli/src/services/constraint_service.rs:62`：suggestion 文案「请先删除旧约束（功能待实现）」指向未实现的 `constraint delete`（见 §7 未来倡议） |
| `novel constraint render`（双形态） | ✅ | ✅ | ✅ | ✅ | — | `views-scopes.md §A 组2/3`；`setting-constraint-render` 归档 |
| `novel constraint list` | ✅ | ✅ | ✅ | ✅ | — | — |
| `novel constraint check` | ❌ | ⚠️ | ✅ | ✅ | —(契约自洽) | `contracts/cli/commands.md`（`constraint check` 写形态）：章节未找到列退出码 6 + `chapter_not_found`，但 `chapter_not_found` 在 `error-codes.md`/`exit-codes.md` 是退出码 20 的规范子码（见 §5 发现 C-09）；形态分裂下 §1 rubric 形态二表头略不显性 |

### 3.3 chapter / verdict 组（7 命令）

| 命令 | ①契约 | ②格式 | ③需求 | ④系统 | 触不变量 | 证据指针 |
|---|---|---|---|---|---|---|
| `novel chapter create` | ✅ | ✅ | ✅ | ✅ | — | — |
| `novel chapter update` | ✅ | ✅ | ✅ | ❌ | 9 | `crates/novel-cli/src/repositories/chapter_repository.rs:103-110`：SQLite 更新失败后正文已替换，无回滚/补偿删除，注释自承「记录为不一致状态」 |
| `novel chapter get` | ✅ | ✅ | ✅ | ✅ | — | `chapter_repository.rs:129` 重算 hash 检测外部修改（exit 21） |
| `novel chapter list` | ✅ | ✅ | ✅ | ✅ | — | `--verify-hash` 已接通 |
| `novel chapter reimport` | ⚠️ | ✅ | ❌ | ✅ | — | 无 `openspec/specs/**` Given/When/Then 场景（仅 `repository/storage-model.md` 提及为修复机制，非 spec） |
| `novel chapter status` | ✅ | ✅ | ✅ | ✅ | — | `validators/guards/state_transition_guard.rs`：非短路多 guard + hash 绑定 + stale 检测 + approved 终态 |
| `novel chapter context` | ✅ | ✅ | ✅ | ✅ | — | `chapter-context-packet` 切片 + `@codegen` 生成测试 |
| `novel verdict set-mainplot-landing` | ✅ | ✅ | ✅ | ✅ | — | `services/verdict_service.rs:54`：approved 硬阻断；`:62` 重算 hash 前置（exit 21） |

### 3.4 volume 组（8 命令）

| 命令 | ①契约 | ②格式 | ③需求 | ④系统 | 触不变量 | 证据指针 |
|---|---|---|---|---|---|---|
| `novel volume create` | ✅ | ✅ | ✅ | ❌ | 12 | `crates/novel-cli/src/commands/volume.rs`：command 层直接 `connection::open_existing_at`，应下沉 service（与 s01 组不一致） |
| `novel volume info` | ✅ | ✅ | ✅ | ❌ | 12 | 同上，command 层开 DB |
| `novel volume list` | ✅ | ✅ | ✅ | ❌ | 12 | 同上 |
| `novel volume seal` | ⚠️ | ✅ | ✅ | ❌ | 12 | `services/volume_service.rs:81`：`seal_volume` 落 `sealing`（非 `sealed`），`advance_to_sealed` 注明「不单独暴露为命令」——状态机入口非显性；command 层开 DB（不变量 12） |
| `novel volume archive` | ✅ | ✅ | ✅ | ❌ | 12 | `services/volume_service.rs:233`：`sealing`→`advance_to_sealed`→`archived`；command 层开 DB |
| `novel volume goal set` | ✅ | ✅ | ✅ | ❌ | 12 | `services/volume_service.rs` inline `volume_freeze_guard`（goal set/remove 未抽 guard 调用）+ command 层开 DB |
| `novel volume goal get` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB |
| `novel volume goal remove` | ✅ | ✅ | ✅ | ❌ | 12 | 同上 |

### 3.5 mainplot / thread / emergence 组（13 命令）

| 命令 | ①契约 | ②格式 | ③需求 | ④系统 | 触不变量 | 证据指针 |
|---|---|---|---|---|---|---|
| `novel mainplot node create` | ✅ | ✅ | ✅ | ❌ | 12 | `commands/mainplot.rs`：command 层开 DB + `mainplot_service.rs:40` `.expect("序列化章节 ID 列表失败")` |
| `novel mainplot node list` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB |
| `novel mainplot node get` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB |
| `novel mainplot node update` | ⚠️ | ⚠️ | ⚠️ | ❌ | 12 | command 层开 DB；`commands.md` 有记但 `mainplot_contract_test.rs` 未覆盖 update（见 §6 T-01）；`mainplot-node-management` spec 场景边界略模糊 |
| `novel thread create` | ✅ | ✅ | ✅ | ❌ | 12 | `commands/thread.rs`：command 层开 DB + command 层 `println!` 渲染人读成功消息（分层违反，见 §5 F-03） |
| `novel thread bind` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB + `thread_service.rs:53` `.expect("validate_binding_type 已验证…")`；`commands/thread.rs` `println!` |
| `novel thread list` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB |
| `novel thread query` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB |
| `novel thread update` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB + `println!` |
| `novel emergence create` | ✅ | ✅ | ✅ | ❌ | 12 | `commands/emergence.rs`：command 层开 DB + `println!` |
| `novel emergence rate` | ❌ | ✅ | ✅ | ❌ | 12 | `commands.md` 标 no-JSON（人读），实现 `commands/emergence.rs` 接通 `--json`——契约↔实现 `--json` 偏差（见 F-01）；command 层开 DB + `println!` |
| `novel emergence decide` | ❌ | ✅ | ✅ | ❌ | 12 | 同 F-01：`commands.md` no-JSON，实现有 `--json`；command 层开 DB |
| `novel emergence execute` | ❌ | ✅ | ✅ | ❌ | 12 | 同 F-01；command 层开 DB |
| `novel emergence archive` | ❌ | ✅ | ✅ | ❌ | 12 | 同 F-01；command 层开 DB |
| `novel emergence list` | ✅ | ✅ | ✅ | ❌ | 12 | command 层开 DB |
| `novel emergence get` | ⚠️ | ⚠️ | ⚠️ | ❌ | 12 | `emergence_contract_test.rs` 未覆盖 get（见 §6 T-02）；command 层开 DB |

### 3.6 export / verify 组（4 命令）

| 命令 | ①契约 | ②格式 | ③需求 | ④系统 | 触不变量 | 证据指针 |
|---|---|---|---|---|---|---|
| `novel export volume` | ⚠️ | ✅ | ✅ | ❌ | 12 | `commands/export.rs`：`--json` 事实上 no-op（默认即 JSON，标志被忽略，见 F-04）；command 层开 DB |
| `novel export chapter` | ❌ | ⚠️ | ✅ | ❌ | 12, —(契约自洽) | `commands.md`（`export chapter`）退出码表自相矛盾：列退出码 6 又注「章节使用 `chapter_not_found`，退出码 20」（见 C-05）；`--json` no-op；command 层开 DB |
| `novel export full` | ⚠️ | ✅ | ✅ | ❌ | 12 | `--json` no-op；command 层开 DB |
| `novel verify integrity` | ❌ | ⚠️ | ✅ | ❌ | 11,12 | `commands/verify.rs:69-72`：`std::process::exit(exit_code)` 绕过 `lib::run()`→`render_error`，hard 违规不经结构化错误对象（违反不变量 11）；command 层开 DB（12） |

---

## 4. 系统设计不变量（判定基准）

以下 16 条不变量取自 `repository/architecture.md`/`contract-boundary.md`/`cli-modules.md`/`storage-model.md`/`terminology.md`，作为「系统设计一致」维度的判定锚点。本表为**索引**，具体语义引用对应 `repository/` 文件，不重抄全文。

| # | 不变量 | 来源 |
|---|---|---|
| 1 | CLI 是世界状态唯一入口；外部不经 novel 命令不得访问世界状态；Spec/Skill/Web/脚本不得绕 service/guard | `repository/architecture.md` |
| 2 | Spec 运行层只调 `novel` 子进程；不得直接读写 SQLite/`.novel/content`、import CLI Rust 模块、忽略非零退出码 | `repository/contract-boundary.md` |
| 3 | SQLite 是元数据权威索引（含正文路径/hash/字数/状态机/裁决/检查/冻结/schemaVersion） | `repository/storage-model.md` |
| 4 | 章节正文 = 文件 + SQLite 元数据；写入须在 SQLite 事务内原子（temp+replace） | `repository/storage-model.md` |
| 5 | Hash 绑定：读正文/推进状态/归档/绑裁决检查须重算 SHA-256 比对；失配 exit 21 | `repository/storage-model.md`/`terminology.md` |
| 6 | Stale 保护：裁决/检查结果绑定正文 hash；正文重写后旧结果 stale=true，阻断 review/approved | `repository/storage-model.md` |
| 7 | approved 是章节内容终态；approved 后不得改正文/裁决/残余/四相日志/thread-binding | `repository/terminology.md` |
| 8 | 卷 sealed→archived 单向冻结；seal 须所有章节 approved+涌现处置+目标达成+监测报告；归档后卷内实体全冻结 | `repository/storage-model.md`/`terminology.md` |
| 9 | 写入流程为 10 步有序事务：校验输入→取写锁→开DB→归档冻结 guard→approved 终态 guard→开事务→算hash/字数→写temp+原子替换→更新SQLite元数据→commit；失败清理 temp | `repository/cli-modules.md`/`storage-model.md` |
| 10 | schemaVersion 单源自 `PRAGMA user_version`；所有 `--json` 含十进制整数字符串 schemaVersion；破坏变更升、兼容增量不升 | `repository/contract-boundary.md` |
| 11 | 错误对象结构化：每错误返 JSON 含 code(snake_case)/message/exitCode(须==进程退出码)/suggestion；exit/code SSOT 在 `exit-codes.md`/`error-codes.md`，文档引用不抄 | `repository/contract-boundary.md` |
| 12 | 分层调用链：commands(解析+enum校验)→services(状态机/guard/事务)→repositories(SQL/文件)→storage(连接/迁移/锁/原子文件)；commands 不得访问 SQLite/文件、不得做业务逻辑；repositories 不得开/提交事务或做 guard 判断 | `repository/cli-modules.md` |
| 13 | 受控 view/scope 名为冻结枚举；非法值 exit 2 | `contracts/cli/views-scopes.md` |
| 14 | 实体 ID 稳定非 UUID：`<prefix>-<number>` 或复合键字符串；sourceRefs 不得含 rowid/内部路径 | `repository/contract-boundary.md` |
| 15 | 契约唯一稳定边界：稳定=命令语法/`--json`结构/错误对象/退出码/ID/view-scope/状态机转移；非稳定=SQLite schema/`.novel`路径/文件命名/Rust 模块/迁移文件名 | `repository/contract-boundary.md` |
| 16 | 文档引用不抄：`docs/`与`CLAUDE.md` 须引用 `contracts/cli/` SSOT，不复制退出码/错误码/view-scope/schemaVersion 表 | `repository/contract-boundary.md`/`openspec/config.yaml` |

---

## 5. 发现目录

严重度：**high** = 阻断状态机/一致性/防绕过；**medium** = 违反分层/契约文档矛盾；**low** = 命名/计数/格式瑕疵。

### 5.1 实现侧发现（15 处）

| 编号 | 描述 | 证据指针 | 触不变量 | 严重度 |
|---|---|---|---|---|
| F-01 | `emergence rate/decide/execute/archive` 实现接通 `--json`，但 `commands.md` 标为 no-JSON（人读）——契约↔实现 `--json` 偏差 | `crates/novel-cli/src/commands/emergence.rs` vs `contracts/cli/commands.md`（emergence rate/decide/execute/archive 条目） | 15 | medium |
| F-02 | 章节正文写入非「10 步事务」：实际 ~5 步，文件写与 SQLite 更新无事务耦合，SQLite 失败无回滚/补偿删除 | `crates/novel-cli/src/repositories/chapter_repository.rs:103-110` | 9 | high |
| F-03 | mainplot/thread/emergence 多处人读成功消息由 command 层 `println!` 直接输出，渲染职责未归 renderer | `crates/novel-cli/src/commands/{mainplot,thread,emergence}.rs`（如 `thread.rs:204`） | 12 | medium |
| F-04 | `export volume/chapter/full` 与 `verify integrity` 的 `--json` 标志事实上 no-op（默认即 JSON，标志被 handler 忽略） | `crates/novel-cli/src/commands/{export,verify}.rs` | 15 | low |
| F-05 | 声称 WAL 但 `open` 仅设 `foreign_keys=ON`，未 `PRAGMA journal_mode=WAL`；并发安全仅靠 flock `.write.lock` | `crates/novel-cli/src/storage/connection.rs`（`open`/docstring） | 4 | medium |
| F-06 | schemaVersion 双写：`PRAGMA user_version` 与 `project_meta.schema_version`（`metadata.rs:20`）均写，读以 user_version 为准——第二源可能漂移 | `crates/novel-cli/src/storage/{connection.rs:86,metadata.rs:20}` | 10 | low |
| F-07 | `verify integrity` hard 违规经 `std::process::exit` 直出，绕过 `lib::run()`→`render_error`，不经结构化错误对象 | `crates/novel-cli/src/commands/verify.rs:69-72` | 11 | high |
| F-08 | 半数命令组（volume/mainplot/thread/emergence/export/verify）在 command 层直接 `connection::open_existing_at`，违反「commands 不访问 storage」 | `crates/novel-cli/src/commands/{volume,mainplot,thread,emergence,export,verify}.rs` | 12 | medium |
| F-09 | `unreachable!()` 用于 thread/emergence/mainplot dispatch catch-all，非 `CliError::usage`（s01 组返 usage）——不变量破裂即 panic | `crates/novel-cli/src/commands/{thread,emergence,mainplot}.rs` | 12 | low |
| F-10 | service 热路径 `expect()`/`unwrap()`：`thread_service.rs:53`、`mainplot_service.rs:40`——不变量破裂即 panic | `crates/novel-cli/src/services/{thread_service.rs:53,mainplot_service.rs:40}` | 12 | low |
| F-11 | 写锁为独立 `.novel/.write.lock` 文件 flock，非 DB 级；两进程各持 flock 关键段时第三进程用裸 SQLite 可绕过 | `crates/novel-cli/src/storage/file_ops.rs`（`ProjectWriteLock`） | 4 | medium |
| F-12 | `volume seal` 落 `sealing` 非 `sealed`，`advance_to_sealed` 注明「不单独暴露为命令」——sealed-but-not-archived 仅经 archive 可达，状态机入口非显性 | `crates/novel-cli/src/services/volume_service.rs:81,96-97` | 8 | medium |
| F-13 | `constraint define` suggestion 文案指向未实现的 `constraint delete` | `crates/novel-cli/src/services/constraint_service.rs:62` | 15 | low |
| F-14 | `info` JSON 用紧凑序列化（`to_string`）而非 sibling 的 `to_string_pretty`，输出风格不一致 | `crates/novel-cli/src/renderers/info.rs` | —(风格) | low |
| F-15 | 多个实体无 delete 路径（setting/constraint/chapter/verdict 无 delete；仅 `volume goal remove` 存在） | `crates/novel-cli/src/commands/{setting,constraint,chapter,verdict}.rs` | —(功能缺失) | low |

### 5.2 契约文档自洽性发现（9 处，不映射实现不变量）

| 编号 | 描述 | 证据指针 | 严重度 |
|---|---|---|---|
| C-01 | 退出码 5 双标签：`exit-codes.md §A` 5=「状态冲突」，`error-codes.md §A` 把 `external_modification`→5 标「外部修改检测」——该类别在 exit-codes 不存在；实际外部修改恒走 21 | `contracts/cli/exit-codes.md §A` vs `contracts/cli/error-codes.md §A` | medium |
| C-02 | README 命令计数 45，实际 `## \`novel…\`` H2 计数 46（off-by-one） | `contracts/cli/README.md:49` vs `contracts/cli/commands.md` | low |
| C-03 | STATUS.md 引用不存在的 `archive_export_e2e.rs`（契约测试目录内）——实位于 `crates/novel-cli/tests/e2e/`，混淆两测试树 | `contracts/cli/contract-tests/STATUS.md`（表格 + run-output 段） | low |
| C-04 | STATUS.md 内部计数矛盾：验收标准写「state_transition.rs 全部 12 个测试」，实际 13 `#[test]`，且 STATUS.md 自身 run-output 写「13 passed」 | `contracts/cli/contract-tests/STATUS.md` | low |
| C-05 | `commands.md`（`export chapter`）退出码表自相矛盾：列退出码 6（场景「章节不存在」），又注「章节使用 `chapter_not_found`，退出码 20」——6 行被自身括注否定 | `contracts/cli/commands.md`（export chapter 退出码表） | medium |
| C-06 | `json-output.md §5.4` 在 `constraint check --list` 段后有散落无配对 ```` ``` ```` 闭合 fence | `contracts/cli/json-output.md §5.4` | low |
| C-07 | STATUS.md 文字「schemaVersion 升至 6」（s01），而 golden 均带 `"schemaVersion":"9"`（当前 `CURRENT_SCHEMA_VERSION`）——文字过时（测试比对常量非硬编码，非契约违反） | `contracts/cli/contract-tests/STATUS.md` vs `contracts/cli/examples/*.golden.json` | low |
| C-08 | `novel init` 无成功 JSON schema——`json-output.md §5.6` 明确排除（Spec 经 `info` 探测非 `init`），仅 error golden 被契约化；需确认实现不让 Spec 依赖 `init --json` 成功形态 | `contracts/cli/json-output.md §5.6` | low |
| C-09 | `commands.md`（`constraint check` 写形态）章节未找到列退出码 6 + `chapter_not_found`，但 `chapter_not_found` 是退出码 20 的规范子码——6 vs 20 跨命令不一致 | `contracts/cli/commands.md`（constraint check 写形态） | medium |

### 5.3 测试侧发现（3 处）

| 编号 | 描述 | 证据指针 | 严重度 |
|---|---|---|---|
| T-01 | `novel mainplot node update` 无契约测试（仅 `mainplot_integration_test.rs` + e2e 覆盖，`mainplot_contract_test.rs` 只含 create/list/get） | `contracts/cli/contract-tests/mainplot_contract_test.rs` | medium |
| T-02 | `novel emergence get` 无契约测试（仅 `emergence_integration_test.rs` 覆盖，`emergence_contract_test.rs` 不含 get） | `contracts/cli/contract-tests/emergence_contract_test.rs` | medium |
| T-03 | `contract_constraint_check.rs` 为契约测试（功能）却归档于 `crates/novel-cli/tests/` 而非 `contracts/cli/contract-tests/`——位置不一致 | `crates/novel-cli/tests/contract_constraint_check.rs` vs `contracts/cli/contract-tests/` | low |

---

## 6. 修复簇路线图

按「同一修复面」聚合（非按命令组）。每簇为**建议**，实际拆分以 `openspec-slices-plan` 评估为准。各簇可独立 apply/verify。

### 簇 1：契约文档自洽修正
- **Scope**：就地修正 `contracts/cli/` 内部 9 处自相矛盾（C-01..C-09），不改任何实际取值（退出码/错误码/view/scope 数值不变）。
- **Non-goals**：不动实现代码；不改 SSOT 取值集合，仅修正文档表述与交叉引用。
- **影响产物**：`exit-codes.md`/`error-codes.md`/`commands.md`/`json-output.md`/`README.md`/`contract-tests/STATUS.md`
- **所含发现**：C-01..C-09
- **预计严重度**：medium（C-01/C-05/C-09）/ low（其余）

### 簇 2：存储一致性
- **Scope**：章节正文写事务原子化（文件写 + SQLite 更新同事务，失败回滚/补偿删 temp）；启用 `PRAGMA journal_mode=WAL`；评估写锁下沉 DB 级或保留 flock 但补裸 SQLite 绕过防护。
- **Non-goals**：不改状态机语义；不改正文存储路径策略。
- **影响产物**：无契约产物变更（实现层 invariant）；可能需补 `content-integrity` spec 的写事务场景。
- **所含发现**：F-02/F-05/F-11
- **预计严重度**：high（F-02）/ medium（F-05/F-11）

### 簇 3：命令分层下沉
- **Scope**：volume/mainplot/thread/emergence/export/verify 的 DB-open 从 command 层下沉至 service；人读成功消息从 command `println!` 收归 renderer；`unreachable!`/`expect`/`unwrap` 替换为 `CliError`。
- **Non-goals**：不改命令语法/JSON 输出/退出码（契约稳定面不变）；不重构 s01 已规范组。
- **影响产物**：无契约产物变更（实现层 invariant）。
- **所含发现**：F-03/F-08/F-09/F-10
- **预计严重度**：medium（F-03/F-08）/ low（F-09/F-10）

### 簇 4：错误渲染路径统一
- **Scope**：`verify integrity` hard 违规经 `lib::run()`→`render_error` 输出结构化错误对象，而非 `std::process::exit` 直出报告 JSON。
- **Non-goals**：不改 verify 的退出码语义（仍 21/11/70/5）；不改 verify 成功路径。
- **影响产物**：可能需明确 `verify-integrity.schema.json` 错误对象承载方式。
- **所含发现**：F-07
- **预计严重度**：high

### 簇 5：契约测试补齐
- **Scope**：补 `mainplot node update` 与 `emergence get` 的契约测试（JSON schema/退出码/error 对象）；将 `contract_constraint_check.rs` 迁归 `contracts/cli/contract-tests/`。
- **Non-goals**：不改实现；不改契约 SSOT 取值。
- **影响产物**：`contracts/cli/contract-tests/`（新增 + 迁移）+ `crates/novel-cli/Cargo.toml`（test 注册）
- **所含发现**：T-01/T-02/T-03
- **预计严重度**：medium（T-01/T-02）/ low（T-03）

### 簇 6：`--json` 偏差与 no-op 清理
- **Scope**：统一 `emergence rate/decide/execute/archive` 与 `commands.md` 的 `--json` 约定（择一：或补契约记 `--json`，或移除实现的 `--json`）；清理 `export`/`verify` 的 `--json` no-op（默认 JSON 时移除冗余标志或让其有语义）。
- **Non-goals**：不改任何命令的成功 JSON schema。
- **影响产物**：`commands.md`（emergence/export/verify 条目）+ 可能 `json-schemas/`（若 emergence 补 JSON）
- **所含发现**：F-01/F-04
- **预计严重度**：medium（F-01）/ low（F-04）

### 簇 7：JSON 输出统一
- **Scope**：统一 `info` 等 renderer 的 JSON 序列化为 `to_string_pretty`（或全紧凑，择一一致）。
- **Non-goals**：不改 JSON schema 字段。
- **影响产物**：无契约产物变更（实现层风格）。
- **所含发现**：F-14
- **预计严重度**：low

### 簇 8：状态机入口显性化
- **Scope**：评估 `volume seal`（落 `sealing`）与 `advance_to_sealed`（仅经 archive 可达 `sealed`）是否需独立命令或文档显性化，避免 sealed-but-not-archived 状态不可显式触达。
- **Non-goals**：不改卷状态机状态集合（active/sealing/sealed/archived）；不改归档语义。
- **影响产物**：可能 `commands.md`（volume seal/archive 条目语义）+ `state-machine.md`
- **所含发现**：F-12
- **预计严重度**：medium

---

## 7. 未来倡议（未实现，不计入修复簇）

以下为三基线显式暴露的**完全未实现**项，属增量建设而非对既有实现的修复，须另行经 `openspec-slices-plan` 全新拆分，不混入任何修复簇：

| 倡议 | 状态 | 来源 |
|---|---|---|
| 实体 delete 系列（`setting delete`/`constraint delete`/`chapter delete`/`verdict delete`） | 未实现；`constraint_service.rs:62` suggestion 已指向 | F-15 / `repository/storage-model.md` |
| 章节四相日志（设定吸收/主线推进/涌现响应/节奏控制） | 未实现；`terminology.md` 列为 review 前置，无命令/表/spec | `repository/terminology.md` |
| 残余（residue）命令 | 未实现；`init` 已建 `content/residues/` 目录，无 `novel chapter residue` 命令/spec | `repository/storage-model.md` |
| 章节契约表（must/may_deviate/encourage_surprise/场景配置/预期出口） | 未实现；`chapter-context-packet` 明确「留后续 change 补表后填」 | `openspec/changes/chapter-context-packet` proposal |
| 监测报告命令（读/写监测报告） | 部分；`volume seal`(s07) 写报告但无独立读写命令，`reports/` 由 init 建 | `repository/storage-model.md` |
| Spec 运行层（`spec/runtime`/`prompts`/`rules`/`workflows`/`fixtures`） | 全规划未建 | `repository/architecture.md` |

---

## 8. 验证自检

- [x] 8.1 矩阵覆盖全部已实现命令：`crates/novel-cli/src/commands/mod.rs` 注册 12 组，§2 命令清单与 §3 矩阵逐组覆盖，无遗漏。
- [x] 8.2 抽查 5 条 ⚠️/❌ 证据指针（均在写作时实读源码确认）：
  1. F-02 → `chapter_repository.rs:103-110`（已读，确认无回滚 + 自承注释）
  2. F-07 → `commands/verify.rs:69-72`（已读，确认 `std::process::exit`）
  3. F-08 → `commands/volume.rs` 等（契约 Explore 记录 command 层 `open_existing_at`，`verify.rs:55` 印证）
  4. F-12 → `services/volume_service.rs:81,96-97`（契约/实现 Explore 记录）
  5. C-05 → `commands.md`（export chapter 退出码表，契约 Explore 记录）
- [x] 8.3 「引用而非自抄」自检：本报告未复制任何 `contracts/cli/` SSOT 的退出码/错误码/view-scope/schemaVersion 取值表；§1 仅描述结构与模板，§4 仅列不变量索引（语义引用 `repository/` 文件），数值判定均引用具体文件名 + 行。
- [x] 8.4 `cargo test -p novel-cli` 回归：本 change 未引入代码改动，预期全绿——已执行。结果：全部测试二进制 0 failed，进程退出码 0（回归保护通过）。