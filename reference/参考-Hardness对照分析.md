# Hardness 实践框架 ↔ novel-generator 对照分析

> **定位**：本文把 `Hardness-Practice-v3.md`（v3.1，位于仓库根目录）的硬约束工程框架，对照映射到 novel-generator 的系统设计，用于判断当前设计在 Hardness 视角下的硬度分布、升级点与剩余软约束盲区。本文是**分析性参考**，不是设计基线；架构实现细节以 `docs/项目整体架构.md`、`docs/CLI架构设计.md`、`docs/Spec架构设计.md` 与 `docs/需求说明书.md` 为准。
>
> **当前状态声明**：本文写于 2026-06-25，当时项目处于纯文档/规划阶段。此后 CLI 硬层已进入实现推进阶段——`novel` CLI 已实现 46 条命令、SQLite 存储、状态机/权限/hash/事务/退出码等硬阻断能力（实现进度见 `repository/dev-progress.md`）。本文的 Hardness 框架对照分析与"状态层硬 STOP > 工具层硬 STOP"等核心论断仍然成立，但文中凡称"命令族当前均未实现""纯规划阶段"之处均为写作时点状态，**请以 `repository/dev-progress.md` 与 `contracts/cli/` 的当前实现状态为准**，不要据本文判断某命令是否已实现。文中标记为"盲区"的 Spec 流程层软 STOP、Hook 原语落地等项，部分仍为规划态。

---

## 0. 与其他文档的关系

- `Hardness-Practice-v3.md`（仓库根）：被对照的外部框架，定义四层约束模型、证据系统、STOP 规范、四原语映射。
- `docs/项目整体架构.md`：定义软硬分层、技术选型、能力边界——本文分析的分层依据。
- `docs/CLI架构设计.md`：定义硬层状态机、权限矩阵、hash、错误码——本文 L3/L4 落点依据。
- `docs/Spec架构设计.md`：定义软层数据流转与 LLM 切入点——本文 L1/L2 落点依据。
- `docs/需求说明书.md`：定义系统必须满足什么——本文验收对照依据。
- `reference/参考-ClaudeCode原语能力边界.md`：四原语能力边界事实依据——本文原语分配判据来源。
- `CLAUDE.md`：执行约束与实现指令，第 12 节"禁止的自我合理化"与本文第六节直接对应。

---

## 1. 核心论断

**novel-generator 本质上是把 Hardness v3.1 的"四层约束模型"从"提示词/文档层的约束强度"重构为"系统架构层的约束强度"：L1-L2 留在 Spec 软层，L3-L4 下沉到 Rust CLI 硬层。**

它不是又一个把规则堆进 `openspec/config.yaml` 的朴素 Hardness 项目，而是把硬约束做成一个**独立于 LLM harness 存在的确定性内核**。这一选择是 Rust + SQLite 技术选型的深层理由——不是单纯技术偏好，而是 Hardness 硬度要求倒逼：当软约束不足以覆盖长程世界状态一致性时，硬约束必须下沉到一个不依赖 Claude Code 会话的内核。

---

## 2. 四层模型对照

| Hardness 层 | 强度 | Hardness 默认实现 | novel-generator 目标落点 | 升级点 |
|---|---|---|---|---|
| L1 Soft Guidance | 极低 | explore guidance（Rule） | Spec 方法论转译、Chapter Context Packet 的 P2/P3 压缩、章契约 `encourage_surprise` / `may_deviate` | 软引导仍由 Spec 承担，但作用域由 CLI 按预算裁剪 |
| L2 Structured Requirement | 中 | proposal/specs 规则（Rule） | 章契约三档结构、`setting/constraint render` 的 view/scope | 结构化要求由 CLI canonical packet 强制产出，Spec 不得自行重排 |
| L3 Hard Constraint | 高 | tasks/apply 规则（Rule，软） | CLI 状态机 + 权限矩阵 + hash + 事务 + 非零退出码 | 从"模型自觉遵守"升级为"代码硬阻断" |
| L4 Gate/Stop | 最高 | archive 规则 + 可选 Hook | `drafting→review` / `review→approved` / 封卷 / 封典门禁 | 从"软 STOP + 可选 Hook"升级为"状态层硬 STOP" |

