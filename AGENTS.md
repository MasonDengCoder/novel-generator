# AGENTS.md

本文件只保留 Codex 在本项目中的长期执行约束和实现指令。

**文档职责边界**：
- `AGENTS.md`（本文件）：Codex 在本项目如何工作（命令、执行边界、规则、危险操作、完成前验证）
- `repository/`：稳定项目知识（术语、架构、模块边界、集成约束、长期决策）
- `docs/`：详细设计文档、用户指南与架构权衡说明
- `contracts/cli/`：CLI Contract 机器可验证契约

整个项目中的文档均使用 **中文**。

## 1. 项目定位

- `novel-generator` 是面向小说创作项目的生成器与运行层工程，不是某一本小说正文、数据库或世界状态的存放处。
- 本仓库负责提供 `novel` CLI、Spec 运行层、Codex 原语、规则、工作流、契约与安装能力。
- 具体小说项目的 `.novel/` 数据库、章节正文、报告和运行产物应保存在目标小说项目中，不应写入本仓库。
- Spec 安装脚本的目标是把本仓库中的 Codex 原语复制到目标小说项目的 `.Codex/` 目录，并调用 `novel` CLI 初始化目标项目内的状态。

**人类可读项目概况与使用方式**：见 `README.md`；**详细架构与分层模型**：见 `repository/architecture.md`。

## 2. 当前仓库状态与命令

- **实现进度**：切片归档、命令组实现矩阵、未实现设计实体清单见 `repository/dev-progress.md`，本文件不承载进度快照。
- 可执行的构建与测试命令：
  - 构建：`cargo build -p novel-cli`
  - 全量测试：`cargo test -p novel-cli`（含 `crates/novel-cli/tests/` 与 `contracts/cli/contract-tests/` 与 `contracts/cli/generated/` 下的集成/契约/生成测试）
  - 单测：`cargo test -p novel-cli --test <name>`，如 `--test init_ok`、`--test info_json`、`--test e2e`
- 生成式契约测试（从 spec @codegen 块生成，构造性消除 spec↔test 漂移）：
  - 生成器：`crates/spec-gen`（dev-only xtask，不在 `novel` CLI 的正常构建链中）
  - 生成：`cargo run -p spec-gen -- generate`（从 `openspec/specs/**/spec.md` 的 `@codegen` 块生成 `contracts/cli/generated/<domain>.gen.rs`，断言期望值以 `exit-codes.md`/`error-codes.md` SSOT 解析为准；散文与 SSOT 冲突即阻塞生成）
  - 防漂移检查：`scripts/check-generated-drift.sh`（在内存重新生成并与仓库生成物 diff，漂移即 exit 1；涉及 `@codegen` 的切片 apply/verify/archive 必须通过，见 `openspec/config.yaml`）
- 尚未配置 lint 或格式化命令；后续可按需引入 `cargo clippy`/`cargo fmt`。
- `.gitignore` 当前忽略 `.Codex/skills/`、`.Codex/commands/` 与 Rust 构建产物 `target/`。
- `.Codex/` 中的本地工具配置或实验变更不要当作默认应提交内容。
- `openspec/` 不再被忽略，其配置文件可以提交。

## 3. 总体边界

- `novel` CLI 是世界状态日志的唯一入口和硬层内核。
- 外部系统只能通过 `novel` 命令访问世界状态。
- 不提供外部 SDK/API 绕过 CLI 内部模块。
- CLI 负责状态机、权限边界、事务一致性、错误码、封卷门禁、归档与防绕过。
- Spec 运行层只负责创作流程、阶段编排、提示词、规则注入、LLM/代理调用、验证流程，以及对子进程 `novel` 命令的调用。
- CLI Contract 是软硬层唯一稳定边界；SQLite schema、`.novel/` 内部路径、Rust module 名称和 Spec prompt 形态都不是跨层契约。

**详细边界定义**：见 `repository/contract-boundary.md`。

## 4. 技术选型硬约束

- CLI 主体采用 **Rust**。
- CLI 正式存储采用 **SQLite**，不得以临时 JSON 文件作为正式世界状态存储。
- 章节正文采用 **文件存储 + SQLite 元数据索引** 的混合策略。
- 推荐 Rust 技术栈：
  - CLI parser：`clap`
  - SQLite：`rusqlite`
  - Migration：`refinery` 或项目内置 SQL migration runner
  - Serialization：`serde`、`serde_json`、`serde_yaml`
  - Error：`thiserror`、`anyhow`
  - Time：`time`
  - Human-readable table：`comfy-table` 或自研轻量 renderer
  - CLI integration tests：`assert_cmd`、`predicates`、`tempfile`

