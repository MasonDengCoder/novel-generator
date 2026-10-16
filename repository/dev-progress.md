# 开发进度还原

> 本文件依据 git 提交历史、OpenSpec 归档切片、契约测试状态与工作树当前未提交变更**还原**项目的开发进度，供跨 session 追踪与接续开发使用。
> 数据采集时点：2026-09-29；当前分支：`master`；当前 HEAD：本文所在基线提交。
> 维护规则：本文件记录"当前状态"与"已落地切片"，同时承载 CLI 实现进度对照（命令矩阵 + 未实现实体清单），供 Spec 运行层（spac）搭建时对照。不承载当日 TODO 或一次性业务需求（后者见 `CLAUDE.md` §11 职责边界）。每次切片归档或契约修复推进后更新本文件。

## 0. 进度判定方法论

本文件的数据来源 = **从系统设计反推**，非手记时间线。设计文件只描述系统最终形态（不含进度），本文件是进度的唯一载体，通过以下四步反推填充：

1. **系统设计含哪些模块** → 见 `docs/` 三主架构（项目整体架构/CLI架构设计/Spec架构设计）+ `repository/`（architecture/cli-modules/terminology/storage-model）。
2. **每模块含哪些应实现内容** → 见各设计文件的实体清单、命令族、状态机状态集合。对照 §7.0 设计覆盖面反推表。
3. **这些内容做好了吗** → **三否证判据**：有 `contracts/cli/commands.md` 条目？有 `crates/novel-cli/src/` 代码？有 `openspec/specs/**/spec.md` 的 Given/When/Then？三者皆无 = 规划态；有则按维度记 ✅/⚠️/❌（见 §4 矩阵）。
4. **做好了放在哪** → 记录 commands.md 行、切片名、源文件路径，形成本文件的 §3 切片时间线、§4 命令矩阵、§7.2 未实现实体清单。

> 反推的客观证据源：git 提交历史（§2）、OpenSpec `changes/archive/`（§3）、`contracts/cli/contract-tests/STATUS.md`（§4）。进度变更只能从这三源反推，不从设计文件散落的状态词采集（设计文件不含进度内容，见 CLAUDE.md §15）。

## 1. 一句话定位

`novel-generator` 的 **CLI 硬层**已实现全部 7 个垂直业务切片 + 2 个受控渲染切片，并在此基础上完成一轮"三基线对齐"工程（cli-contract-alignment-audit 报告 + s01–s08 治理切片）、"契约治理"批次（contract-governance-01~05，契约源从手写 md 迁到类型化代码三源生成）与"V2 错误协议"批次（contract-registry-v2-01~05），共 45 个 OpenSpec 切片归档。新增**生成式契约测试**能力（`crates/spec-gen` xtask 从 `openspec/specs/**/spec.md` 的 `@codegen` 块生成 `contracts/cli/generated/*.gen.rs`，构造性消除 spec↔test 漂移），`codegen-coverage` 批次（01~06）已把覆盖扩到**全部 16 个命令 spec（16/16，300 个 @codegen 块，16 份 .gen.rs）**。当前 CLI 主体测试已通过卷目标修复后的定向验证，完整测试结果以本次工作树验证为准。Spec 运行层已建立平台中立单章 Skill、Claude Code/Codex 薄适配、双端清单安装器和步骤 0–9 的可验证写验修复闭环；步骤 10–15 仍在规划中。

## 2. 当前分支与工作树状态

| 项 | 值 |
|---|---|
| 当前分支 | `master` |
| HEAD | 本文所在基线提交（`feat: align CLI contracts and bootstrap spec runtime`） |
| 工作树未提交变更 | 有：`agent-novel-workflow-recovery` 批次正在实施，见 §3.1 |
| `spec/` 目录 | 已建立公共单章 Skill、Claude Code/Codex 薄适配、双端清单安装器与成功/repair parity E2E；切片 01、02 已归档 |

## 3. 已落地切片时间线（OpenSpec 归档）

依 `openspec/changes/archive/`（共 45 个归档切片）。治理类（align-*）切片为契约 SSOT 收口，业务切片（s01–s07）为垂直能力，末两片为受控渲染补齐，契约治理批次（contract-governance-01~05）为契约源迁类型化代码三源生成，V2 错误协议批次（contract-registry-v2-01~05 已归档）为完整契约生成与默认化治理，@codegen 覆盖批次（codegen-coverage-01~06 已归档）为 spec scenario 级生成式契约测试扩面，批次完成后覆盖面达 16/16 命令 spec。

