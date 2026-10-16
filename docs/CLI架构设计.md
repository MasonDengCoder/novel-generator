# 小说创作工具系统 — CLI 架构设计

> **定位**：本文只描述 CLI 硬层"做什么"与"为什么这样分层"。设计亮点（状态机、章契约、涌现、设定分层等）已抽离至 `docs/设计亮点-*.md`；命令契约以 `contracts/cli/commands.md` 为 SSOT；模块职责细节见 `repository/cli-modules.md`。

---

## 1. CLI 的职责

`novel` CLI 是世界状态日志的唯一入口和硬层内核。CLI 必须做到：

1. **唯一入口**：所有世界状态读写必须通过 `novel` 命令完成。
2. **内部封闭**：不暴露外部 SDK/API，不允许 Spec import/link 内部模块。
3. **硬阻断**：状态机、权限矩阵、前置条件、封卷门禁由 CLI service/guard 强制执行。
4. **一致性**：SQLite 元数据、正文文件、hash、状态推进在同一用例中处理。
5. **可审计**：所有机器输出包含稳定 schema、实体 ID、sourceRefs、错误码和建议动作。
6. **可替换内部实现**：SQLite schema、`.novel/` 布局、Rust 模块名都不作为公开契约。

CLI 不做：生成章节正文；调度 LLM/writer/verifier/repair agent；决定 prompt 形态；保存 Spec 运行时上下文；让 Web/view/serve 绕过 service/use-case 与 guard。

---

## 2. 为什么 CLI 单独成层

### 2.1 需要下沉到 CLI 的问题

以下问题不能依赖 LLM 或 Spec 自觉完成，应由 CLI 用确定性代码、事务、锁和错误码处理：章节状态是否能从 `drafting` 进入 `review`；`approved` 或已归档章节是否被修改；主线裁决是否存在且 `achieved=1`；hard 级规则是否有违规；卷目标是否达成；涌现是否完成评分/决策/执行/留痕；正文文件是否被绕过 CLI 修改；命令失败后是否留下半写入状态。

### 2.2 为什么通过命令而不是 SDK

CLI Contract 稳定的是：命令名称与参数语义、`--json` 输出结构、`schemaVersion`、结构化错误对象/错误码/退出码、实体 ID 与引用规则。不稳定的是：SQLite schema、`.novel/` 内部路径、正文文件布局、Rust module 名称、repository/storage 实现、migration 文件名。如果提供 SDK 或允许 import 内部模块，Spec 会自然依赖实现细节，CLI 就失去作为硬边界的价值。因此外部系统只能通过子进程调用 `novel`。

### 2.3 为什么正文用文件 + SQLite 索引

正文不直接作为 SQLite BLOB/TEXT 唯一存储，是为兼顾：作者可读（Markdown 可直接打开/备份/审阅）；硬层一致（SQLite 记录路径引用/hash/字数/状态/归档标记）；防绕过（CLI 在读取/推进/归档/校验时重算 hash，发现外部修改则拒绝继续或要求显式重新导入/确认）。

---

## 3. 内部分层怎么做

### 3.1 模块划分

CLI 内部按模块组织（模块名是实现建议，不属于公开契约）。**各模块的稳定职责定义见 `repository/cli-modules.md` §模块职责（SSOT），本节不重复。** 大致划分：

```text
crates/novel-cli/
  src/
    bin/                  # novel 命令入口，只负责启动 parser
    commands/             # 参数解析、基础参数校验、输出模式选择
    services/             # 用例层：状态机、权限矩阵、硬阻断、跨实体一致性
    domain/               # 领域模型、枚举、状态机、错误码、权限规则
    repositories/         # SQL 与文件引用读取封装，不做业务判断
    storage/              # SQLite 连接、transaction、migration、锁、原子文件写入
    renderers/            # 人读输出、JSON 输出、view/export 渲染
    validators/           # 引用完整性、参数合法性、schemaVersion 校验
    errors/               # 统一错误类型、错误对象、退出码映射
```

### 3.2 调用链