## 5. 目标目录边界

采用 monorepo，逻辑拆分为 CLI 硬层项目与 Spec 运行层项目。新增代码时按以下目标边界落位；不要因为当前目录尚未全部存在而改用临时结构。

**目标结构**（部分目录已存在，部分规划中）：

```text
novel-generator/
  crates/
    novel-cli/              # Rust CLI：硬层、SQLite、状态机、事务、命令 [已存在]
  spec/                     # Spec 运行层 [规划中]
    runtime/                # 运行引擎
    prompts/                # 提示词模板
    rules/                  # 规则定义
    workflows/              # 流程编排
    fixtures/               # 测试 fixture
  contracts/
    cli/                    # CLI 契约 [已存在]
  docs/                     # 项目文档 [已存在]
  reference/                # 参考文档 [已存在]
  repository/               # 项目知识库 [已存在]
  openspec/                 # OpenSpec 配置与变更 [已存在]
```

依赖方向必须保持单向：

```text
Spec 运行层
  │
  │  仅通过子进程调用 novel 命令
  ▼
CLI 硬层
  │
  ▼
SQLite + 正文文件
```

约束：

- Spec 只能通过子进程调用 `novel` 命令访问世界状态。
- CLI 不得反向依赖 Spec，不 import / link `spec/runtime`，不调度 LLM，不读取 Spec 运行时缓存作为权威状态。
- Web/view/serve 等适配层不得绕过 service/use-case 与 guard。

## 6. CLI 内部分层

简要职责：
- `commands` 只做参数解析、基础参数校验和输出选择，不直接访问 SQLite 或正文文件。
- `services` / guard 统一执行状态机、权限矩阵、前置条件和跨实体一致性。
- `repositories` 封装 SQL 与文件引用读取，不做业务判断。
- `storage` 负责 SQLite transaction、migration、锁和文件写入原子性。
- `renderers` 负责人读输出与 JSON 输出，不决定业务规则。
- `validators` 负责引用完整性、参数合法性、schemaVersion 校验等可复用检查。
- `errors` 负责统一错误类型、结构化错误对象与退出码映射。

**详细模块职责与调用链**：见 `repository/cli-modules.md`。

## 7. 章节正文存储与一致性

正文存储与一致性是**系统运行行为**：SQLite 权威索引 + 文件存储的混合策略、10 步写入流程、hash 检测外部修改、`approved` 终态、Stale 保护、归档冻结的完整定义见 `repository/storage-model.md`，相关术语见 `repository/terminology.md`。

Codex 与 Spec 开发时遵守的执行规则：

- 写入章节正文必须经 `novel chapter update <id> --content <file>` 或受控命令，不得绕过 CLI 直接改 `.novel/content/chapters/` 下的正文文件。
- 绕过 CLI 的直接编辑会被 hash 检测拦截（错误码 `content_modified_externally`；默认 V2 下退出码为 `external_modification` 类别码 `5`），CLI 拒绝继续推进状态/归档/校验。退出码/错误码取值以 `contracts/cli/exit-codes.md`/`error-codes.md` 为 SSOT。
- 存储细节、一致性保障与写入流程的权威清单不在本文件维护；以 `storage-model.md` 为准，本文件不复制其列表（避免漂移）。

**详细存储模型、一致性保障与 10 步写入流程**：见 `repository/storage-model.md`。

## 8. Spec 运行层访问规则

Spec 可以执行受控 CLI 命令，例如：

```bash
novel chapter context <id> --view writer --budget <n> --json
novel chapter status <id> review
```

完整命令清单见 `contracts/cli/commands.md`。写 Spec 流程时须以 CLI Contract 与实现状态为准：对尚未记录于 `commands.md` 的命令不得假设其已可用；未实现设计实体（事件/小节/角色/残余/四相日志等）属规划态，清单见 `repository/dev-progress.md` §7.2。Spec 运行层（`spec/`）目录状态见 `repository/dev-progress.md` §2（当前规划态、目录未创建），落地方向见同文件 §7.1，其提示词与工作流形态不属于已实现内容。

Spec 不得：