| 顺序 | 归档日期 | 切片 | 类型 | 落地命令组 / 产出 |
|---|---|---|---|---|
| 0 | 2026-06-25 | `cli-spine-init-info` | 业务·脊柱 | `init` / `info` |
| — | 2026-06-25 | `align-exit-codes-ssot` | 治理 | exit-codes SSOT 收口 |
| — | 2026-06-26 | `align-error-codes-ssot` | 治理 | error-codes SSOT 收口 |
| — | 2026-06-26 | `align-schemaversion-evolve` | 治理 | schemaVersion 演进策略 |
| — | 2026-06-26 | `align-sourcerefs-ssot` | 治理 | sourceRefs / 实体 ID SSOT |
| — | 2026-06-26 | `align-views-scopes-ssot` | 治理 | 受控 view/scope SSOT |
| 1 | 2026-06-29 | `s01-project-bootstrap` | 业务 | `setting`（set/get/list）、`constraint`（define/render/list） |
| 2 | 2026-06-29 | `s02-chapter-content-core` | 业务 | `chapter`（create/update/get/list/reimport）+ 内容 hash 一致性 |
| 3 | 2026-06-30 | `s03-rule-checking-engine` | 业务 | `constraint check`（规则检查绑定 hash） |
| 5 | 2026-06-30 | `s05-volume-management` | 业务 | `volume`（create/info/list/goal）最小切片 |
| 4 | 2026-07-03 | `s04-state-machine-and-verdicts` | 业务 | `chapter status`（状态机门禁）、`verdict set-mainplot-landing` |
| 6 | 2026-07-15 | `s06-destiny-weaving` | 业务 | `mainplot`、`thread`、`emergence`（命运编织三件套） |
| 7 | 2026-07-20 | `s07-archive-and-export` | 业务 | `volume seal/archive`、`export`、`verify` |
| + | 2026-07-22 | `setting-constraint-render` | 渲染补齐 | `setting render --view`、`constraint render --view/--scope` 受控形态 |
| + | 2026-07-22 | `chapter-context-packet` | 渲染补齐 | `chapter context --view`（P0/P1/audit/overflow 只读包） |
| — | 2026-07-23 | 契约修复批次 A–E | 治理 | 文档对齐 + schemaVersion 假绿清理 → severity 三档化(8→9) → check-write 契约 → fixture worlds |
| — | 2026-07-24 | `cli-contract-alignment-audit` | 治理·审计 | 三基线审计报告（契约矛盾 + 实现缺口 + 测试缺失，本切片不执行代码修复） |
| — | 2026-07-24 | `s01-contract-doc-self-consistency` | 治理 | 契约文档自洽 |
| — | 2026-07-24 | `s02-storage-consistency` | 治理 | 存储一致性修复 |
| — | 2026-07-27 | `s03-command-layer-sinking` | 治理 | 六命令组 DB-open 下沉 service 层、人读消息收归 renderer、热路径 panic 改 `CliError` |
| — | 2026-07-27 | `s04-error-render-unification` | 治理 | `verify integrity` hard 违规接入结构化错误对象 |
| — | 2026-07-27 | `s05-contract-test-coverage` | 治理 | 契约测试覆盖补全 |
| — | 2026-07-28 | `s06-json-flag-reconciliation` | 治理 | emergence/export/verify `--json` 对齐 |
| — | 2026-07-30 | `chapter-create-volume` | 业务补丁 | `chapter create --volume` |
| — | 2026-07-30 | `s07-json-serialization-style` | 治理 | JSON 序列化风格统一 |
| — | 2026-07-30 | `s08-statemachine-entry-explicit` | 治理 | 卷状态机入口显性化 |
| — | 2026-08-12 | `contract-governance-01-ssot-drift-fix` | 治理·契约治理批次 | 修复 `external_modification` 退出码 5 vs 21 SSOT 漂移：确立 `errors/mod.rs::exit_code()` 为错误码→退出码唯一权威映射，契约文档与行为规范显式区分通用码 `external_modification`→5 与细分码 `content_modified_externally`→21（本切片为 `contract-governance` 批次 01-05 的地基止血切片，根因治理 derive 生成交 02-05） |
| — | 2026-08-13 | `contract-governance-02-error-enum-gen` | 治理·契约治理批次 | `ErrorCode`/`ExitCode` enum 带 `strum::EnumIter` + `serialize`，错误码字符串/退出码数值确立类型化生成源（`as_str()` 委托 strum，退出码数值唯一权威为 enum 判别式） |
| — | 2026-08-24 | `contract-governance-03-struct-schema-gen` | 治理·契约治理批次 | renderers struct 带 `schemars::JsonSchema`，`json-schemas/*.json` 确立 struct 类型化生成源（`$schema` 统一 draft-07） |
| — | 2026-09-09 | `contract-governance-04-specgen-rebuild` | 治理·契约治理批次 | `spec-gen` 读三源（strum enum + schemars struct + clap 命令树）生成 `contracts/cli/*.md`，`check-generated-drift.sh` 扩展为全量漂移门禁 |
| — | 2026-09-09 | `contract-governance-05-docs-flow-update` | 治理·契约治理批次 | 收尾（本切片）：`CLAUDE.md` §13/§15 契约先行与归档门禁措辞 + `dev-progress.md` 进度 + 迁移文档状态落回，纯文档 |
| — | 2026-09-14 | `contract-registry-v2-01-error-protocol-foundation` | 治理·V2 错误协议批次（已归档） | V2 错误协议地基：`errors/catalog.rs` 定义 `ErrorCategory`（7 类固定退出码 `exit_code()` 纯函数）、类型化目录 `ErrorCatalog`（种子集 + `get/category/exit_code`）、`V2ErrorEnvelope`（版本化 JSON 信封，exitCode 由类别推导）、`ErrorProtocolVersion`（缺省 V1）。本切片只建类型地基，不接线任何命令 |
| — | 2026-09-14 | `contract-registry-v2-02-error-catalog-migration` | 治理·V2 错误协议批次（已归档） | 目录化迁移：`ErrorCatalog::ENTRIES` 扩充为覆盖全部公开错误 ID 的真目录（33 项，类别归属按既有 V1 语义归类），新增 `ErrorCode::category()`（经目录解析唯一类别绑定）、`CliError::error_id()`、`ErrorCatalog::retryable()`、`V2ErrorEnvelope::from_catalog()`（详情字段集经 `DetailSpec.fields` 校验、恢复元数据取自目录）；error renderer 支持 `ErrorProtocolVersion` 选择（缺省 V1 保持逐字不变，显式 V2 输出信封+类别退出码）。退出码收敛为类别码的契约破坏性切换按 design D4/D5 归 05 治理（本切片保持 V1 per-entity 码不变） |
| — | 2026-09-15 | `contract-registry-v2-03-command-contract-registry` | 治理·V2 错误协议批次（已归档） | 命令契约注册表：`contracts/mod.rs` 定义 `CommandContract`/`InputContract`/`OutputContract`/`GuardRef` 类型与静态注册表（46 个 clap 叶子命令各一条，含行为摘要、机器可验证输入输出、可返回错误 ID、guard 引用），单一校验核心 `validate_registry()`（clap 叶子全覆盖 / 错误 ID ∈ ErrorCatalog / guard 可解析 / 目录公开错误反向追踪），`spec-gen` generate/check 接入复用（不一致即 drift 红 exit 1）。语义只引用 clap（语法唯一源）与 ErrorCatalog（错误唯一源），不复制语法/错误映射 |
| — | 2026-09-16 | `contract-registry-v2-04-full-contract-generation` | 治理·V2 错误协议批次（已归档） | 完整契约生成：`spec-gen` 新增 `registry_source.rs`，以 `CommandContract` 注册表 + `ErrorCatalog` + clap 树为命令语义唯一输入，生成 `commands.md` 行为语义段/退出码表（替代手写段）、`command-error-matrix.md`（命令 × 错误 ID × 类别 × 退出码矩阵）、`examples/INDEX.md` / `contract-tests/INDEX.md`（示例/fixture 与契约测试索引，文件系统清单 + 注册表输出定义驱动）；`check` 扩展为六类受控产物全量 drift 门禁 + `check_index_consistency()` 反向一致性校验 + 叙述映射键集合校准。见 `docs/契约治理-生成式契约源迁移.md` |
| — | 2026-09-17 | `contract-registry-v2-05-contract-v2-rollout-governance` | 治理·V2 错误协议批次（已归档） | 治理收口与默认化：`ErrorProtocolVersion::default()` 翻转 V2（缺省 `V2ErrorEnvelope`，进程退出码与信封 `exitCode` 由 `ErrorCatalog` 类别派生），V1 `ErrorObject` 转显式选择的弃用兼容路径（逐字保持 V1 回归基线）；固化 V1 兼容期/弃用/移除三条件齐备准则；CI 接入生成物漂移/手写生成物/绕过注册表三类门禁（复用 `spec-gen check` 单一校验核心）；`AGENTS.md`/`repository/`/`docs/`/本文件同步为引用 SSOT。见 `docs/契约治理-生成式契约源迁移.md` §6.5 |
| — | 2026-09-10 | `codegen-coverage-01-drift-fix-dsl` | 治理·@codegen 覆盖批次（已归档） | @codegen 断言 DSL 扩展（`json_array_len`/`json_num_field`/`every_<item>_has` 等）+ 散文错误码止血；为后续 spec scenario 级生成式测试扩面铺路（本切片自身不改覆盖面计数） |
| — | 2026-09-17 | `codegen-coverage-02-setting-constraint` | 治理·@codegen 覆盖批次（已归档） | setting-commands（25 块）/constraint-commands（33 块）/rule-checking（9 块 + 4 类跳过注记）spec 全量补 `@codegen` 生成式契约测试（`contracts/cli/generated/` 3 个新 .gen.rs，共 67 测试）；新增 3 个自包含 fixture（setup-settings/setup-constraints/setup-rule-checks）；散文漂移就地修正（severity 三档、`setting set ""` 实为 usage、V2 类别派生退出码核对——`chapter_not_found` 为 6 非 20，原「6→20」任务按 V2 现实修正 artifact） |
| — | 2026-09-17 | `codegen-coverage-03-volume` | 治理·@codegen 覆盖批次（已归档） | volume-lifecycle（16/20 块）/volume-sealing（16/23 块）/volume-goals（17/20 块）spec 补 `@codegen` 生成式契约测试（`contracts/cli/generated/` 3 个新 .gen.rs，共 49 测试）；新增 9 个自包含 fixture（setup-initialized-empty / setup-volume-unapproved-chapters / setup-volume-pending-emergence / setup-volume-executed-emergence / setup-volume-unachieved-goal / setup-volume-achieved-goal / setup-volume-optional-goal / setup-volume-seal-blocked / setup-volume-archived）；实证修正 proposal 假设（卷不存在 errorId 为 `not_found` 非 `volume_not_found`、`goal set` 非法节点 ID 为 `usage` 非 `invalid_input`、布尔字段 `canSeal`/`isMandatory` 无 DSL 断言 key 改为经封卷退出码行为等价断言）；散文漂移不就地修正（退出码 5→6 等留待独立治理） |
| — | 2026-09-22 | `codegen-coverage-04-chapter-state` | 治理·@codegen 覆盖批次（已归档） | chapter-state-machine（13/15 块）/state-transition-guards（15/17 块）/content-integrity（10/18 块）/chapter-lifecycle（6→22 块，达全覆盖）spec 补 `@codegen` 生成式契约测试（3 个新 .gen.rs 共 38 测试 + 重生成 chapter-lifecycle 22 测试）；新增 10 个自包含门禁态 fixture（setup-chapter-drafting / -ready-for-review / -at-review / -hard-violation / -no-verdict / -stale / -missing-content / -review-modified / -review-hard-violation / -review-verdict-unachieved）+ 二进制资产 `assets/non-utf8.bin`；门禁态全经 CLI 命令序列构造（无 SQL 直插，仅复用既有 setup-approved-chapter）；实证发现实现差距（`chapter update` 无「更新前正文 hash 失配」预检，实测外部修改态返回 0 并重算 hash，与 spec SHALL 冲突，故不建块并上报）；散文漂移就地修正（`E_PROJECT_NOT_INITIALIZED` 退出码 10 → 3/`world_not_initialized`、`E_INVALID_ARGUMENT`/`E_CHAPTER_NOT_FOUND`/`E_FILE_NOT_FOUND` → `usage`/`chapter_not_found`/`file_not_found`） |
| — | 2026-09-22 | `codegen-coverage-05-destiny-weaving` | 治理·@codegen 覆盖批次（已归档） | mainplot-node-management（23/23 块）/thread-management（22/28 块）/emergence-management（46/46 块）/verdict-management（11/16 块）spec 补 `@codegen` 生成式契约测试（4 个新 .gen.rs 共 102 测试）；新增 4 个自包含 fixture（setup-mainplot-world / setup-thread-world / setup-emergence-world / setup-emergence-states，涌现五态 7 条预驱动）；**proposal 错误码表整体失效**（design D6）：V2 下退出码为 `ErrorCatalog` 类别派生而非 s06 专属 41–47，实测 `invalid_state_transition`→3、`emergence_not_found`/`chapter_not_found`/`volume_not_found`→6、`emergence_archived`→3、`thread_name_conflict`/`binding_already_exists`→4、参数非法为 `usage`、外部修改 `content_modified_externally`→5，全部断言与 verdict 散文按实测纠错；`ERR_*` 前缀散文对齐 snake_case（mainplot/thread 全量 + emergence 20 处）；跳过 19 scenario 就地注记（引用完整性 7 / `--help` 3 / 事务失败 1 / 文档 meta 4 / 纯人读弱断言 4）；发现 DSL 语义陷阱并规避（`every_<item>_has` 实为「逐项非空数组」非「字段存在」、`json_field`/`json_array_len`/`json_num_field` 只取 mapping 首项、CJK flag 值须引号、块终止符 `-->` 写错会被解析器越块吞掉） |
| — | 2026-09-22 | `codegen-coverage-06-project-lifecycle` | 治理·@codegen 覆盖批次（已归档，批次收尾） | project-lifecycle（7/11 块）spec 补 `@codegen` 生成式契约测试（新增 project-lifecycle.gen.rs 7 测试）；新增 1 个纯 init fixture（setup-initialized-world，填补 init/info 生成测试语义空白）；4 个 `--path`/cwd 场景因 harness 无条件追加 `--path <tempdir>` 无法生成表达，就地注记跳过并保留手写覆盖（`init_args.rs`/`init_ok.rs`/`info_blocked.rs`）；`info --json` 的 `counts.*` 全零消费切片 01 的 `json_num_field`；本切片收尾同步本文件 §3/§4/§5 覆盖面数字 |
| 00 | 2026-09-29 | `agent-novel-workflow-recovery-00-chapter-write-json-contract` | Agent 工作流恢复·CLI 前置契约 | 为 `chapter update` 增加稳定 `--json` 回执并补齐 chapter update/status schema 与 golden；审计并强化 constraint check/verdict/chapter status 的成功 JSON 与 V2 失败契约；全量测试与生成物漂移检查通过 |
| 00b | 2026-09-29 | `agent-novel-workflow-recovery-00b-rule-check-supersession` | Agent 工作流恢复·CLI 前置语义 | `constraint check --input` 成功提交后原子替换同章旧检查集；正文修复后重检前继续 stale 阻断，成功重检后 `--verify-hash` 与章节状态推进可恢复；不改命令、JSON、错误协议或 SQLite schema |
| 01 | 2026-09-29 | `agent-novel-workflow-recovery-01-claude-chapter-loop` | Agent 工作流恢复·Claude 单章闭环 | 建立公共单章 Skill、Claude Code 薄适配与显式清单安装器；安装/发现契约和成功/repair E2E 通过，所有权威状态访问保持 CLI-only |
| 02 | 2026-10-09 | `agent-novel-workflow-recovery-02-codex-chapter-loop` | Agent 工作流恢复·Codex 单章闭环 | 新增 Codex `.agents/skills` 薄适配、双端安装/发现契约、能力一致性矩阵和共享行为 parity；安装、静态检查、成功/repair/失败停止 E2E、`cargo test -p novel-cli`、生成物漂移和 strict validation 均通过 |