关键张力：Hardness v3.1 自己承认 Rule 是"软（可忽略）"、Hook 是"硬（无法绕过）"。novel-generator 把 L3-L4 几乎全部搬出 Rule/Hook，搬进 `service/guard` 的 Rust 代码 + 非零退出码。

---

## 3. 硬 STOP 形态升级：状态层 > 工具层

这是对照中最关键的一点。

**Hardness v3.1 的硬 STOP** 绑在 Claude Code `PreToolUse` Hook 上——拦截的是"Claude 要不要执行这条 Bash 命令"。局限：只在 Claude Code 会话内生效，人直接跑命令就绕过；Hook 无状态、不能跨会话。

**novel-generator 的硬 STOP** 在 CLI 内部 `service/guard`——即使 Spec 执行了 `novel chapter status <id> review`，CLI 也会返回退出码 3/5 拒绝。它保护的是**世界状态本身**，不是某个工具调用。

| 维度 | Hardness Hook 硬 STOP | novel-generator CLI 硬 STOP |
|---|---|---|
| 拦截对象 | 工具调用 | 状态推进 |
| 是否依赖 harness | 是（Claude Code 不在即失效） | 否（任何调用者：Spec/Web/脚本/人，同等约束） |
| 状态 | 无状态 | SQLite + hash 持久化 |
| 不可绕过性 | 模型无法绕过 | 模型无法绕过 + 人也无法通过命令绕过 |
| 唯一缺口 | 跨会话、非会话调用 | OS 层直接改 `.novel/` 文件 |

novel-generator 用 **hash 检测 + 拒绝推进**补上"OS 层直接改文件"这个缺口——这正好是 Hardness Hook"无状态、不能跨会话"缺口的反面解法：CLI 把状态写进 SQLite，下次任何读取/推进/归档前重算 hash，发现不一致就返回非零退出码与结构化错误码（预留错误码 `CONTENT_HASH_MISMATCH` 见 `contracts/cli/error-codes.md` §B，对应切片实现前不生效）阻断。这是"事后检测 + 拒绝继续"而非"事前阻断"，但对长程世界状态是正确的取舍——无法事前拦住人改文件，但能保证**带病状态推不进下一章**。

---

## 4. 证据系统升级：从"手写 verify.md"到"机器绑定 hash"

Hardness 证据四要素：命令、输出、时间戳、位置。本质是 AI 把证据手写在 `verify.md`——**可伪造、可遗漏、仍是软的**。

novel-generator 的"证据"机制把证据锚定到客观文件状态：

| Hardness 证据 | novel-generator 对应 | 硬度提升 |
|---|---|---|
| 手写命令+输出 | `content_hash` 绑定裁决、绑定规则检查 | 正文改了 → hash 变 → 旧裁决自动 stale |
| 时间戳 | append-only verdict（chapter / node / hash / achieved / evidence / reason / verifier / ctime） | 不可篡改，每次 verifier 留痕 |
| 证据位置 | `sourceRefs` 指向稳定实体 ID（非文件路径） | 跨 schema 演进仍可追溯 |
| "证据缺失视为未完成" | "裁决 stale 视为无裁决，不得进 review/approved" | 同一原则的硬层落地 |
| verify.md 的 Evidence 章节 | `audit` / `overflow` manifest | 机器生成，非人手记录 |

核心升华：Hardness 的"证据缺失视为未完成"靠 AI 自觉核对；novel-generator 的"裁决 stale 视为未完成"靠 CLI 在状态推进时校验 `content_hash` 匹配——**verifier 没法拿旧正文的裁决冒充新正文**。这用 hash 把"LLM 的话术"锚定到了"客观文件状态"，是 Hardness 证据系统没能解决的"证据可信度"问题。

---

## 5. STOP 三层体系与最大盲区

novel-generator 实际形成三层 STOP，比 Hardness 两层更细：

| 层 | 机制 | 强度 | 保护对象 |
|---|---|---|---|
| ① 世界状态硬阻断 | CLI guard + 非零退出码 | 最硬（不可绕过） | 运行时世界状态 |
| ② 流程软 STOP | Spec 消费 CLI 退出码后停止推进 | 中（依赖 Spec 守纪律） | 多步流程的顺序 |
| ③ 开发期软 STOP | CLAUDE.md / openspec/config.yaml | 软 | 造 novel-generator 的开发过程 |

### 盲区 A（最大）：Spec 流程层软 STOP 残留