- 直接读取或修改 SQLite。
- 直接读取或修改 `.novel/content` 下的正文文件作为状态来源。
- import / require / link CLI 内部模块。
- 依赖 SQLite 表结构、迁移文件、内部路径或 Rust 模块名。
- 自行判断章节是否可进入 `review` / `approved`、卷是否可归档、封典是否通过。
- 自行重排、删除、重压缩或补全 CLI canonical Chapter Context Packet 的 P0/P1 片段。
- 忽略 CLI 非零退出码继续推进流程。

这些判断必须由 CLI service/use-case 与 guard 执行；Spec 只消费 CLI 稳定机器输出并将其渲染为 prompt 或流程输入。

## 9. CLI Contract 对齐

CLI 项目与 Spec 项目必须通过版本化 **CLI Contract** 对齐。

CLI Contract 至少包括：
- 命令语法与参数约定
- 稳定 JSON 输出 schema
- 结构化错误对象 schema
- 退出码语义
- 实体 ID 与引用规则
- golden examples
- fixture worlds
- contract tests
- schema/version 演进策略

**详细契约边界定义、演进策略与 Spec 消费最佳实践**：见 `repository/contract-boundary.md`。

稳定契约：
- `novel` 命令名称与参数语义
- `--json` 输出结构
- `schemaVersion` 字段
- 退出码类别
- 结构化错误码
- 错误协议版本（默认 V2：`V2ErrorEnvelope` + `ErrorCatalog` 类别派生退出码；显式选择 V1 为弃用兼容路径）
- 受控 view/scope 名称及其机器语义

非稳定实现细节：
- SQLite schema
- `.novel` 内部路径
- 正文文件布局
- Rust module 名称
- repository/storage 实现
- migration 具体文件名

## 10. 状态机与硬阻断重点

CLI 硬层内置三类状态机（章节 / 涌现 / 卷）：状态值、合法转换、终态约束、进入 `review`/`approved` 的生效门禁清单是**系统运行行为**，以 `contracts/cli/state-machine.md` 为 SSOT，术语定义见 `repository/terminology.md` §状态机。规划态门禁（残余、四相日志，对应命令尚未实现）见 `repository/dev-progress.md` §7.2，实现前不阻断状态推进。

Codex 与 Spec 开发时遵守的执行规则：

- 状态推进只能经 `novel chapter status <id> <target> --json`、`novel volume seal|archive` 触发，Spec 不得自行判断章节可否进入 `review`/`approved`、卷可否封卷/归档。
- 默认错误协议为 V2：任何 hard 阻断 MUST 以非零退出码（full 类别退出码，如 `3` 前置阻断/非法状态转换、`5` 外部修改、`6` 数据不存在，完整取值以 `exit-codes.md` 为 SSOT）+ 结构化错误信封（`V2ErrorEnvelope`）返回，不允许静默降级为提示。进程退出码与信封 `exitCode` 由 `ErrorCatalog` 类别派生；V1 per-entity 码仅在显式选 V1 时生效。
- 门禁与转换路径的权威清单不在本文件维护；以 `state-machine.md` 为准，本文件不复制其列表（避免漂移）。

**详细术语定义**（章节四相日志、主线落地裁决、规则检查结果、Stale 保护、Hash 检测等）：见 `repository/terminology.md`；生效门禁清单见 `contracts/cli/state-machine.md`。

## 11. 文档职责边界

| 载体 | 职责 |
|---|---|
| `README.md` | 面向人类说明项目用途、使用方式、产物位置和架构摘要 |
| `AGENTS.md`（本文件） | Codex 在本项目工作时必须遵守的长期规则和实现指令，不承载一次性需求、当天 TODO、阶段性计划或完整架构说明 |
| `repository/` | 稳定项目知识（术语、架构、模块边界、集成约束、长期决策） |
| `docs/` | 详细设计文档、用户指南与架构权衡说明 |
| `contracts/cli/` | CLI Contract 机器可验证契约 |
| `reference/` | 参考文档（如 Codex 四原语能力边界） |
| `openspec/` | OpenSpec 配置、变更流程与 artifact 规则 |

## 12. 禁止的自我合理化

当规则与便利性冲突时，不要通过重新解释规则来绕过约束。常见的自我合理化模式包括但不限于：

- "虽然规则说不要 X，但这次情况特殊..." → STOP，询问用户
- "这只是临时的..." → 临时方案必须有明确的清理计划
- "用户可能想要..." → 不要替用户做架构决策
- "这样更快..." → 速度不是绕过分层边界的理由
- "规则没有明确禁止 Y..." → 边界外的操作需要询问