单向调用链（`bin/novel → command parser → adapter → service/use-case → validator/guard → repository → storage transaction → renderer`）与各层"做什么/不得做什么"的职责边界见 `repository/cli-modules.md` §分层调用链与 §模块职责（SSOT）。规则要点：`commands` 只解析参数不直接访问 SQLite 或正文文件；`services` 执行状态机/权限矩阵/前置条件/跨实体一致性；`repositories` 只封装数据访问不做业务判断；`storage` 负责 transaction/migration/锁/文件原子替换/失败清理；`renderers` 只负责输出格式不决定业务规则；`view`/`serve` 只是 adapter 必须复用同一套 service/use-case 与 guard。

### 3.3 写命令的一般执行模板

```text
1. 解析 argv。
2. 做基础参数校验。
3. 获取项目根与 .novel 定位。
4. 获取写锁。
5. 开启 SQLite transaction。
6. service 加载相关实体与当前状态。
7. guard 检查状态机、权限矩阵、前置条件和引用完整性。
8. repository / storage 执行写入。
9. renderer 生成 JSON 或人读输出。
10. 提交 transaction。
11. 释放锁。
12. 失败时回滚并返回结构化错误与非零退出码。
```

这样做的理由：把硬阻断集中在 service/guard 避免命令分支各自漏检查；让 repository 保持简单避免 SQL 层隐含业务规则；用 transaction 和锁保证失败不污染世界状态；用统一 renderer 保证输出稳定可测。

---

## 4. 项目与存储布局

`.novel/` 布局（实现细节，非公开契约）：

```text
.novel/
  novel.db
  content/
    chapters/
      <chapter-id>.md
    residues/
      <chapter-id>.md
    reports/
      <volume-id>.md
  tmp/
  locks/
```

约束：SQLite 是权威元数据、状态机、索引、引用、hash、归档状态来源；正文文件是正文内容载体但其路径与布局不是公开契约；CLI 输出不得要求调用方解析 `.novel/` 路径；导出命令应能从 SQLite 元数据和正文文件组合出完整、可审计的世界数据。

> 存储模型、10 步写入流程、半提交恢复、hash 检测、stale 保护、并发锁、migration 策略的详细设计见 `repository/storage-model.md`。

---

## 5. 正文写入怎么做

涉及正文的写操作必须按 10 步有序事务实现：校验输入正文文件 → 获取项目级写锁 → 打开数据库 → 归档冻结门禁（卷 archived 时拒绝）→ approved 终态门禁 → 开启 SQLite transaction → 计算新 hash 与字数 → 写临时文件 + 原子替换正式正文文件 → 更新 SQLite 元数据 → 提交 transaction（失败清理 temp）。

如果无法严格做到跨 SQLite 与文件系统的完全原子提交，必须保证失败后 CLI 能检测并修复/拒绝不一致状态，不得静默继续。尤其覆盖"正文文件已原子替换但 SQLite commit 失败"的场景——通过 hash 失配检测兜底，CLI 在读取/推进/归档/导出前检查未完成 intent，根据 SQLite 元数据与正文文件 hash 判定为可清理、可修复或必须拒绝。

> **10 步流程的完整步骤、层归属（services 编排门禁与事务、repositories 在事务内执行正文写入与元数据更新）、半提交恢复、hash 检测、stale 保护、并发锁、migration 策略以 `repository/cli-modules.md` §正文写入一致性保障与 `repository/storage-model.md` 为 SSOT，本节不重复步骤明细。**

### 5.1 外部修改检测

CLI 在以下操作前应重新计算正文 hash：读取章节、推进章节状态、归档、规则检查、导出。发现 hash 与 SQLite 记录不一致时返回非零退出码、输出结构化错误码（`content_modified_externally`）、给出修复建议（重新导入/丢弃外部修改/人工处理）。

---

## 6. 功能模块与命令契约引用

CLI 对外命令统一以 `novel` 为入口。**命令语法、参数、退出码属 CLI Contract SSOT（`contracts/cli/commands.md`），本节只述功能模块划分与责任边界，不重复命令语法。**

### 6.1 已实现功能模块清单