CLI 硬阻断只保护"单条状态推进命令"。但 Spec 是 15 步流水线（预检→上下文→写作→写入→检测→验证→修复→回写→推进→归档）。如果 LLM 在某步 CLI 报错后自我合理化跳过，CLI 拦得住"这条 `chapter status review` 被拒"，**但拦不住 Spec 跳到下一章、或用缓存状态假装写入成功**。

设计文档已列出对应反模式（Spec架构设计.md §9："CLI 非零退出后继续执行后续状态推进命令""失败后不得用缓存状态假装写入成功"）——说明设计者已意识到。但这仍靠 Rule 约束，不是硬机制。**按 Hardness v3.1"高风险操作不要只依赖软 STOP"的判据，这是 novel-generator 在 Hardness 框架下仍存在的软约束残留。**

可选补法（供实现期讨论，非当前决策）：

- Spec 运行层加 `PostToolUse` Hook：上一条 `novel` 命令退出码非零则禁止下一条推进类命令；
- 流程骨架用确定性运行时编排（非 LLM），只在创作节点调 LLM；
- CLI 提供"续传令牌"：Spec 须显式 acquire，CLI 校验上一章 `approved` 才发下一章 context。需求说明书 §4.4 提到"断点续传，从最早未完成章节继续"，但未明确这是 CLI 强制还是 Spec 自觉——**这是个需要锁定的硬度点**。

---

## 6. 防止自我合理化：结构封堵 vs 话术封堵

novel-generator 的封堵是**结构性的**（不同 LLM 实例 + 命令分离 + hash 绑定），比 Hardness 的"在 CLAUDE.md 写禁止话术"高一个量级。CLAUDE.md 第 12 节直接照搬了 Hardness 的反合理化清单（"虽然规则说不要 X，但这次情况特殊""这只是临时的""用户可能想要…""这样更快…"）——这是开发期纪律，与运行期结构封堵分层互补。

| Hardness 合理化话术 | novel-generator 的结构性封堵 |
|---|---|
| "我看过代码，应该没问题" | 写验分离：writer 不得自评主线落地，必须独立 verifier + `verdict set-mainplot-landing` |
| "基本完成，可以先推进" | `drafting→review` 前置条件全硬阻断（裁决 achieved=1 + hard 无违规 + hash 一致） |
| "虽然正文变了但旧裁决还能用" | 裁决绑定 `content_hash`，正文变则 stale |
| "这个 must 目标差不多达成了" | 章契约 `must` 不可静默降级 |
| "改动小不需要测" | 章级命令禁止改主线/卷目标（权限矩阵） |

---

## 7. 四原语分配判据对照

依据 `reference/参考-ClaudeCode原语能力边界.md` 的判据：

| 原语 | Hardness 判据 | novel-generator 分配 | 对齐 |
|---|---|---|---|
| Rule | 跨阶段不变式，忘了就崩，≤200 行 | CLAUDE.md / openspec/config.yaml 开发期不变式 | ✓ |
| Hook | 必然发生的确定性机械动作 | **未展开（见盲区 C）** | ⚠ 空白 |
| Skill | 某阶段触发的多步流程 | Spec 阶段编排（开天/编织/运行/封典） | ✓ |
| Subagent | 读完即弃、撑爆主上下文的监测/校验 | 独立 verifier | ✓ 且升华 |

### verifier = Subagent 是对 Hardness v3.1 的架构级升华

Hardness v3.1 §"证据验证的实现：委托给 Subagent"列出三类适用场景（证据缺失检查、三维一致性核对、Archive 前门禁扫描）。novel-generator 的独立 verifier 是这条判据的运行期落地：独立上下文、最小权限、读完即弃。

但 novel-generator 的 verifier 要写裁决（状态写），而 Subagent 原语"不拥有状态写权限"——解法是 verifier 只通过 `novel verdict set-mainplot-landing` 提议，CLI 才接受。**状态写权限被收束到 CLI，Subagent 只是提议者**，完美尊重原语边界。

进一步：Hardness 的 verify 可能是同一 agent 自检，novel-generator 强制 writer ≠ verifier 不同实例，从源头杜绝"自评"。

---

## 8. 两套阶段，各归各位

novel-generator 有**两套阶段**，Hardness v3.1 的"按 OpenSpec 阶段配硬度"只覆盖了其中一套：

