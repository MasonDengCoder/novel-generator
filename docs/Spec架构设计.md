# 小说创作工具系统 — Spec 架构设计

> 本文描述 Spec / Claude Code 软层的数据流转、LLM 角色，以及 LLM 在哪些位置通过 `novel` CLI 读取或写入世界状态。设计亮点（写作流水线、章契约与上下文包等）已抽离至 `docs/设计亮点-*.md`；项目级分层见 `docs/项目整体架构.md`；CLI 硬层见 `docs/CLI架构设计.md`；命令契约以 `contracts/cli/commands.md` 为 SSOT。

---

## 1. Spec 定位

Spec 是世界运行引擎，也是 Claude Code LLM harness 控制层。

Spec 不保存世界状态本身，而负责：把《创作即创世》方法论转译为可执行流程；在正确阶段组织 prompt、上下文包、任务列表、验证标准和修复流程；调度 writer/verifier/repair/extractor/monitor 等 LLM 执行实例；通过 `novel` CLI 查询权威状态、提交创作结果、触发硬层校验；在 CLI 拒绝时停止流程，进入修复、补写、回退或人工处理。

Spec 的核心价值不在于"记住世界"，而在于"按方法论正确调度 LLM 与 CLI"。

> Spec 运行层落地状态见 `repository/dev-progress.md` §7.1。CLI 契约面（`contracts/cli/`）已稳定可消费，Spec 落地时以 CLI 契约 + `repository/dev-progress.md`（含命令实现矩阵与未实现实体清单）为对照。

---

## 2. 数据权威原则

### 2.1 状态来源

```text
SQLite + 正文文件
  ▲
  │  只能由 novel CLI 读写
  │
novel CLI
  ▲
  │  --json / 非零退出码 / 结构化错误
  │
Spec / Claude Code / LLM
```

权威状态只存在于 CLI 硬层接受后的世界日志中。Spec 运行过程中产生的所有内容，在写入 CLI 并被接受前，都只是中间产物。

### 2.2 中间产物

Spec 可以产生：prompt、Chapter Context Packet、Chapter Task List、章节正文草稿、verifier 报告、repair 建议、residual/phase-log/emergent 候选、复杂度解释与修复建议。这些内容只有通过明确的 `novel` 命令写入并通过 CLI guard 后，才成为世界状态。

### 2.3 禁止事项

Spec 必须遵守以下边界：

1. 不存储创作状态作为权威来源。
2. 不直接读取或修改 SQLite。
3. 不直接读取或修改 `.novel/content` 下的正文文件作为状态来源。
4. 不解析 `.novel/` 内部路径、SQLite schema、migration 文件或 Rust module 名称。
5. 不 import / require / link CLI 内部模块。
6. 不自行判断章节是否可进入 `review` / `approved`。
7. 不自行判断卷是否可关闭大轮回、归档或封典完成。
8. 不把 prompt 输出、agent 记忆、markdown 缓存当成权威世界状态。
9. 不忽略 CLI 非零退出码继续推进流程。

---

## 3. 总体数据流

### 3.1 读流程

```text
Spec 确定当前阶段和目标
  │
  │  通过 CLI 契约命令（--json）读取权威状态
  ▼
CLI 读取权威状态并执行 hash / 权限 / schema 检查
  │
  │  stdout JSON + stderr 诊断 + exit code
  ▼
Spec 解析 schemaVersion、实体 ID、sourceRefs、状态字段
  │
  ▼
组织 LLM prompt / Context Packet / Task List
```

读流程约束：面向机器消费的命令必须使用 `--json`；Spec 必须检查退出码；Spec 必须检查 `schemaVersion`（检测到不支持的版本时停止后续推进，不静默降级、不猜测语义，见 `contracts/cli/json-output.md` §2 SSOT）；Spec 不得依赖人读文案；Spec 不得从 CLI 输出中的内部路径推断存储布局。

### 3.2 写流程

```text
LLM / Spec 产生候选结果
  │
  │  写入临时草稿或准备命令参数
  ▼
Spec 通过 CLI 契约写命令提交候选
  │
  ▼
CLI 执行 guard、transaction、hash、状态机、权限矩阵
  │
  ├─ 成功：返回稳定 JSON 或人读摘要
  │
  └─ 失败：返回非零退出码 + 结构化错误 + suggestion
```

写流程约束：Spec 只提交候选，不绕过 CLI 写状态；CLI 拒绝时 Spec 必须停止后续动作；失败后不得用缓存状态假装写入成功；对正文写入，Spec 只能通过 `chapter update` 命令，不能直接写 `.novel/content`。

### 3.3 CLI 拒绝处理

当 CLI 返回非零退出码时，Spec 必须：停止当前后续动作；读取结构化错误码、原因、`suggestion`；判断进入 repair、补写缺失数据、回退阶段，还是交给用户处理；不得忽略失败继续推进状态。

---

## 4. LLM 在 CLI 的切入点（Spec 功能模块）

LLM 不直接接触 SQLite 或正文存储，与世界状态的交互只发生在受控切入点。每个切入点通过对应 CLI 契约命令完成（具体命令见 `contracts/cli/commands.md`）。