| 模块 | 职责 | 命令契约见 commands.md |
|---|---|---|
| 项目生命周期 | 世界初始化与信息查询 | `init`、`info` |
| 设定与约束 | 设定 KV 管理、受控视图渲染、约束定义/渲染/列表、规则检查绑定 hash | `setting`、`constraint` |
| 章节内容 | 章节创建/更新/读取/列表/重导入/状态机推进/上下文包 | `chapter`（含 `status`、`context`） |
| 主线落地裁决 | 章级主线落地裁决写入，绑定正文 hash | `verdict` |
| 卷管理 | 卷创建/查询/封卷/归档/卷目标，卷状态机 | `volume` |
| 命运编织 | 主线节点、线索绑定、涌现状态机 | `mainplot`、`thread`、`emergence` |
| 归档导出 | 卷/章/全量导出、完整性校验 | `export`、`verify` |

### 6.2 三组承载命令

`setting render --view`、`constraint render --view/--scope`、`chapter context --view` 三组受控 view/scope 承载命令。受控 view/scope 名及其机器语义见 `contracts/cli/views-scopes.md` §A（SSOT）；三组命令的实现状态见 `repository/dev-progress.md` §4。

### 6.3 设计语义（已抽离）

以下设计语义不含命令语法，已抽离至独立文件：

| 设计语义 | 抽离至 |
|---|---|
| 章契约三档约束（must/may_deviate/encourage_surprise） | `docs/设计亮点-章契约与上下文包.md` |
| Chapter Context Packet 五层×五优先级矩阵、预算压缩、audit/overflow | `docs/设计亮点-章契约与上下文包.md` |
| 涌现状态机、评分四维度、decide 动作、卷末处置 | `docs/设计亮点-涌现与命运编织.md` |
| 章节/卷/创作周期状态机、权限矩阵、hash/stale 保护 | `docs/设计亮点-状态机与门禁体系.md` |
| setting 分层渲染、thread 索引外化、滚动规划窗口 | `docs/设计亮点-设定分层与滚动规划.md` |

### 6.4 规划态未实现设计实体

需求说明书与早期设计文档列出的部分创作对象、生命周期命令与适配命令（事件、小节、残余命令、四相日志、角色、支线、伏笔、爽点、承诺、creation-cycle、great-cycle、archive 独立命令、verdict get、setting/constraint import、view summary/entity/graph、serve 等）当前为规划态。

> 完整未实现实体清单（含前缀登记/spec 状态/设计来源）见 `repository/dev-progress.md` §7.2，设计覆盖面反推对照见同文件 §7.0。Spec 运行层搭建时不得假设这些实体可用。

---

## 7. 状态机与硬阻断

状态机语义（章节 outline→drafting→review→approved、涌现 pending→rated→decided→executed→archived、卷 active→sealing→sealed→archived、创作周期开天→编织→运行→封典→完成）、权限矩阵、hash 检测与 stale 保护的设计已抽离至 `docs/设计亮点-状态机与门禁体系.md`。

门禁清单与转换约束以 `contracts/cli/state-machine.md` 为 SSOT，本文不重复。任何硬阻断 MUST 以非零退出码 + 结构化错误对象返回，不得静默降级。

---

## 8. 权限矩阵

权限矩阵（操作域×可改/禁改对象）的设计见 `docs/设计亮点-状态机与门禁体系.md` §6。

---

## 9. 章节上下文包

`chapter context` 命令的设计（三种视图、五层×五优先级矩阵、预算规则、压缩阶梯、audit/overflow、诚实省略机制）已抽离至 `docs/设计亮点-章契约与上下文包.md`。命令契约见 `contracts/cli/commands.md`；受控 view 名见 `contracts/cli/views-scopes.md` §A 组 4；输出结构见 `contracts/cli/json-schemas/chapter-context.schema.json`。

---

## 10. 复杂度评估

`creation-cycle complexity` 的设计（scope 维度、复杂度等级、blocking 阻断）依赖 `creation-cycle` 命令族。"硬阻断"与 complexity 的 `blocking` 关系统一：复杂度评估达到 blocking 且影响当前推进时，CLI 仍通过统一错误对象和非零退出码硬阻断。

> `creation-cycle` 命令族与 complexity 相关实体的规划态清单见 `repository/dev-progress.md` §7.2。

---

## 11. 输出、错误与退出码

### 11.1 输出格式