### A. 开发期（造 novel-generator 本身）

走 OpenSpec，`openspec/config.yaml` 已按 proposal→specs→design→tasks→apply→verify→archive 配了 L1-L4，且每条都诚实标注"当前项目无构建命令""命令不存在则保持未勾选"。这层完全套用 Hardness v3.1，软 Rule 合适。

### B. 运行期（用 novel-generator 写小说）

走创作周期 `开天→编织→运行→封典→完成` + 章节级 `outline→drafting→review→approved`。这层 Hardness 没有对应物——它是"世界状态机流转"，硬度从"软规则"升级到"代码硬阻断"。

对照 Hardness 的 archive（最高硬度，"tasks 未完成必须 STOP"）在运行期对应**封卷/封典门禁**：卷目标未达成不得封卷、涌现未处置不得封卷、`archive verify` 返回 PASS 才能封典完成。**同一个"防止错误固化"原则，但 Hardness 是软 STOP + 可选 Hook，novel-generator 是 CLI 非零退出码硬阻断。**

章节状态机本身就是"带 Gate 的责任单元流水线"：

- `drafting→review` 前置条件 = Hardness"进入 verify 前证据齐备"；
- `review→approved` = Hardness"archive 前门禁"；
- `approved` 终态不可改 = Hardness"归档后不可改"。

---

## 9. 用 Hardness 六条禁忌审视

| 禁忌 | novel-generator 表现 | 判定 |
|---|---|---|
| 1. 探索阶段不过度 Hardness | `encourage_surprise` / `may_deviate` / 涌现候选给创作留软空间；P2/P3 可压缩、P4 进 overflow | ✓ 符合 |
| 2. 不全写成 MUST | 章契约三档、规则 hard/soft、complexity 四级（low/medium/high/blocking） | ✓ 分级清晰 |
| 3. 不用错误事实做硬规则 | CLI Contract 强调已验证事实；config.yaml 标注"假设命令尚不存在" | ✓ 符合 |
| 4. 不让 Hard Rules 互相冲突 | 权限矩阵写死"章节级命令禁止改主线/卷目标" | ✓ 但有张力（见下） |
| 5. 不把 Hardness 当替代验证 | 独立 verifier + 确定性检测 + contract tests + golden examples | ✓ 符合 |
| 6. 不破坏可维护性 | 章是"正文/验证/残余/裁决最小闭环单元"；滚动批次 1-3/1-5 章 | ✓ 粒度合理 |

### 禁忌 4 的张力点

滚动规划"下一卷半细、远期粗"与"卷目标是冻结/半冻结封卷门禁来源"并存。若远期卷目标后来要调整，会和"冻结主线不可改"冲突。设计用"半冻结" + `may_deviate` 留了口子，**但这个边界要靠 CLI 权限矩阵精确实现**，否则就是 Hardness 说的"冲突规则让模型无所适从"——这是实现期要盯死的点。

---

## 10. 盲区清单（不止唱赞歌）

### 盲区 A：Spec 流程层软 STOP 残留（最大）

见第 5 节。CLI 拦得住单条命令，拦不住 LLM 跳步。需 Hook 或确定性运行时补。

### 盲区 B：verifier 仍是 LLM，achieved 值可被同构失败

hash 绑定解决了"用旧裁决冒充新正文"，但解决不了"这一轮 verifier 本身判断错"。writer/verifier 同模型同 prompt 谱系可能共享盲区。项目整体架构.md §3.2 已承认"部分判断仍需 LLM"——**这是不可硬化的软约束残留，需人类抽检或多 verifier 投票补，不是设计缺陷**。

### 盲区 C：Hook 原语落地未定义

项目整体架构.md §4.3 明确"当前 Spec 文档先不展开完整原语设计"。哪些"必然发生的确定性机械动作"该是 Hook？候选（非决策）：

- 写正文前自动 hash 预检；
- `novel` 命令后自动捕获退出码并注入上下文；
- archive 前自动跑 `archive verify`；
- 上一条 `novel` 非零退出则禁止下一条推进类命令（直接补盲区 A）。

**这些只用 Rule 就是软的，用 Hook 才硬——是从文档走向实现时必须补的原语设计空白**，也是补盲区 A 的关键抓手。

