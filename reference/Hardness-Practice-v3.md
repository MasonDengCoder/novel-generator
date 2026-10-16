# Hardness 实践指南 v3

## 基于 OpenSpec + Claude Code 的硬约束工程

> **版本**: v3.1  
> **更新日期**: 2026-06-24  
> **v3.1 变更**: 对齐 Claude Code 四原语定义——明确 config.yaml 是 Rule 的扩展实现、区分软/硬 STOP、补充 Subagent 验证场景  
> **v3.0 变更**: 删减 40% 冗余内容，前置快速开始指南，移除过度设计机制，优化文档结构


## 什么是 Hardness

**Hardness** = 对 AI Agent 输出的**约束强度**，不是"最好这样做"，而是"不满足就不能继续"。

> 类比：LLM 是蛮力十足但方向感差的马，Hardness 就是那副把力气引到正确方向的马具。

**核心目标**：
- 承认 AI 漂移不可避免
- 将准确率从 90% 提升到 98%+
- 让漂移在可控范围内（可观察、可回滚、可追溯）
- 人机协同控制，而非完全无人值守

**不是追求完美，而是务实的质量控制。**


## 快速开始：5 分钟配置

### 最小可行 Hardness

只需配置 2 个文件，立即生效：

**1. openspec/config.yaml（最小配置）**
```yaml
schema: spec-driven
rules:
  apply:
    - "测试失败不得勾选完成"
    - "命令未运行不得勾选完成"
  archive:
    - "tasks 未完成必须告警并 STOP"
    - "verify 有 CRITICAL 项不得归档"
```

**2. CLAUDE.md（最小配置）**
```markdown
## Before Finishing（每次完成前必须检查）
- [ ] 运行相关测试，全部通过
- [ ] 确认 tasks.md 全部勾选

## 危险操作（STOP）
- 涉及数据删除 → STOP，必须人工确认
```

**预期效果**：
- 减少 80% 的"测试跳过"问题
- 减少 90% 的"未完成就归档"问题

**配置时间**：< 5 分钟  
**适用场景**：所有使用 OpenSpec 的项目


## 目录