如果你发现自己在解释为什么某个操作"实际上符合规则"，停下来重新评估。

## 13. 开发 Do / Don’t

Do：

- 新增代码前先确认属于 CLI 硬层、Spec 运行层、CLI Contract、docs 还是安装能力。
- 新增 CLI 功能时优先补契约、JSON schema/golden examples 和契约测试，再实现内部存储细节。
- 新增写命令时把状态机、权限、hash、一致性和非零退出码作为验收条件。
- 新增 Spec 流程时只通过 `novel ... --json` 消费世界状态，并显式处理中断和 repair/回退路径。
- 修改文档时保持职责单一：架构解释放 `docs/`，Codex 执行规则放本文件。
- 新增或修改 CLI 命令实现（`crates/novel-cli/src/commands/`）后，必须同步更新对应的 `contracts/cli/` 文档（`commands.md` 条目 + 涉及的 `exit-codes`/`error-codes`/`json-output`/`views-scopes`/`entity-ids-sourcerefs`）+ 补/改 golden example、json-schema、contract test，并运行 `cargo test -p novel-cli` 与 `scripts/check-generated-drift.sh`；契约未对齐不得归档。
- CLI 命令实现或切片推进后，必须同步更新 `repository/dev-progress.md` 对应章节（切片时间线 §3 / 命令矩阵 §4 / 未实现实体清单 §7.2）；进度未同步不得归档。

Don’t：

- 不要在本仓库保存具体小说项目的 `.novel/`、章节正文、世界数据库或运行报告。
- 不要让 Spec、Skill、Subagent、Web 或脚本直接读取/修改 SQLite 或 `.novel/content`。
- 不要提供绕过 CLI 的 SDK/API 或公开内部 Rust module 作为稳定入口。
- 不要让 CLI 调度 LLM、决定 prompt 形态或保存 Spec 运行时上下文。
- 不要在 JSON 输出中混入 ANSI 颜色、解释性旁白或非结构化提示。
- 不要把临时 JSON 文件作为正式世界状态存储。
- 不要将用户的一次性业务需求、临时计划、聊天上下文或阶段性开发任务写入 `AGENTS.md`。

## 14. Dangerous Operations（危险操作与 STOP 条件）

以下操作具有高风险，执行前必须 **STOP 并确认**：

### 数据删除与破坏性操作

**STOP 条件**：
- 删除 `.novel/` 目录或其子目录
- 删除 SQLite 数据库文件（`novel.db`）
- 删除章节正文文件（`.novel/content/chapters/`）
- 删除契约测试、golden examples 或 JSON schemas
- 批量删除或重命名实体（如批量删除章节、卷、涌现）

**原因**：不可逆，可能导致世界状态丢失或契约验证失效。

**替代方案**：
- 使用 CLI 提供的归档命令而非直接删除
- 如需清理测试环境，使用 `tempfile` 创建隔离目录
- 删除前备份 `.novel/` 目录

### 绕过 CLI 的直接数据库操作

**STOP 条件**：
- 直接打开 SQLite 连接并执行 INSERT/UPDATE/DELETE（除了 CLI 内部实现）
- 让 Spec、Skill、Subagent、Web 或脚本直接读写 `novel.db`
- 绕过 `services/` 和 `storage/` 层直接修改数据

**原因**：破坏状态机、一致性保障、hash 检测和防绕过机制。

**替代方案**：
- 通过 `novel` 命令访问世界状态
- 在 CLI 内部实现中通过 `storage/` 和 `repositories/` 层访问数据库

### 修改已归档或 `approved` 状态的实体

**STOP 条件**：
- 修改已 `approved` 章节的正文、裁决、残余、四相日志
- 修改已封卷的卷内实体
- 修改已归档的涌现或事件

**原因**：违反状态机终态约束，破坏审计追溯能力。

**替代方案**：
- 如需修复错误，先回退状态（如 `approved` → `review`），修复后重新推进
- 记录回退原因与修复日志

### 破坏性 Git 操作

**STOP 条件**：
- `git push --force` 到 `master` 或 `main`
- `git reset --hard` 且工作区有未提交变更
- 删除远程分支（除非明确是临时分支）
- 修改已发布的 git tag

**原因**：可能覆盖他人工作或丢失历史记录。

**替代方案**：
- 使用 `git revert` 而非 `git reset --hard`
- Force push 前确认远程分支状态
- 删除分支前确认已合并

### 修改稳定契约而不更新测试