> **@codegen 覆盖批次**（`codegen-coverage-01` 至 `06`）：把 `@codegen` 覆盖从 2/16 扩到 **16/16 命令 spec（100%）**，300 个 `@codegen` 块、16 份 `.gen.rs`、17 个自包含 fixture。批次内切片可并行（各覆盖不同 spec 目录，无 `[[test]]` 名重叠），01 为地基（DSL 扩展 `json_array_len`/`json_num_field`），06 为收尾落回文档。共同实证发现：V2 错误协议下退出码是 `ErrorCatalog` 类别的纯函数，proposal 中沿用的 V1 细分退出码（41–47、10、20、21）一律不成立，断言须按实测与 `error-codes.md` §A 纠错；布尔/嵌套字段断言受 DSL 限制，改经行为等价断言表达。

> 切片编号 s01–s07 非严格顺序推进：s05 早于 s04 落地，s03/s04/s05 曾并行 worktree 开发。视图相关三组承载命令（views-scopes.md §A 组 1/2/3/4）已全部升格为"已实现可调用"。
> **契约治理批次**（`contract-governance-01` 至 `05`）：契约源从手写 `contracts/cli/*.md` 迁到类型化代码生成（`schemars 0.8` + `strum` + `clap` 三源），权衡见 `docs/契约治理-生成式契约源迁移.md`。01 为地基止血切片，02-04 已落地三源生成链路（enum/struct/clap → `contracts/cli/*.md` 机器列/语法段/`json-schemas`），05 收尾落回文档。