| 切入点 | 职责 | 使用的 CLI 契约命令（见 commands.md） |
|---|---|---|
| **4.1 写前读取** | 让 LLM 获得本章写作所需的受控上下文 | `setting render --view`、`constraint render --view/--scope`、`chapter get`、`chapter context --view writer`、`thread query` |
| **4.2 正文写入** | writer 产出草稿后写入正文 | `chapter update <id> --content <file>` |
| **4.3 写后确定性检测** | 正文写入后触发确定性检测 | `constraint check --chapter`、`chapter get` |
| **4.4 独立验证** | verifier 读取 verifier view 与正文机器视图，写入主线落地裁决 | `chapter context --view verifier`、`chapter get`、`verdict set-mainplot-landing` |
| **4.5 修复** | verifier 或 CLI 检测失败时读取 repair view，只修复失败项 | `chapter context --view repair`、`chapter get`、`chapter update` 重写 |
| **4.6 状态回写** | 章节通过基本验收后回写结构化状态 | emergence/thread 命令组；residue/phase-log/event/foreshadowing/character 命令 |
| **4.7 状态推进** | 所有结构化状态补齐后请求推进章节状态 | `chapter status review`、`chapter status approved` |

LLM 不得：直接补读 `.novel/content`；要求 Spec 绕过 CLI 读 SQLite；根据内部文件路径推断更多状态。

> 各切入点的数据流细节、writer/verifier/repair 三角色职责、修复循环与 hash/stale 失效规则已抽离至 `docs/设计亮点-写作流水线与写验分离.md`；Chapter Context Packet 的设计语义见 `docs/设计亮点-章契约与上下文包.md`。

---

## 5. 章节写作数据流

标准章节写作流水线（15 步）、写前/写作/验收/修复四阶段、写验分离原则、裁决存在性门禁、修复循环回退路径（review→drafting 最小修复 / review→outline 章契约重做）的设计已抽离至 `docs/设计亮点-写作流水线与写验分离.md`。

---

## 6. Chapter Context Packet 数据流

Chapter Context Packet 的输入来源、输出结构、三种视图差异、audit 与 overflow 处理、压缩与省略规则的设计已抽离至 `docs/设计亮点-章契约与上下文包.md`。命令契约见 `contracts/cli/commands.md`；受控 view 名见 `contracts/cli/views-scopes.md` §A 组 4；输出结构见 `contracts/cli/json-schemas/chapter-context.schema.json`。

Spec 不得自行重排、删除、重压缩或补全 canonical packet 的 P0/P1 片段；不得用缓存、对话记忆或推测内容补全 omitted 片段。若 CLI 因预算、hash、stale check 或 stale verdict 返回非零退出码，Spec 必须停止当前推进，进入拆章、补查、repair、重新检测或人工处理路径。

---

## 7. 卷末与封典数据流

卷末流程依赖 `great-cycle` 命令族与 `archive verify` 封典终检命令（落地状态见 `repository/dev-progress.md` §7.2）。

卷末流程读取卷状态、事件列表、pending 涌现、复杂度评估，LLM 可生成卷末监测报告、支线状态快照建议、涌现处置建议、未闭环项解释，写入仍通过 CLI emergence/volume 命令完成。封典阶段执行终检，`archive verify` 的 PASS/FAIL 由 CLI 返回，Spec 不得自行替代。

---

## 8. Web / view 数据流

`serve` 与 `view summary/entity/graph` 命令的落地状态见 `repository/dev-progress.md` §7.2。

Web 或后续可视化层与 Spec 一样，不得直接访问 SQLite 或 `.novel/content`。`view` 和 `serve` 必须复用 CLI service/use-case 与 guard；默认只读；若未来开放写入，也必须走与 CLI 命令同等的状态机、权限矩阵与硬阻断。

---

## 9. 反模式

以下做法禁止：

1. Spec 直接读取 SQLite。
2. Spec 直接读取 `.novel/content` 作为状态来源。
3. LLM 根据内部文件路径追读状态。
4. Skill 把运行时 markdown 文件当作权威创作状态。
5. Subagent 直接修改数据库或正文文件。
6. CLI 调度 LLM 或决定 prompt 编排。
7. writer 自己裁决主线是否落地。
8. context 超限时静默截断。
9. 自行重新排序、重压缩或删除 P0/P1 片段。
10. 把 omitted 片段当作"不存在"或用对话记忆自行补全。
11. approved / archived 后继续直接修改正文。
12. 验收失败时先更新角色、事件、伏笔等结构化状态。
13. CLI 非零退出后继续执行后续状态推进命令。

---

## 10. 与其他文档的关系

- `docs/项目整体架构.md`：项目级分层、优缺点、技术选型、能力边界和功能边界。
- `docs/CLI架构设计.md`：CLI 硬层职责与模块清单。
- `docs/设计亮点-*.md`：五份设计语义独立文件（写作流水线、章契约、涌现、状态机、设定分层）。
- `docs/需求说明书.md`：系统需求与验收边界。
- `reference/参考-ClaudeCode原语能力边界.md`：Claude Code 四原语能力边界事实依据。
- `repository/dev-progress.md`：CLI 实现进度（命令矩阵 + 未实现实体清单）。
- `contracts/cli/`：CLI 契约 SSOT。
- `CLAUDE.md`：项目执行约束与实现指令。

---

**文档版本**：2.0
**最后更新**：2026-08-11
**变更说明**：v2.0 文档对齐治理——写作流水线与 CCP 设计抽离至 `docs/设计亮点-*.md`，§4 LLM 切入点改为"功能模块清单+契约引用"剥离命令语法，§7/§8 标注规划态，新增 §1 Spec 当前实现状态。主文档激进瘦身（591→约 170 行）。