**STOP 条件**：
- 修改 `contracts/cli/` 文档但未更新契约测试
- 修改命令参数、JSON 输出、退出码但未运行 `cargo test -p novel-cli`
- 修改 `schemaVersion` 策略但未更新所有 JSON 输出

**原因**：破坏软硬层契约，导致 Spec 运行层失败。

**替代方案**：
- 契约变更必须同步更新契约测试、golden examples 和 JSON schemas
- 运行全量测试确保契约对齐

### 生产部署与外部系统变更

**STOP 条件**：
- 部署到生产环境（本项目当前无生产环境，未来如有需 STOP）
- 修改外部 API、数据库连接配置
- 修改权限、认证或访问控制逻辑

**原因**：影响线上服务或安全边界。

**替代方案**：
- 在测试环境验证后再部署生产
- 配置变更前备份现有配置
- 权限变更前明确影响范围

### Hook 硬门禁升级建议

高风险操作可升级为 **Codex Hook（PreToolUse）** 实现模型无法绕过的硬门禁。如需配置 Hook，见 `/harness-hook-setup` 或项目 `repository/` 中的 Harness 使用流程文档（如存在）。

## 15. Before Finishing（完成前检查清单）

完成工作前必须执行以下检查（构建与测试命令见 §2）：

### 通用检查（所有变更）

- [ ] 命令执行成功（如 `cargo build -p novel-cli`、`cargo test -p novel-cli`）
- [ ] 没有记录私密凭据、账号、token 或具体小说项目正文
- [ ] 没有把一次性需求、今日 TODO 或聊天上下文写入 `AGENTS.md` 或 `repository/`
- [ ] 文档职责边界清晰（`AGENTS.md` vs `repository/` vs `docs/` vs `contracts/cli/`）

### CLI 实现变更检查

- [ ] 命令语法、`--json` 输出、错误对象和退出码符合 `contracts/cli/` 契约
- [ ] 状态机、权限矩阵和前置条件在 `services/` / guard 层硬阻断
- [ ] 涉及正文写入时，具备 transaction、锁、临时文件清理和 hash 检测
- [ ] `approved` / archived / stale verdict / stale rule check / external hash mismatch 被正确拒绝（非零退出码）
- [ ] 集成测试、数据一致性测试和契约测试覆盖新增行为
- [ ] `cargo test -p novel-cli` 全量通过
- [ ] 新增/修改 CLI 命令实现后，已同步更新 `contracts/cli/` 对应文档（`commands.md` + 涉及的 `exit-codes`/`error-codes`/`json-output`/`views-scopes`）+ schema + golden + contract test，并运行 `cargo test -p novel-cli` 与 `scripts/check-generated-drift.sh`
- [ ] 命令实现或切片推进后，已同步更新 `repository/dev-progress.md` 对应章节（§3 时间线 / §4 命令矩阵 / §7.2 未实现实体清单）

### Spec 实现变更检查（规划中）

- [ ] 只通过 `novel` 子进程调用访问世界状态
- [ ] 面向机器消费的读取使用 `--json` 并检查 `schemaVersion`
- [ ] 正确处理非零退出码并停止后续推进
- [ ] 不依赖 SQLite、正文文件路径、CLI 内部模块或运行时缓存作为权威状态
- [ ] 尊重 CLI canonical Chapter Context Packet 的 P0/P1、audit 与 overflow

### 文档变更检查

- [ ] 与 `README.md`、`repository/`、`docs/`、`contracts/cli/` 的职责边界一致
- [ ] 删除或避免了过时命令、死路径、猜测性技术栈和重复大段说明
- [ ] 明确区分了"当前状态"与"目标规划"
- [ ] 遵守"引用而非自抄"原则（不复制退出码表、错误码表、view/scope 列表）
- [ ] 进度内容只落 `repository/dev-progress.md`：切片归档时间线、命令实现矩阵、未实现实体清单、schemaVersion 当前值、状态快照句（"已实现 X 条"/"规划态"/"尚未"）不得出现在 `docs/`、`reference/`、`README.md` 或本文件；设计文件涉及实现状态时只留指针"见 `repository/dev-progress.md` §N"

### Contract 变更检查

- [ ] 更新了对应的 `contracts/cli/` 文档（commands/exit-codes/error-codes/json-output/etc.）
- [ ] 更新了 `json-schemas/` 和 golden examples（如涉及）
- [ ] 添加或更新了契约测试
- [ ] 契约测试通过（`cargo test -p novel-cli` 包含契约测试）