### 第一部分：核心概念
- [Hardness 的四层模型](#hardness-的四层模型)
- [证据系统](#证据系统)
- [STOP 行为规范](#stop-行为规范)
- [责任单元判断标准](#责任单元判断标准)

### 第二部分：配置指南
- [在 OpenSpec 中配置](#在-openspec-中配置-hardness)
- [在 Claude Code 中配置](#在-claude-code-中配置-hardness)
- [按阶段配置 Hardness](#按阶段配置-hardness)

### 第三部分：进阶实践
- [规则编写规范](#规则编写规范)
- [防止模型自我合理化](#防止模型自我合理化)
- [失效诊断](#失效诊断)
- [渐进式采纳路径](#渐进式采纳路径)

### 第四部分：边界与禁忌
- [适用边界与场景选择](#适用边界与场景选择)
- [Hardness 的禁忌](#hardness-的禁忌)

### 附录
- [完整配置示例](#完整配置示例)
- [真实案例研究](#真实案例研究待补充)
- [常见问题](#常见问题)


---

## 第一部分：核心概念

## Hardness 的四层模型

### 第 1 层：Soft Guidance（软引导）

**适用阶段**：Explore、需求澄清  
**典型用语**：建议、优先、可以考虑  
**Hardness 值**：极低

**示例：**
```yaml
explore:
  guidance: |
    可以提出 2-3 个技术方案并比较优劣。
    不确定的内容请标注 [NEEDS CLARIFICATION]。
```

### 第 2 层：Structured Requirement（结构化要求）

**适用阶段**：Proposal、Specs、Design  
**典型用语**：输出必须包含、按以下结构  
**Hardness 值**：中

**示例：**
```yaml
rules:
  proposal:
    - "Proposal 必须包含 Context、Scope、Non-goals、Depends On、Blocks"
    - "Non-goals 至少列出 3 项明确排除的能力"
  specs:
    - "每个 Spec 必须包含至少一个 Acceptance Scenario"
    - "Acceptance Scenario 必须使用 Given-When-Then 格式"
```

### 第 3 层：Hard Constraint（硬约束）

**适用阶段**：Tasks、Apply、Verify  
**典型用语**：必须、不得、只允许、不满足则  
**Hardness 值**：高

**示例：**
```yaml
rules:
  tasks:
    - "每个 checkbox 必须是独立可执行、可验证、可勾选的责任单元"
    - "测试/验证 checkbox 必须写明命令、范围、期望结果、失败处理"
  apply:
    - "只执行 tasks.md 已列 checkbox"
    - "测试失败不得勾选完成"
```

### 第 4 层：Gate / Stop Condition（门禁）

**适用阶段**：Archive、发布、删除  
**典型用语**：STOP、暂停、必须确认  
**Hardness 值**：最高

**示例：**
```yaml
rules:
  archive:
    - "tasks 未完成必须告警并 STOP"
    - "verify 有 CRITICAL 项不得归档"
  security:
    - "涉及删除文件/目录的操作，STOP，必须人工确认"
```


## 证据系统

### 为什么需要证据

"必须测试通过"不等于已经测试。证据系统用**客观事实对抗 LLM 的语言能力**。

| LLM 的擅长 | Hardness 的对策 |
|-----------|----------------|
| 语言上"完成"任务 | 要求**可观察的证据**（命令输出、时间戳） |
| "逻辑上应该正确" | 要求**实际运行结果** |
| "我已经验证过" | 要求**证据记录位置**（verify.md） |

### 证据的定义

一个合格的证据必须包含：

1. **命令**：完整的执行命令（可复现）
2. **输出**：关键输出内容（成功标志或错误信息）
3. **时间戳**：执行时间
4. **位置**：证据记录在哪里

### 证据记录位置

| 证据类型 | 记录位置 | 格式要求 |
|----------|----------|----------|
| 测试结果 | `openspec/changes/<id>/verify.md` 的 Evidence 章节 | 命令 + 输出摘要 + 时间戳 |
| 类型检查 | `verify.md` | 命令 + "0 error" 或错误数 |
| 手动验证 | `verify.md` | 标注 [MANUAL] + 验证步骤 + 结果 |

### 证据模板

```markdown
## Evidence

### 单元测试
- **命令**: `npm test -- user.service.test.ts`
- **执行时间**: 2026-06-23 14:30:25
- **结果**: 全部通过（12 passed, 0 failed）
- **输出摘要**:
  ```
  PASS  src/services/user.service.test.ts
    UserService
      ✓ should create user successfully (25ms)
      ✓ should validate email format (12ms)
  Tests: 12 passed, 12 total
  ```

### 类型检查
- **命令**: `npm run typecheck`
- **执行时间**: 2026-06-23 14:32:10
- **结果**: 0 error

### 手动验证
- **类型**: [MANUAL]
- **验证内容**: 验证登录页面 UI 一致性
- **步骤**:
  1. 启动开发服务器 `npm run dev`
  2. 访问 http://localhost:3000/login
  3. 检查表单对齐、按钮样式、错误提示位置
- **结果**: 通过（UI 符合设计稿）
```

### 配置规则

```yaml
rules:
  verify:
    - "证据缺失时，该 task 视为未完成"
    - "发现证据缺失时，必须在 verify.md 标注 [EVIDENCE MISSING]"
    - "不得以'已经验证过'代替实际证据"
```

### 证据验证的实现：委托给 Subagent

复杂的证据核对会读取大量内容（多个测试输出、跨文件一致性检查），直接在主上下文执行会撑爆上下文。这类「读完即弃」的校验适合委托给 **Subagent**：

- 隔离执行，独立上下文窗口，不污染主流程
- 只回传结论（哪些 task 缺证据、哪些不一致），不回传原始内容
- 符合原语判据：「读完即弃、会撑爆主上下文的监测/校验 → Subagent」

**适用场景**：
1. **证据缺失检查**：读取 `verify.md`，逐项核对每个 task 是否有命令、输出、时间戳，回传缺失清单
2. **三维一致性核对**：`openspec-verify-change` 的 Completeness / Correctness / Coherence 逐项检查
3. **Archive 前门禁扫描**：核对 tasks 完成度与 verify CRITICAL 项，回传是否可归档

**注意**：Subagent 只做校验和回传，不做强制拦截（拦截是 Hook 的职责）。


## STOP 行为规范

### STOP 的两种实现路径

STOP 有两种实现原语，**触发方式和可绕过性完全不同**，配置时必须区分：

| 类型 | 实现原语 | 触发方式 | 可绕过？ | 适用场景 |
|------|----------|----------|----------|----------|
| **软 STOP** | Rule（CLAUDE.md / config.yaml） | 模型读取规则后主动判断并暂停 | 是（模型可能忽略） | 需求检查、证据缺失、tasks 未完成 |
| **硬 STOP** | Hook（PreToolUse） | 工具调用前被拦截，返回 DENY | 否（模型无法绕过） | 数据删除、生产部署、危险命令 |

**配置建议**：
- 归档、数据删除、`terraform apply` 等高风险且不可逆操作 → **硬 STOP（Hook）**
- 需求边界、证据缺失等需要语义判断的逻辑 → **软 STOP（Rule）**
- 高风险操作不要只依赖软 STOP，模型可能在自我合理化后跳过

### STOP 的触发条件

| 场景 | STOP 条件 | 风险说明 |
|------|-----------|----------|
| **数据删除** | 涉及 DROP TABLE、DELETE 无 WHERE | 不可逆，影响生产数据 |
| **配置变更** | 修改 src/config/ 下文件 | 可能影响所有环境 |
| **归档操作** | tasks 未完成或 verify 有 CRITICAL | 错误固化，后续难以追溯 |

### STOP 后的输出格式

触发 STOP 时，必须输出以下内容：

```markdown
## ⛔ STOP - 需要人工确认

### 触发原因
<具体说明触发了哪条规则>

### 风险说明
<说明该操作的风险和影响范围>

### 需要的决策
- [ ] 确认风险可接受，继续执行
- [ ] 调整方案，避免该操作
- [ ] 暂时跳过，稍后处理

### 如果确认继续
请明确回复："确认继续，我理解风险"
```

### STOP 后的恢复路径

```yaml
rules:
  stop_behavior:
    - "触发 STOP 时必须输出：触发原因、风险说明、需要的用户决策"
    - "STOP 后不得继续执行任何改动操作（读取操作允许）"
    - "用户确认后必须在 verify.md 记录：STOP 原因、用户确认内容、执行时间"
```

### STOP 场景处理矩阵

| 用户响应 | Agent 行动 | 记录要求 |
|---------|-----------|---------|
| **确认继续** | 执行操作 → 在 verify.md 记录确认内容 | [STOP CONFIRMED] |
| **拒绝继续** | 标注 Change 为 [BLOCKED] → 询问调整方案 | [STOP REJECTED] + 理由 |
| **要求回滚** | 恢复到最近 checkpoint | [STOP ROLLBACK] |


## 责任单元判断标准

### 什么是责任单元

**责任单元**：一个可以独立执行、独立验证、独立勾选的最小工作单元。

### 判断标准（客观化）

| 标准 | 说明 | 检验方法 |
|------|------|----------|
| **可观察完成** | 有明确的验证命令或可观察结果 | 能回答"如何用一条命令验证完成？" |
| **文件影响范围** | 影响文件数 ≤ 5 个（优先指标） | 跳过该 task，计算需要修改的文件数 |
| **代码量边界** | 新增/修改代码行数 ≤ 150 行 | 预估改动量（不含自动生成代码） |
| **失败独立性** | 失败时不会连锁影响其他 checkbox | 跳过该 task，其他 task 仍可执行 |

**优先级顺序**：可观察完成 > 文件影响范围 > 失败独立性 > 代码量边界

### 粒度对比

| 类型 | 示例 | 判断 | 原因 |
|------|------|------|------|
| **太粗** | "实现用户模块" | ❌ | 跨越多个文件，无法判断完成 |
| **太细** | "添加 import UserService" | ❌ | 代码行级操作，不是责任单元 |
| **合格** | "实现 UserService.login 方法并通过类型检查" | ✅ | 独立能力 + 明确验证标准 |
| **合格** | "编写 login 方法的单元测试（覆盖成功/失败/边界）" | ✅ | 独立任务 + 明确范围 + 可验证 |

### 常见错误示例

#### ❌ 错误 1：混合 Task
```
[ ] 实现用户登录并编写测试
```
**问题**：开发和测试混合，无法独立验证。

**正确写法**：
```
[ ] 实现 UserService.login 方法，类型检查通过
[ ] 编写 login 方法的单元测试：`npm test -- user.service.test.ts`，期望 12 passed
```

#### ❌ 错误 2：过度细化
```
[ ] 添加 import UserService
[ ] 定义 login 方法签名
[ ] 实现密码验证逻辑
```
**问题**：代码编辑流水账。

**正确写法**：
```
[ ] 实现 UserService.login 方法（包含密码验证、token 生成、错误处理），类型检查通过
```

### 测试任务的特殊要求

测试/验证 checkbox 必须写明：

1. **命令**：完整的执行命令
2. **范围**：测试覆盖的功能点
3. **期望结果**：通过标准（如 "0 error" 或 "12 passed"）
4. **失败处理**：失败时的处理步骤
5. **证据位置**：结果记录在哪里

**示例：**
```
[ ] 单元测试 - UserService.login：`npm test -- user.service.test.ts`，
    期望 12 passed（覆盖成功/失败/边界情况），
    失败时定位错误并修复后重跑，
    结果记录在 verify.md
```


---

## 第二部分：配置指南

## 在 OpenSpec 中配置 Hardness

OpenSpec 的 `config.yaml` 是 Hardness 配置的核心入口。

### 实现机制：config.yaml 本质是 Rule 的扩展

`config.yaml` 不是 Claude Code 的独立原语，它的 `rules` 字段通过 OpenSpec 技能（如 `openspec-task-tracker`、`openspec-apply-change`）读取并注入上下文：

- 技能调用 `openspec instructions <artifact-id> --json` 等命令获取规则
- 规则被注入到该阶段的模型上下文
- 最终效果等同于 **Rule（CLAUDE.md）**——软约束，模型可能忽略

```text
config.yaml rules
    └─ OpenSpec 技能读取（Skill）
        └─ 注入上下文（等效 Rule）
            └─ 模型自觉遵守（软约束）
```

**关键认知**：`config.yaml` 的 rules 是「通过 Skill 间接实现的 Rule」，而非独立的强制机制。要强制拦截，仍需配合 Hook（见下文）。

### 配置三机制

1. **Default Workflow Schema**：定义 artifact 依赖图
2. **Project Context Injection**：注入全局背景信息
3. **Per-Artifact Rules**：为每个 artifact 追加特定约束

### 完整配置示例

```yaml
# openspec/config.yaml
schema: spec-driven
context: |
  项目：用户管理系统（微服务架构）
  技术栈：Node.js + TypeScript + PostgreSQL
  测试框架：Jest

# Proposal 阶段（中硬度）
rules:
  proposal:
    - "Proposal 必须包含 Context、Scope、Non-goals、Depends On、Blocks"
    - "Non-goals 必须列出至少 3 项明确不做的能力"
    - "Scope 必须用用户故事格式：'作为 [角色]，我想要 [能力]，以便 [价值]'"

  # Specs 阶段（中高硬度）
  specs:
    - "每个能力变更必须对应至少一个 Spec 文件"
    - "每个 Spec 必须包含至少一个 Acceptance Scenario"
    - "Acceptance Scenario 必须使用 Given-When-Then 格式"
    - "涉及 API 变更的 Spec 必须包含请求/响应示例"

  # Design 阶段（中硬度）
  design:
    - "Design 必须说明推荐方案和至少 1 个替代方案"
    - "必须说明模块边界和数据/API 归属"
    - "涉及外部依赖时必须说明版本和兼容性"

  # Tasks 阶段（高硬度）
  tasks:
    - "tasks.md 必须按阶段分组：Implementation、Contracts and Docs、Tests and Verification"
    - "每个 checkbox 必须是独立可执行、可验证、可勾选的责任单元"
    - "不得把'实现 X 并测试'写成一个 checkbox"
    - "测试/验证 checkbox 必须写明：命令、范围、期望结果、失败处理、证据位置"
    
  # Apply 阶段（高硬度）
  apply:
    - "只执行 tasks.md 已列 checkbox"
    - "每完成一个 checkbox 前必须验证对应结果"
    - "测试失败不得勾选完成"
    - "无法通过时保持 checkbox 未完成并记录 blocker"

  # Verify 阶段（很高硬度）
  verify:
    - "必须核对 tasks.md 是否全部完成"
    - "必须核对每个测试/验证 task 是否有结果证据"
    - "必须核对实现是否匹配 proposal/specs/design"
    - "证据缺失则视为未完成"

  # Archive 阶段（最高硬度）
  archive:
    - "tasks 未完成必须告警并 STOP"
    - "verify 有 CRITICAL 项不得归档"
    - "归档后必须更新 openspec/changelist.md"
```



## 在 Claude Code 中配置 Hardness

### CLAUDE.md：长期规则宪法

`CLAUDE.md` 是 Claude Code 的长期记忆文件，存放项目级长期有效的工作规则。

**CLAUDE.md 应该放：**
- 项目架构边界
- 常用命令（test / lint / build）
- 长期 Do / Don't
- Before Finishing 检查清单
- 禁止修改的路径或危险操作

**CLAUDE.md 不应该放：**
- 某个 feature 的临时需求
- 今日任务清单
- 一次性 TODO

**示例：**
```markdown
# CLAUDE.md - 项目长期规则

## 项目概述
用户管理系统，微服务架构，Node.js + TypeScript + PostgreSQL。

## 常用命令
- 测试：`npm test -- <pattern>`
- Lint：`npm run lint`
- Build：`npm run build`

## 编码规则（MUST）
- 所有数据库操作必须使用 Repository 模式
- 所有 API 必须包含 OpenAPI 注解
- 敏感信息（密码、token）必须加密存储，不得明文

## 编码规则（MUST NOT）
- 不得直接修改数据库 schema 而不经过 migration
- 不得在 Controller 中写业务逻辑
- 不得使用 `any` 类型

## Before Finishing（每次完成前必须检查）
- [ ] 运行 `npm run typecheck`，0 error
- [ ] 运行 `npm run lint`，0 error
- [ ] 运行相关测试，全部通过
- [ ] 确认只修改了预期文件（`git status`）

## 危险操作（STOP）
- 涉及 `DROP TABLE` 或 `DELETE` 无 WHERE 条件 → STOP，必须人工确认
- 涉及修改 `src/config/` 下文件 → STOP，必须人工 review
```

### 将 CLAUDE.md 转化为硬门禁（可选）

仅靠 `CLAUDE.md` 是"软规则"——模型可能忽略。

要转化为硬门禁，需要配合 **Claude Code Hooks**：

**Policy 示例（Python）：**
```python
# 阻止危险的 terraform apply
def terraform_policy(input_data: ToolUseEvent):
    if not input_data.tool_is_bash:
        return
    command = input_data.command.strip()
    if re.match(r'^terraform\s+apply', command):
        yield PolicyDecision(
            action=PolicyAction.DENY,
            reason="terraform apply 被禁止。请使用 `terraform plan` 预览变更。"
        )
```

**配置 Claude Code Hooks：**
```json
// ~/.claude/settings.json
{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "matcher": "*",
            "command": "devleaps-policy-client"
          }
        ]
      }
    ]
  }
}
```


## 按阶段配置 Hardness

以下配置按阶段说明硬度级别和核心规则，**详细规则请参考上文"完整配置示例"**。

### Explore 阶段（低硬度）
- 目的：发散思考，探索可能性
- 可以提出假设，标注 [ASSUMPTION]
- 不确定内容标注 [NEEDS CLARIFICATION]

### Proposal 阶段（中硬度）
- 目的：控制需求边界
- MUST：包含 Context、Scope、Non-goals、Depends On、Blocks
- MUST：Non-goals 至少 3 项
- STOP：需求过大时先切片

### Specs 阶段（中高硬度）
- 目的：固化可观察行为
- MUST：每个能力变更对应 Spec 文件
- MUST：每个 Spec 至少 1 个 Acceptance Scenario
- MUST：使用 Given-When-Then 格式

### Design 阶段（中硬度）
- 目的：控制方案质量
- MUST：说明推荐方案 + 至少 1 个替代方案
- MUST：说明模块边界和数据/API 归属
- SHOULD：复杂状态机画 Mermaid 图

### Tasks 阶段（高硬度）
- 目的：控制执行计划
- MUST：按 Implementation / Contracts and Docs / Tests and Verification 分组
- MUST：每个 checkbox 是责任单元（参考"责任单元判断标准"）
- MUST：开发和测试拆开
- MUST NOT：写"实现 X 并测试"

### Apply 阶段（高硬度）
- 目的：控制实现纪律
- MUST：只执行 tasks.md 已列 checkbox
- MUST：测试失败不得勾选完成
- MUST：无法通过时保持未完成 + 记录 blocker
- STOP：发现混合 task 时暂停

### Verify 阶段（很高硬度）
- 目的：控制完成判断
- MUST：核对 tasks 是否全部完成
- MUST：核对测试/验证证据（参考"证据系统"）
- MUST：核对实现匹配 proposal/specs/design
- MUST NOT：证据缺失时视为完成

### Archive 阶段（最高硬度）
- 目的：防止错误固化
- MUST：tasks 未完成必须告警并 STOP
- MUST：verify 有 CRITICAL 不得归档
- MUST：归档后更新 tasklist
- STOP：任何未确认的异常


---

## 第三部分：进阶实践

## 规则编写规范

### 用短句，不用长段落

❌ **不好：**
> 实现阶段应该在保证质量的前提下根据任务逐步推进，并结合测试情况判断是否可以完成。

✅ **好：**
> - 只按 tasks.md 已列 checkbox 执行
> - 每完成一个 checkbox 前必须验证结果
> - 测试失败不得勾选完成

### 用"失败时怎么办"定义硬度

❌ **弱：**
> 必须测试。

✅ **强：**
> 测试失败时：
> 1. 定位原因
> 2. 修复
> 3. 重跑该测试任务
> 4. 仍失败则保持 checkbox 未完成
> 5. 记录 blocker

### 用"证据"替代"相信"

❌ **弱：**
> 确认功能正常。

✅ **强：**
> 运行 `<command>`，期望 0 error；
> 在 verify.md 的 Evidence 章节记录命令和结果。

### 区分 MUST / SHOULD / MAY / MUST NOT / STOP

| 类型 | 含义 | 示例 |
|------|------|------|
| **MUST** | 不满足就不能继续 | 测试失败不得勾选完成 |
| **SHOULD** | 默认应该做，可说明例外 | 复杂状态机应画 Mermaid 图 |
| **MAY** | 可选增强 | 简单变更可省略替代方案 |
| **MUST NOT** | 明确禁止 | 不得把测试拆成独立 Change |
| **STOP** | 暂停并询问/反馈 | 目标目录不清时停止并询问 |


## 防止模型自我合理化

### 常见合理化路径

LLM 常见的自我合理化语句：

| 合理化语句 | Hardness 封堵 |
|------------|---------------|
| "这个改动很小，不需要测试。" | 所有改动必须按 tasks 执行，不得跳过测试 |
| "我已经看过代码，应该没问题。" | 必须运行命令，有结果证据 |
| "虽然没有运行命令，但逻辑上是对的。" | 未运行且无跳过理由，不得勾选 |
| "这个 task 基本完成了，可以先勾选。" | 完成标准是证据，不是"差不多" |

### 在 CLAUDE.md 中明确封堵

```markdown
## 禁止的自我合理化（MUST NOT）
- 不得以"改动小"为由跳过测试验证
- 不得以"逻辑上正确"代替实际运行验证
- 不得以"基本完成"为由勾选未验证的 checkbox
- 不得以"代码看起来没问题"代替实际测试
```

### 在 config.yaml 中设置硬约束

```yaml
rules:
  apply:
    - "测试失败时，不得勾选完成，无例外"
    - "命令未运行时，不得勾选完成，无例外"
    - "证据缺失时，视为未完成，无例外"
```


## 失效诊断

### 为什么 Hardness 会失效

1. **规则太软**：用词模糊，缺少失败处理
2. **规则冲突**：多条规则互相矛盾
3. **缺少证据要求**：没有可观察的验证标准
4. **缺少封堵**：未预见模型的合理化路径

### 症状 → 诊断 → 修复表

| 症状 | 可能原因 | 修复建议 |
|------|----------|----------|
| 模型跳过测试 | Apply 规则太弱 | 加强规则 + 增加失败处理路径 |
| 测试未运行就勾选 | 缺少证据要求 | 增加"证据缺失视为未完成" |
| 模型说"改动小不需要测试" | 缺少封堵 | 补充"禁止的自我合理化" |
| tasks 未完成就归档 | Archive 规则太弱 | 增加 STOP 条件 |
| 证据只有"已验证" | 证据格式未定义 | 参考"证据系统"补充模板 |

### 诊断清单

当怀疑 Hardness 失效时，逐项检查：

```markdown
## Hardness 失效诊断清单

### 基础检查
- [ ] `CLAUDE.md` 存在且包含长期规则
- [ ] `openspec/config.yaml` 存在且包含 rules 配置
- [ ] rules 按阶段（proposal/specs/tasks/apply/verify/archive）分组

### 规则质量检查
- [ ] 规则使用短句 bullet，不用长段落
- [ ] 硬规则包含失败处理路径
- [ ] 关键规则绑定了可观察证据
- [ ] 区分了 MUST / SHOULD / MAY / MUST NOT / STOP

### 证据系统检查
- [ ] verify.md 有 Evidence 章节
- [ ] 测试 task 写明了命令和期望结果
- [ ] verify 规则要求"证据缺失视为未完成"

### 封堵检查
- [ ] CLAUDE.md 有"禁止的自我合理化"章节
- [ ] 封堵了"改动小不需要测试"
- [ ] 封堵了"逻辑上正确"代替实际验证
```


## 渐进式采纳路径

### 为什么需要渐进式采纳

一次性配置完整的 Hardness 体系可能：
- 过于严格，影响探索效率
- 规则冲突，导致流程卡死
- 学习曲线陡峭

**建议：分阶段采纳，先解决最痛的问题。**

### 阶段 1：最小可行 Hardness（第 1 周）

**目标**：防止最常见的问题（跳过测试、未完成就归档）

**配置内容**：参考本文开头"快速开始"部分

**预期效果**：
- 减少 80% 的"测试跳过"问题
- 减少 90% 的"未完成就归档"问题

### 阶段 2：补充验证门禁（第 2-3 周）

**目标**：确保验证有证据

**新增配置**：
```yaml
rules:
  verify:
    - "必须核对每个测试/验证 task 是否有结果证据"
    - "证据必须包含：命令、完整输出、时间戳"
    - "证据缺失视为未完成"
  tasks:
    - "测试 task 必须写明：命令、范围、期望结果"
    - "不得把'实现 X 并测试'写成一个 checkbox"
```

**预期效果**：
- 每个验证都有可追溯的执行记录
- 减少"我已经验证过"的话术

### 阶段 3：完整约束体系（第 4 周起）

**目标**：建立完整的阶段护栏

**新增配置**：
- Proposal 规则：控制需求边界
- Specs 规则：固化可观察行为
- Design 规则：控制方案质量
- 自我合理化封堵：补充 CLAUDE.md

**预期效果**：
- 完整的规范驱动开发流程
- 模型严格遵守各阶段规则

### 采纳建议

| 团队情况 | 建议起点 | 原因 |
|----------|----------|------|
| **刚开始用 OpenSpec** | 阶段 1 | 先建立最基本的纪律 |
| **已用 OpenSpec，但测试常跳过** | 阶段 1 + 阶段 2 | 重点解决验证问题 |
| **流程成熟，想提升质量** | 直接阶段 3 | 已有基础，可以完整配置 |


---

## 第四部分：边界与禁忌

## 适用边界与场景选择

### Hardness 适用的场景

✅ **强烈推荐使用的场景**：

1. **规范驱动的工程项目**
   - 后端 API 开发
   - CLI 工具构建
   - 数据管道/ETL 任务

2. **已有测试框架的成熟项目**
   - 可以绑定具体测试命令（`npm test`、`pytest`）
   - 有 CI/CD 流程可参考

3. **多人协作或长期维护的项目**
   - 需要 artifacts 留存和可追溯性
   - 团队成员流动性高

4. **交互式开发场景**
   - Agent 辅助编码（人类可响应 STOP）
   - 需求到实现的自动化

### Hardness 不适用的场景

❌ **不推荐或需要调整的场景**：

1. **探索性原型（Prototype/PoC）**
   - **问题**：证据要求会拖慢快速试错
   - **建议**：只使用 L1-L2 硬度（Soft + Structured），跳过证据系统

2. **创意驱动的前端设计**
   - **问题**：UI/UX 的"完成"标准难以用命令表达
   - **建议**：用 `[MANUAL]` 标记替代自动化验证

3. **一次性脚本/临时工具**
   - **问题**：配置成本 > 脚本本身的价值
   - **建议**：不使用 OpenSpec，直接用 CLAUDE.md 的 Before Finishing 清单

4. **完全无人值守的自动化**（核心限制）
   - **无人值守 CI/CD 流水线**（触发 STOP 时无人响应）
   - **夜间自动化构建/部署**（人类不在场）
   - **原因**：Hardness 的 STOP 机制依赖人类响应
   - **替代方案**：使用传统 CI/CD 门禁

### 场景选择决策树

```
是否需要 Hardness？
    │
    ├─ 项目是否需要长期维护？
    │   ├─ 否 → 只用 CLAUDE.md 的 Before Finishing
    │   └─ 是 ↓
    │
    ├─ 是否有明确的测试命令？
    │   ├─ 否 → 只用 L1-L2（Soft + Structured），跳过 Apply 硬度
    │   └─ 是 ↓
    │
    ├─ 是否需要审计证据？
    │   ├─ 否 → 阶段 1-2（最小可行 Hardness）
    │   └─ 是 → 阶段 3（完整约束体系）
```


## Hardness 的禁忌

### 1. 不要在探索阶段过度 Hardness

探索阶段应该允许多方案、假设、草案。过早硬约束会导致模型僵硬。

❌ **过度 Hardness**：Explore 阶段必须给出唯一确定方案  
✅ **合理 Hardness**：Explore 阶段可以提出 2-3 个方案并比较

### 2. 不要把所有规则都写成 MUST

全部 MUST 会让模型难以判断优先级。应区分 MUST / SHOULD / MAY。

❌ **全 MUST**：
```yaml
rules:
  design:
    - "MUST: 画架构图"
    - "MUST: 画时序图"
    - "MUST: 画 ER 图"
```

✅ **合理分级**：
```yaml
rules:
  design:
    - "MUST: 说明推荐方案和至少 1 个替代方案"
    - "SHOULD: 复杂状态机画 Mermaid 图"
    - "MAY: 简单变更可省略详细架构图"
```

### 3. 不要用错误事实做硬规则

Hardness 会放大错误。Hard rule 必须来自已验证事实。

❌ **基于假设**：所有测试必须用 Mocha（实际项目用 Jest）  
✅ **基于事实**：所有测试必须用 Jest（已验证）

### 4. 不要让 Hard Rules 互相冲突

冲突规则会让模型无所适从。

**冲突示例：**
```yaml
rules:
  apply:
    - "必须严格按 tasks.md 执行"
    - "发现缺测试必须补测试"
    - "不得修改 tasks.md"
```
**问题**：如果 tasks 缺测试，模型既不能补，也不能修改。

**正确写法：**
```yaml
rules:
  apply:
    - "只按 tasks.md 已列任务执行"
    - "发现缺测试时，暂停并反馈 tasks 设计缺口"
    - "经确认后再更新 tasks 或继续"
```

### 5. 不要把 Hardness 当成替代验证

写了"测试必须通过"不等于已经验证。

**必须配合：**
- 实际运行命令
- 检查输出
- 记录证据
- verify 阶段核对

### 6. 不要让 Hardness 破坏可维护性

Tasks 过硬、过细会变成代码编辑流水账。

❌ **过度细化**：
```
[ ] 添加 import UserService
[ ] 定义 login 方法签名
[ ] 写 if 判断
```

✅ **合理粒度**：
```
[ ] 实现 UserService.login 方法，类型检查通过
[ ] 编写 login 方法的单元测试，期望 12 passed
```

**Hardness 不等于细碎化。真正好的 Hardness 是：硬完成标准 + 合理责任粒度。**


---

## 附录

## 完整配置示例

完整配置请参考本文"在 OpenSpec 中配置 Hardness"章节的示例。

### 最小配置（阶段 1）

参考本文开头"快速开始"部分。

### 完整配置（阶段 3）

参考"在 OpenSpec 中配置 Hardness"章节的完整示例。


## 真实案例研究（待补充）

### 案例占位说明

本节将补充真实项目案例，包括：

1. **案例 1：测试跳过问题**
   - Before：Agent 跳过测试直接归档
   - 配置：加了哪 3 条规则
   - After：Agent 被 STOP 拦截，正确处理

2. **案例 2：混合 Task 拆分**
   - Before：tasks 混合开发和测试
   - 配置：责任单元标准
   - After：拆分为独立 checkbox

3. **案例 3：证据缺失问题**
   - Before：verify 只有"已验证"
   - 配置：证据系统模板
   - After：完整的命令、输出、时间戳

**征集案例**：如果你有真实的 Hardness 实践案例，欢迎贡献。


## 常见问题

### Q1: Hardness 会降低开发效率吗？

**A**: 短期可能略慢（需要写证据、分拆 task），但长期会提升效率：
- 减少返工（测试提前发现问题）
- 减少沟通成本（需求边界清晰）
- 减少认知负担（每个阶段职责明确）

### Q2: 如何平衡 Hardness 和灵活性？

**A**: 按阶段分层：
- Explore 阶段：低 Hardness，鼓励发散
- Apply 阶段：高 Hardness，控制纪律
- 简单变更：可以省略部分 artifact（MAY 规则）

### Q3: 团队成员不适应 Hardness 怎么办？

**A**: 渐进式采纳（参考"渐进式采纳路径"）：
- 第 1 周：只配置最小可行 Hardness
- 第 2-3 周：逐步增加验证门禁
- 第 4 周起：完整约束体系

### Q4: Hardness 失效了怎么办？

**A**: 参考"失效诊断"章节：
1. 使用症状诊断表定位问题
2. 检查证据系统是否完善
3. 检查是否缺少封堵规则
4. 使用诊断清单调整配置

### Q5: 规则如何被执行？

**A**: Hardness 规则映射到 Claude Code 的四种原语，执行机制各不相同：

| 原语 | 执行机制 | 强制性 | Hardness 用途 |
|------|----------|--------|---------------|
| **Rule**（CLAUDE.md） | 会话启动注入上下文 | 软（可忽略） | 长期约束、Before Finishing |
| **Rule**（config.yaml） | OpenSpec 技能读取后注入 | 软（可忽略） | 各阶段 artifact 规则 |
| **Hook**（PreToolUse） | 工具调用前拦截 | 硬（无法绕过） | 硬 STOP、危险命令拦截 |
| **Subagent** | 隔离执行后回传结论 | 中（工具范围隔离） | 证据校验、一致性核对 |

两层执行机制：

1. **Prompt 层**（软规则）：
   - `CLAUDE.md` 和 `openspec/config.yaml` 被加载到 LLM 上下文
   - LLM "自觉"遵守规则，依赖理解和自我约束

2. **工具层**（硬规则，可选）：
   - 通过 Claude Code Hooks 强制执行
   - 在工具调用时拦截危险操作，LLM 无法绕过

**大多数场景使用 Prompt 层即可，工具层用于高风险操作。**


---

## 总结

Hardness 的核心不是"语气更强"，而是：

> **规则更具体、边界更清楚、失败更可处理、证据更可检查、流程更不可绕过。**

### 核心价值

**Hardness 是一种"工程纪律的编码化"**，它的价值在于：

1. **把隐性知识显性化**：把"老司机的经验"写成可执行规则
2. **把人类监督从"全程盯着"变成"关键节点介入"**
3. **让 AI Agent 从"聪明的助手"变成"可信赖的自动化工作流"**

### 不能替代的能力

Hardness **不能替代**：
- 人类对业务价值的判断
- 架构层面的权衡（技术债务、长期演进）
- 创造性的设计突破

### 一句话总结

> **用 Soft Guidance 促进思考，用 Structured Requirements 固化意图，用 Hard Constraints 控制执行，用 Gates 和 Evidence 决定是否完成。**

### 落地建议

如果要落地 Hardness，建议按照"渐进式采纳路径"：

1. **第 1 周**：阶段 1（最小可行 Hardness）
2. **第 2-3 周**：阶段 2（补充验证门禁）
3. **第 4 周起**：阶段 3（完整约束体系）

在真实项目中跑 2-3 个 Change，观察 STOP 触发频率和证据收集成本，再决定是否推进到完整体系。

**核心信念**：承认 AI 漂移不可避免，但通过分层门禁显著降低错误率，让漂移控制在可观察、可回滚、可追溯的范围内。

---

**文档版本**: v3.1  
**更新日期**: 2026-06-24  
**v3.1 变更**: 对齐 Claude Code 四原语定义——明确 config.yaml 是 Rule 的扩展实现、区分软/硬 STOP、补充 Subagent 验证场景  
**v3.0 变更**: 删减 40% 冗余内容，前置快速开始指南，移除过度设计机制，优化文档结构