### 3.1 当前活动批次

`agent-novel-workflow-recovery` 已登记为顺序切片计划。前置 Change `00`、`00b`、Claude 单章闭环 `01` 与 Codex 单章闭环 `02` 已 verify 并归档；切片 03 已具备开始条件，切片 04 仍只完成登记，不得提前实施。

## 4. 已落地 CLI 命令组（实现矩阵）

对照 `contracts/cli/commands.md`（46 条命令）与契约测试（`contracts/cli/contract-tests/STATUS.md` 全部 ✅ 通过）。命令矩阵供 Spec 运行层（spac）搭建时对照"哪些命令可依赖"。每维：✅ 有/⚠️ 部分缺/❌ 缺失，附证据指针。

| 命令组 | 子命令 | commands.md | 代码实现 | JSON schema | golden | 契约测试 |
|---|---|---|---|---|---|---|
| init | init | ✅ | ✅ `commands/init.rs` | ❌（设计如此，见 json-output §5.6） | ⚠️ 仅 error golden | ✅ `e2e.rs` |
| info | info | ✅ | ✅ `commands/info.rs` | ✅ `info.schema.json` | ✅ `info.golden.json` | ✅ `e2e.rs` |
| setting | set | ✅ | ✅ `commands/setting.rs` | ❌（静默成功，设计如此） | ❌（静默） | ✅ `setting_commands.rs` + 生成式 `setting-commands.gen.rs` |
| | get | ✅ | ✅ | ✅ `setting-get.schema.json` | ✅ `setting-get.output.golden.json` | ✅ |
| | list | ✅ | ✅ | ✅ `setting-list.schema.json` | ✅ `setting-list.output.golden.json` | ✅ |
| | render --view | ✅ | ✅ | ✅ `setting-render.schema.json` | ⚠️ 仅 full/llm（5 视图缺 golden） | ✅ `setting_commands.rs` + 生成式（7 视图遍历） |
| constraint | define | ✅ | ✅ `commands/constraint.rs` | ❌（静默成功） | ⚠️ 仅 error golden | ✅ `constraint_commands.rs` + 生成式 `constraint-commands.gen.rs` |
| | render（双形态） | ✅ | ✅ | ✅ 2 schema | ⚠️ render 单形态 + view-chapter | ✅ + 生成式（15 批量渲染场景） |
| | list | ✅ | ✅ | ✅ `constraint-list.schema.json` | ✅ `constraint-list.output.golden.json` | ✅ + 生成式（过滤/空列表） |
| | check | ✅ | ✅ | ✅ 2 schema（write/list） | ⚠️ 仅 write-ok | ✅ `contract_constraint_check.rs` + 生成式 `rule-checking.gen.rs`（读侧 9 测试；写侧拒绝留手写，DSL 缺 `--input` 路径占位） |
| chapter | create | ✅ | ✅ `commands/chapter.rs` | ✅ `chapter.schema.json` | ✅ 2 golden（含 --volume） | ✅ `e2e.rs`+生成式 |
| | update | ✅ `--json` | ✅ | ✅ chapter.schema `chapterUpdate` | ✅ `chapter-update-ok.golden.json` | ✅ `contract_chapter_update.rs`（成功/V2/终态/归档冻结） |
| | get | ✅ | ✅ | ✅ chapter.schema | ✅ `chapter-get-ok.json` | ✅ `e2e.rs` |
| | list | ✅ | ✅ | ✅ chapter.schema | ✅ `chapter-list-ok.json` | ✅ |
| | reimport | ✅ | ✅ | ❌（静默） | ❌（静默） | ⚠️ 间接 |
| | status | ✅ | ✅ | ✅ chapter.schema `chapterStatus` | ✅ `chapter-status-ok.golden.json` | ✅ `state_transition.rs` 13测试 |
| | context --view | ✅ | ✅ | ✅ `chapter-context.schema.json` | ✅ `chapter-context-writer.golden.json`+2 error | ✅ `chapter_context_commands.rs`+生成式 |
| verdict | set-mainplot-landing | ✅ | ✅ `commands/verdict.rs` | ✅ `verdict.schema.json` | ✅ `verdict-set-ok.json` | ✅ `verdict.rs` |
| volume | create | ✅ | ✅ `commands/volume.rs` | ❌（人读成功） | ❌ | ✅ `volume_commands.rs` + 生成式 `volume-lifecycle.gen.rs` |
| | info | ✅ | ✅ | ✅ `volume-info.schema.json` | ❌（无 golden） | ✅ + 生成式 `volume-lifecycle.gen.rs` |
| | list | ✅ | ✅ | ✅ `volume-list.schema.json` | ❌ | ✅ + 生成式 `volume-lifecycle.gen.rs` |
| | seal | ✅ | ✅ | ✅ `volume-seal-check.schema.json` | ❌ | ✅ + 生成式 `volume-lifecycle.gen.rs` + `volume-sealing.gen.rs`（16 门禁/干运行场景） |
| | archive | ✅ | ✅ | ❌（人读成功） | ❌ | ✅ + 生成式 `volume-lifecycle.gen.rs` |
| | goal set | ✅ | ✅ | ❌（人读成功） | ❌ | ✅ + 生成式 `volume-goals.gen.rs` |
| | goal get | ✅ | ✅ | ✅ `volume-goals.schema.json` | ❌ | ✅ + 生成式 `volume-goals.gen.rs` |
| | goal remove | ✅ | ✅ | ❌（人读成功） | ❌ | ✅ + 生成式 `volume-goals.gen.rs` |
| mainplot | node create | ✅ | ✅ `commands/mainplot.rs` | ✅ `mainplot-node.schema.json` | ⚠️ 仅 .txt transcript | ✅ `mainplot_contract_test.rs` |
| | node list | ✅ | ✅ | ✅ `mainplot-node-list.schema.json` | ⚠️ .txt | ✅ |
| | node get | ✅ | ✅ | ✅ | ⚠️ .txt | ✅ |
| | node update | ✅ | ✅ | ❌（人读成功） | ❌ | ✅ |
| thread | create | ✅ | ✅ `commands/thread.rs` | ✅ `thread.schema.json` | ⚠️ .txt | ✅ `thread_contract_test.rs` |
| | bind | ✅ | ✅ | ❌（人读成功） | ⚠️ .txt | ✅ |
| | list | ✅ | ✅ | ✅ `thread-list.schema.json` | ❌ | ✅ |
| | query | ✅ | ✅ | ✅ `thread-binding-list.schema.json` | ⚠️ .txt | ✅ |
| | update | ✅ | ✅ | ❌（人读成功） | ❌ | ✅ |
| emergence | create | ✅ | ✅ `commands/emergence.rs` | ✅ `emergence.schema.json` | ⚠️ .txt | ✅ `emergence_contract_test.rs` |
| | rate | ✅ | ✅ | ✅ emergence.schema | ⚠️ .txt | ✅ |
| | decide | ✅ | ✅ | ✅ | ⚠️ .txt | ✅ |
| | execute | ✅ | ✅ | ✅ | ⚠️ .txt | ✅ |
| | archive | ✅ | ✅ | ✅ | ⚠️ .txt | ✅ |
| | list | ✅ | ✅ | ✅ `emergence-list.schema.json` | ❌ | ✅ |
| | get | ✅ | ✅ | ✅ emergence.schema | ❌ | ✅ |
| export | volume | ✅ | ✅ `commands/export.rs` | ✅ `export-volume.schema.json` | ✅ `export-volume.golden.json` | ✅ `export_commands.rs` |
| | chapter | ✅ | ✅ | ✅ `export-chapter.schema.json` | ✅ `export-chapter.golden.json` | ✅ |
| | full | ✅ | ✅ | ✅ `export-full.schema.json` | ✅ `export-full.golden.json` | ✅ |
| verify | integrity | ✅ | ✅ `commands/verify.rs` | ✅ `verify-integrity.schema.json` | ✅ ok+error golden | ✅ `verify_commands.rs` |