查询类、读取类与导出类命令必须同时支持人读输出（默认，文本/表格/分段摘要/Markdown）与机器输出（显式 `--json`，结构稳定、字段完整、可解析、可追踪）。JSON 输出约束：必须含 `schemaVersion`；必须含实体 id/type/状态字段/引用字段/必要时间戳；不得混入 ANSI 颜色/解释性旁白/非结构化提示；长文本提供 view/summary/chunk/budget/reference-only 模式；派生视图保留 sourceRefs；省略与压缩提供 audit/overflow。

### 11.2 错误对象

**默认（V2）**：JSON 模式下 `--json` 失败输出为 V2 错误信封 `V2ErrorEnvelope`，结构字段：`schemaVersion`（`error-protocol/v2`）、`status`、`errorId`（结构化错误 ID，稳定字符串）、`category`（错误类别）、`exitCode`（类别派生退出码）、`message`（人读说明），可选 `details`（结构化附加）。`exitCode` 取值与语义以 `contracts/cli/exit-codes.md` 为唯一权威，错误 ID 以 `contracts/cli/error-codes.md` 为唯一权威。

> **V1 弃用兼容路径**：V1 `ErrorObject`（`code`/`message`/`exitCode`/`suggestion`，不携带 `schemaVersion`）仅在显式选择 V1 时输出，逐字保持 V1 回归基线。默认化、兼容期与 V1 移除准则见 `repository/dev-progress.md` §7.2 与 `docs/契约治理-生成式契约源迁移.md` §6.5，本文件不复制错误对象字段表。

### 11.3 退出码

退出码语义的唯一事实来源是 `contracts/cli/exit-codes.md`（SSOT）。任何文档不得自抄退出码语义表，须引用 SSOT。任何硬阻断 MUST 以非零退出码返回，不得静默降级。

> 错误码字符串语义以 `contracts/cli/error-codes.md` 为 SSOT；JSON 输出结构以 `contracts/cli/json-output.md` 为 SSOT。

---

## 12. 测试怎么做

- **CLI 集成测试**：初始化世界；创建设定/约束/主线/卷/章节；正常状态流转；裁决缺失/hard 违规时进入 review 被拒绝；approved 章节修改被拒绝；hash 不一致时读取/推进/归档被拒绝；涌现跳步被拒绝；卷目标未达成时封卷被拒绝；涌现未处置时封卷被拒绝；非封典阶段终检被拒绝。
- **数据一致性测试**：命令失败后数据库和正文文件不被污染；临时文件失败清理；并发写入不产生脏数据；导出数据可审计；目标状态变化后旧评估不得继续作为推进依据。
- **契约与封装测试**：面向 Spec 的 JSON 输出符合 schema；golden examples 稳定；Spec mock 能消费 CLI 输出并捕获非零退出码中断；外部代码无法通过公开包入口调用内部模块；CLI 内部存储变更不影响 CLI Contract。

> 契约测试位置与覆盖见 `contracts/cli/contract-tests/`；生成式测试见 `crates/spec-gen` 与 `contracts/cli/generated/`。

---

## 13. 与其他文档的关系

- `docs/项目整体架构.md`：项目级分层、优缺点、技术选型、能力边界和功能边界。
- `docs/Spec架构设计.md`：Spec 数据流转与 LLM 切入点。
- `docs/设计亮点-*.md`：五份设计语义独立文件（状态机、章契约、涌现、写作流水线、设定分层）。
- `docs/CLI模块对齐清单.md`：模块职责说明（docs 侧补充）。
- `docs/需求说明书.md`：系统需求与验收边界。
- `repository/cli-modules.md`：模块职责稳定知识库。
- `repository/storage-model.md`：存储模型与一致性保障。
- `contracts/cli/`：CLI 契约 SSOT。
- `CLAUDE.md`：项目执行约束与实现指令。

---

**文档版本**：2.0
**最后更新**：2026-08-11
**变更说明**：v2.0 文档对齐治理——设计亮点抽离至 5 份独立文件，§6 命令设计（原 ~80 条命令语法串）整节重写为"功能模块清单+契约引用"，主文档激进瘦身（843→约 200 行）。