### 盲区 D：config.yaml 的 Rule 本质（Hardness v3.1 核心认知）

Hardness v3.1 强调"config.yaml rules 是通过 Skill 间接实现的 Rule，要强制拦截仍需 Hook"。novel-generator 已内化这条：**开发期软约束放 config.yaml，运行期硬约束放 CLI 代码**，各归各位。这是它相对"把所有 Hardness 堆进 config.yaml"的高明之处——但意味着运行期硬度**不能靠改 config.yaml 调**，必须改 Rust 代码 + 发版，这是硬约束的代价。

### 盲区 E："无人值守"边界

Hardness 明确 STOP 依赖人类响应，不适合完全无人值守。novel-generator 长流程 + 断点续传，天然倾向半无人/异步。CLI 硬阻断触发时预留了"人工处理"出口（符合 Hardness"人机协同"）。**但若用户期望"一键百万字无人值守"，会和 Hardness 边界冲突**——这是期望管理问题，需在需求层说清。

---

## 11. 元结论

novel-generator 是 Hardness v3.1 理念在"世界状态系统"维度的工程化投影：

| Hardness v3.1 概念 | novel-generator 工程化投影 |
|---|---|
| 四层模型（L1-L4） | Spec 软层（L1-L2）+ CLI 硬层（L3-L4） |
| 证据系统（手写） | content_hash + rules_revision + sourceRefs + audit/overflow + append-only verdict（机器绑定） |
| 责任单元 | 章节 = 正文/验证/残余/裁决最小闭环单元 |
| 防自我合理化（话术封堵） | 写验分离 + 权限矩阵 + 非零退出码（结构封堵） |
| 硬 STOP（Hook，工具层） | 硬 STOP（CLI guard，状态层）——更强形态 |
| 四原语判据 | 已对齐，verifier = Subagent 是升华 |

**Hardness 解决"AI 写代码的漂移"，novel-generator 解决"AI 写小说的漂移"——但小说世界状态比代码 PR 更长程、更易遗忘，所以它不能停在 config.yaml 软约束，必须下沉到 Rust + SQLite 硬层。**

反过来，novel-generator 也给 Hardness v3.1 提供了一条可推广的范式：**当软约束不足以覆盖长程状态一致性时，必须把硬约束下沉到一个独立于 LLM harness 的确定性内核**。Hardness v3.1 的硬 STOP 还绑在 Claude Code Hook 上，harness 不在就没了；novel-generator 的硬阻断绑在世界状态 CLI 上，任何调用者都受约束。**状态层硬 STOP > 工具层硬 STOP**——这是 novel-generator 对 Hardness 框架本身的贡献。

**剩余的软约束残留（Spec 流程跳步、verifier 同构失败）不是靠再加一层 Rule 能解决的，要靠 Hook 原语补全 + 人类抽检**——这正是从"规划文档"走向"实现"时，Hardness 视角下最该优先攻坚的两个点。

---

## 12. 从文档到实现的落地建议

本文是分析参考，不新增实现任务。从 Hardness 视角出发，进入实现阶段时应优先澄清以下硬度点（均待 CLI Contract 与实现状态确认后展开，不假设未实现命令已可用）：

1. **盲区 A 的硬补法**：明确 Spec 流程骨架是确定性运行时编排还是 LLM 驱动；若是后者，定义 `PostToolUse` Hook 的退出码联动规则。
2. **盲区 C 的 Hook 原语清单**：把上节候选逐条判定为 Hook / Skill / Rule / CLI 内置，给出 matcher 与容错降级策略（Hook 失败应 `exit 0` 放行写作，参考原语能力边界.md Hook 判据）。
3. **断点续传的硬度归属**：锁定"从最早未完成章节继续"是 CLI 强制门禁还是 Spec 自觉行为。
4. **禁忌 4 的权限边界**：在 CLI Contract 的权限矩阵中精确刻画"半冻结卷目标"的可改条件，避免与"冻结主线不可改"冲突。
5. **verifier 同构失败的兜底**：在需求层明确人类抽检频率或多 verifier 策略，作为不可硬化软约束的补丁。

以上均为后续阶段的讨论项，不在当前规划文档中形成决策。

---

**文档版本**：1.0
**最后更新**：2026-06-25
**对照来源**：`Hardness-Practice-v3.md`（v3.1，2026-06-24）