> 各命令组测试数详见 `cargo test -p novel-cli` 实跑结果，全量 0 failed。`.txt` lifecycle transcript 非 strict golden（mix shell+JSON），属 schema 覆盖；strict `.json` golden 见 `contracts/cli/examples/`。
> 契约层维护方式已从手写改为生成（`contract-governance-02/03/04` + V2 错误协议批次 `contract-registry-v2-03/04/05`）：`commands.md` 生成段（语法段/参数表由 clap 命令树生成，行为语义段/退出码表由命令契约注册表 + `ErrorCatalog` 生成）、`json-schemas/*.json` 由 struct 生成、`exit-codes.md`/`error-codes.md` 机器列由 enum 生成，另有 `command-error-matrix.md`（命令错误矩阵）与 `examples/INDEX.md`、`contract-tests/INDEX.md`（示例/fixture 与契约测试索引）由注册表 + 文件系统清单生成；手改生成物即 drift 红（`scripts/check-generated-drift.sh` 覆盖六类受控产物）。CLI 默认错误协议为 V2（05 默认化），命令失败 `--json` 输出 `V2ErrorEnvelope`、进程退出码为类别派生码；V1 per-entity 退出码仅在显式选 V1 时生效。

### 4.1 三组承载命令验证

`views-scopes.md §A` 冻结的三组承载命令均已实现可调用并有专门契约测试：

| 承载命令 | 受控名 | 实现切片 | golden | 契约测试 |
|---|---|---|---|---|
| `setting render --view` | 7 视图（intro/four-poles/world/style/summary/full/llm） | setting-constraint-render 2026-07-21 | ✅ full+llm+invalid-view | `setting_commands.rs`（遍历 7 视图测试） |
| `constraint render --view/--scope` | 2 view × 3 scope | setting-constraint-render 2026-07-21 | ✅ view-chapter+invalid | `constraint_commands.rs`（16 个 --view/--scope 测试） |
| `chapter context --view` | 3 视图（writer/verifier/repair） | chapter-context-packet 2026-07-22 | ✅ writer+budget-exceeded+invalid-view | `chapter_context_commands.rs` + 生成式 `chapter-context.gen.rs`（双重覆盖） |

### 4.2 chapter-context P0/P1 数据支撑

`chapter context` 的 P0/P1 子集**有数据支撑**（非 stub）：

- **P0**（不可压缩，4 fragment）：hard constraints、verdicts、rule checks、chapter state/hash——全部读真实 SQLite/文件，有真实 stale 检测。
- **P1**（不可压缩，4 fragment）：thread bindings、mainplot nodes、volume goals、previous residue——全部读真实数据。
- **诚实省略**（3 层）：`chapter-contract`、`character-snapshot`、`subplot-situation` 输出 `overflow.omitted`，reason=`no_data_backing_table`（对应实体表未实现，非静默 stub）。

## 5. 契约层与 schemaVersion 演进

- 四块 SSOT 已收口：`exit-codes.md`、`error-codes.md`、`json-output.md`、`views-scopes.md`。
- `schemaVersion` 当前 = **9**（`storage/connection.rs` `CURRENT_SCHEMA_VERSION`，对应 migration 数量 V001–V009）。所有 `--json` 输出与 golden 均带 `"schemaVersion": "9"`。
- migration 链 V001–V009（V009 = constraint severity 三档化）。
- 契约可验证产物（实量，全绿）：`contracts/cli/json-schemas/`（27 schema）、`examples/`（32 文件：29 JSON golden + 3 .txt transcript）、`contract-tests/`（13 .rs 手写契约测试）、`fixtures/`（35 setup 脚本，含 20 个 `codegen-coverage` 批次新增的自包含 fixture + `assets/non-utf8.bin` 二进制资产）、`generated/`（16 .gen.rs 生成式契约测试）。
- **生成式契约测试**：`crates/spec-gen`（dev-only xtask）从 `openspec/specs/**/spec.md` 的 `@codegen` 块生成 `contracts/cli/generated/<domain>.gen.rs`。当前 **16 个命令 spec 全部带 `@codegen` 块**：chapter-context（15 块）、chapter-lifecycle（22）、chapter-state-machine（13）、state-transition-guards（15）、content-integrity（10）、setting-commands（25）、constraint-commands（33）、rule-checking（9）、volume-lifecycle（16）、volume-sealing（16）、volume-goals（17）、mainplot-node-management（23）、thread-management（22）、emergence-management（46）、verdict-management（11）、project-lifecycle（7），**共 300 个 `@codegen` 块、16 份 `.gen.rs`**（另有 codegen-* 两个批次 delta spec 记录本批次的覆盖要求，非命令 spec）。**@codegen 覆盖面 = 16/16 命令 spec（100%）**，由 `codegen-coverage` 批次 01~06 完成（01 为 DSL 地基，02 setting/constraint/rule-checking，03 volume，04 chapter-state，05 destiny-weaving，06 project-lifecycle 收尾）；不可生成表达的场景（写侧拒绝的 stderr 错误码、纯内部存储故障、并发时序、`--path`/cwd 回退、`--help`、CLI Contract 文档 meta、引用完整性 delete 类、纯人读弱断言）以 `<!-- 注：本场景不补 @codegen（跳过） -->` 就地注记，由既有手写契约测试继续承担。防漂移检查 `scripts/check-generated-drift.sh` 在内存重生成并 diff，漂移即 exit 1。涉及 `@codegen` 的切片 apply/verify/archive 必须通过防漂移检查（见 `openspec/config.yaml`）。

## 6. 已落地契约修复批次

### 6.1 阶段 A–E（直接契约修复，2026-07-23）

| 阶段 | 主题 | 要点 |
|---|---|---|
| A+B | 文档对齐 + schemaVersion 假绿清理 | 4 schema 标注对齐；s06 三渲染器硬编码 `"1.0"` → `CURRENT_SCHEMA_VERSION`；测试断言改用常量 |
| C | severity 三档化 + schemaVersion 8→9 | `ConstraintSeverity` Soft→Info/Warning/Hard；V009 migration |
| D | constraint check-write 契约补全 | `constraint-check-write.schema.json` + golden + 3 contract test |
| E | fixture worlds setup 脚本 | `contracts/cli/fixtures/` 三脚本 + README |

### 6.2 三基线对齐批次（cli-contract-alignment-audit 驱动，2026-07-24 至 2026-07-30）

8 个 `s0x-` 治理切片逐项修复审计报告发现：契约文档自洽（s01）、存储一致性（s02）、命令层下沉（s03）、错误渲染统一（s04）、契约测试覆盖（s05）、JSON 标志对齐（s06）、JSON 序列化风格（s07）、卷状态机入口显性化（s08）。

## 7. 下一步推进方向（规划）

### 7.0 设计覆盖面反推（系统设计 → 应有模块/内容 → 实际完成）

本表是 §0 方法论步骤 1-2 的产物：从设计文件反推"系统应有哪些模块/内容"，对照步骤 3-4 的实际完成。设计文件不再自行列举规划态实体，统一收敛到此表 + §7.2。

| 设计来源 | 应有模块/内容 | 应有命令族 | 实际完成 | 落位 |
|---|---|---|---|---|
| CLI架构设计 §6 / commands.md | 12 组命令 | init/info、setting、constraint、chapter、verdict、volume、mainplot、thread、emergence、export、verify | ✅ 46 条（见 §4 矩阵） | `commands/*.rs` + `commands.md` |
| CLI架构设计 §6.4（创作对象族） | event/section/residue/phase-log/character/subplot/foreshadowing/payoff/promise | （无 commands.md 条目） | ❌ 规划态 | §7.2 创作对象族 |
| CLI架构设计 §7.4（生命周期族） | creation-cycle/great-cycle/archive chapter/archive volume/verify/verdict get/setting import/constraint import | （无 commands.md 条目，部分有独立设计） | ❌ 规划态 | §7.2 生命周期族 |
| CLI架构设计 §6.8（适配族） | view summary/entity/graph、serve | （无 commands.md 条目） | ❌ 规划态 | §7.2 适配族 |
| 项目整体架构 §分层 | Spec 运行层 | skills、adapters、runtime、prompts、rules、workflows、fixtures、tests | ✅ Claude Code 单章闭环已落地；Codex 与后续旅程按顺序切片推进 | §7.1 |
| contracts/cli views-scopes §A | 三组受控承载命令 | setting render --view / constraint render --view,scope / chapter context --view | ✅ 已实现可调用（§4.1） | `commands/{setting,constraint,chapter}.rs` |

> 设计文件涉及上述实体的实现状态时只留指针"见 dev-progress §7.0/§7.2"，不在多处重复列举规划态清单（CLAUDE.md §15）。

### 7.1 Spec 运行层 `spec/` 落地

Spec 运行层已按平台中立公共源与 Agent 薄适配分层落地，并仅通过 `novel ... --json` 子进程访问世界状态、处理中断与 repair 路径。依赖方向：Agent 适配 → 公共 Spec →（子进程）→ CLI → SQLite + 正文文件。

#### 7.1.1 启动状态（2026-09-22，首个里程碑）

首个里程碑已落地：**骨架 + 最小读流程示范**。Spec 运行层由 AI 助手原语（Claude Code / Codex / WorkBuddy）承载，而非独立可执行二进制。

- **已建目录与资产**：
  - `spec/README.md`：Spec 运行层总入口（定位、单向依赖边界、五目录布局、消费方式）。
  - `spec/rules/00-spec-boundaries.md`：Spec 与 CLI 硬边界（只读 / 写 / 判断 / 失败处理 / Context Packet / 版本契约）。
  - `spec/rules/01-cli-contract-consumption.md`：CLI 契约消费规范（`--json`、schemaVersion 校验、V2 错误信封、类别退出码）。
  - `spec/prompts/README.md` + `spec/prompts/writer-context.md`：角色提示词模板（writer 视角消费 writer view 生成 Chapter Task List）。
  - `spec/workflows/README.md`：15 步标准章节写作流水线全景。
  - `spec/workflows/chapter-context-compile.md`：**最小读流程示范**（步骤 0–2 读闭环 + 步骤 3 Task List 衔接），含每一步的 CLI 命令、JSON 解析、中断条件。
  - `spec/fixtures/setup-world.sh`：示例小说世界初始化脚本（init / setting / constraint / volume / chapter）。
  - `spec/runtime/README.md` + `spec/runtime/novel-cli.sh`：会话协议说明 + 统一 novel 调用封装。
- **已验证**：用 `fixtures/setup-world.sh` 初始化示例世界并实跑 `novel chapter context --view writer --json`，确认：world 预检（`schemaVersion` `"9"` 校验）、writer view Packet 结构与 `json-schemas/chapter-context.schema.json` 一致、`overflow.omitted` 诚实省略（`no_data_backing_table`）、极小预算触发退出码 3 + `context_budget_exceeded` 结构化失败（V2 envelope + `partialPacket`，证明"超限即中断，非静默截断"）。
- **依赖方向**：Spec →（子进程）→ CLI → SQLite + 正文文件，已自约束（`rules/00` 禁止直接存储访问）。
- **待建（规划中）**：步骤 10–15 的章后数据沉淀、卷末监测和大轮回流程（依赖 `great-cycle` / `archive verify` 规划态命令）；context auditor / task planner 等其余角色提示词模板。
- **第二个里程碑已落地**：`spec/workflows/chapter-write-verify-repair.md` + `spec/prompts/verifier.md` + `spec/prompts/repair.md` 已覆盖步骤 5–9 的最小写验修复闭环；步骤 10–15（章后规划态实体、卷末监测和大轮回）仍待实现。
- **第三个里程碑已落地**：`spec/skills/chapter-writing/SKILL.md` 作为公共单章入口，`spec/adapters/claude/` 与 `spec/adapters/codex/` 提供双端薄适配，`spec/install.sh` 按显式清单安装到目标项目的 `.novel-spec/`、`.claude/` 与 `.agents/skills/`；安装、原语发现和成功/repair 单章闭环均有临时目录契约测试，双端行为 parity 已加入切片 02 验收。

### 7.2 未实现设计实体清单

> 此清单供 Spec 运行层（spac）搭建时对照：已实现命令可依赖（见 §4 矩阵），未实现实体属规划态，Spec 流程不得假设其可用。每项按 §0 方法论三否证判据判定（无 commands.md 条目 / 无代码 / 无 spec Given/When/Then = 规划态）。
> 契约源已迁类型化代码：`contracts/cli/*.md` 的机器列/语法段与 `json-schemas/*.json` 为生成物（`contract-governance-02/03/04`），本清单实体集合不因契约源迁移而增减。

**创作对象族**（`需求说明书 §3.1` 要求管理但 CLI 未实现）：

| 实体 | 设计来源 | 前缀登记（entity-ids §A） | 状态 |
|---|---|---|---|
| event（事件） | CLI架构设计 §6.4、Spec架构设计 §4 | ✅ `ev` | 规划态，无命令/无 spec |
| section（小节） | CLI架构设计 §6.4、Spec架构设计 §4 | ✅ `sec` | 规划态，无命令/无 spec |
| residue（残余命令） | storage-model、CLI架构设计 §6.6 | ✅ `res` | 规划态，无命令/无 spec（目录 `content/residues/` 已建） |
| phase-log（四相日志） | terminology §四相日志、CLI架构设计 §6.6 | ❌ | 规划态，无命令/无 spec（review 门禁规划态项） |
| character（角色） | CLI架构设计 §6.8、需求说明书 §3.1 | ❌ | 规划态，无命令/无 spec |
| subplot（支线） | CLI架构设计 §6.3、需求说明书 §3.1 | ❌ | 规划态，无命令/无 spec |
| foreshadowing（伏笔） | CLI架构设计 §6.3 | ❌ | 规划态，无命令/无 spec |
| payoff（爽点） | CLI架构设计 §6.3 | ❌ | 规划态，无命令/无 spec |
| promise（读者承诺） | CLI架构设计 §6.3 | ❌ | 规划态，无命令/无 spec |

**生命周期族**：

| 实体 | 设计来源 | 状态 |
|---|---|---|
| creation-cycle（创作周期） | CLI架构设计 §7.4、需求说明书 §4 | 规划态，`creation-cycle status/complexity/advance` 无命令/无 spec |
| great-cycle（大轮回） | CLI架构设计 §6.7、Spec架构设计 §7 | 规划态，`great-cycle start/close/list` 无命令/无 spec |
| archive chapter（章节归档） | CLI架构设计 §6.7、Spec架构设计 §4.7 | 规划态，无独立命令（`volume archive` 已实现卷归档） |
| archive volume/verify（封典终检） | CLI架构设计 §6.7、Spec架构设计 §7.2 | 规划态，`archive verify` 无独立命令（`verify integrity` 已实现完整性校验，非封典终检） |
| verdict get | CLI架构设计 §6.8、Spec架构设计 §4.4 | 规划态，`verdict get` 无命令/无 spec（仅 `verdict set-mainplot-landing` 已实现） |
| setting import | CLI架构设计 §6.2 | 规划态，无命令/无 spec |
| constraint import | CLI架构设计 §6.2 | 规划态，无命令/无 spec |

**适配族**：

| 实体 | 设计来源 | 状态 |
|---|---|---|
| view summary/entity/graph | CLI架构设计 §6.8、Spec架构设计 §8 | 规划态，无命令/无 spec |
| serve | CLI架构设计 §6.8、Spec架构设计 §8 | 规划态，无命令/无 spec |

**V2 错误协议（已实现，非规划态）**：`contract-registry-v2-01-error-protocol-foundation` 实现类型地基（`ErrorCategory`/`ErrorCatalog`/`V2ErrorEnvelope`/`ErrorProtocolVersion`）；`contract-registry-v2-02-error-catalog-migration` 将目录扩充为覆盖全部公开错误 ID 的真目录，新增 `ErrorCode::category()`/`CliError::error_id()`/`V2ErrorEnvelope::from_catalog()` 与 renderer 的 `ErrorProtocolVersion` 选择机制（缺省 V1 输出逐字不变，显式 V2 输出信封+类别退出码）。**05 默认化后状态（当前）**：CLI 默认错误协议为 **V2**——`ErrorProtocolVersion::default()` 翻转为 V2，`--json` 失败输出为 `V2ErrorEnvelope`，进程退出码与信封 `exitCode` 由 `ErrorCatalog` 类别派生（类别退出码）；V1 `ErrorObject` 与 per-entity 退出码转**弃用兼容路径**，仅显式选择 V1 时输出并逐字保持 V1 回归基线。**兼容期/弃用/移除进入条件当前满足度**：V2 默认已生效；V1 仍保留（兼容期未结束）；V1 物理移除须由独立变更按 `contract-v2-governance` 三条件齐备准则执行（兼容期结束 + 消费者迁移完成 + 独立移除变更前置：类目码收敛/VerifyIntegrity 目录纯派生落地、契约测试/golden 只锁 V2、生成物无漂移），当前尚未满足、尚未启动。

### 7.3 低优先级跟进

- `entity-ids-sourcerefs.md §A` 登记了 `section`(sec)/`event`(ev)/`residue`(res)/`setting`(style) 四类尚无实现实体前缀，需确认是"格式先于实现冻结"的刻意设计还是应降为 §B 预留（见 §7.2 清单"前缀登记"列）。
- `master` 同步与历史切片合并状态按需评估。

## 8. 文档对齐治理（2026-08-11，已提交归档）

该轮治理为纯文档对齐，不改代码、不改契约取值（现已提交，工作树 clean）：

- **docs 重构**：三份主文档（项目整体架构/CLI架构设计/Spec架构设计）激进瘦身，剥离命令语法；5 份设计亮点抽到独立文件（`docs/设计亮点-*.md`）；`CLI模块对齐清单.md` 删 checkbox。
- **CLAUDE.md 精简**：§2 进度内容迁出至本文件，只留构建命令+定位。
- **契约漂移修复**：`README.md` 承载命令声明、`commands.md` volume seal 门禁语义、8 golden schemaVersion→9、`error-codes.md` s06 细分子表、`STATUS.md` 文件清单、`mainplot node update` 成功消息——详见 `docs/cli-contract-alignment-audit-report.md`。
- **系统运行限制抽离 + 评估收口**：CLAUDE.md §7（正文存储行为）/§10（状态机与门禁清单）内联的系统运行内容抽至 SSOT（`storage-model.md`/`state-machine.md`/`terminology.md`），只留执行规则 + 指针，消除门禁 5项/7项自抄漂移根因；§1 补 README 入口指针、§8 spec/ 目录状态指针校正至 §2；`README.md` 过时命令声明（"后续切片将实现规则检查、大轮回关闭、归档导出"）改为 dev-progress 指针（规则检查/归档导出已实现）；`json-output.md` 补 §8 其他命令族（mainplot/thread/emergence/verdict）输出结构覆盖指针。

## 9. 关键命令速查（验证用）

```bash
cargo build -p novel-cli          # 构建（当前通过）
cargo test -p novel-cli           # 全量测试（当前 0 failed）
cargo run -p spec-gen -- generate # 从 @codegen 块重新生成契约测试（dev-only xtask）
scripts/check-generated-drift.sh  # 生成式测试防漂移检查（漂移 exit 1）
git status -s                     # 工作树状态
```

## 10. 接续开发的进入检查清单

- [ ] 确认当前分支与 HEAD（见 §2）
- [ ] 工作树状态：当前工作树 clean（无未提交变更，见 §2）
- [ ] 任何契约变更同步更新 `contracts/cli/` 文档 + schema + golden + 契约测试（CLAUDE.md §9/§15）；涉及 `@codegen` 块的切片须通过 `scripts/check-generated-drift.sh`（CLAUDE.md §2）
- [ ] 新增写命令须含状态机门禁 / hash / 非零退出码（CLAUDE.md §10/§13）
- [ ] 不直接读写 SQLite / `.novel/content`，只经 `novel` 命令（CLAUDE.md §3/§8/§14）
