# Claude Code 与 Codex 能力一致性矩阵

本文记录单章创作入口的跨 Agent 一致性要求。公共创作语义以 `spec/skills/`、`spec/rules/`、`spec/workflows/`、`spec/prompts/` 和 `spec/runtime/` 为唯一来源；CLI 命令、JSON 输出、错误协议、退出码、view/scope 与实体引用只引用 `contracts/cli/`，不在此复制具体表格。

| 能力维度 | Claude Code 入口 | Codex 入口 | 一致性判定 |
| --- | --- | --- | --- |
| Skill 发现 | `.claude/skills/chapter-writing/SKILL.md` | `.agents/skills/chapter-writing/SKILL.md` | 目录不同，均加载同一 `.novel-spec/skills/chapter-writing/SKILL.md` |
| 公共规则与工作流 | `.novel-spec/rules/`、`.novel-spec/workflows/` | `.novel-spec/rules/`、`.novel-spec/workflows/` | 文件内容与安装清单源一致 |
| writer | 当前主 Agent 会话 | 当前 Codex 会话 | 生成候选正文，不负责自我验收 |
| verifier | 独立 `chapter-verifier` Agent | 独立 subagent | 只读权威上下文并返回检查/裁决候选 |
| repair | 独立 `chapter-repair` Agent | 另一个独立 subagent | 只处理明确失败证据并返回修复候选 |
| 权威状态访问 | `.novel-spec/runtime/` 调用 `novel ... --json` | `.novel-spec/runtime/` 调用 `novel ... --json` | 不直接访问 SQLite 或 `.novel/content` |
| 版本与失败处理 | 检查成功 `schemaVersion`、失败信封和退出码 | 检查成功 `schemaVersion`、失败信封和退出码 | 非零或不兼容时停止后续推进并保留诊断 |
| 成功闭环 | CLI 接受正文、检查、裁决和状态转换后回读 | CLI 接受正文、检查、裁决和状态转换后回读 | 最终章节 ID、状态和正文 hash 可规范化比较 |
| repair 闭环 | 首次失败保持 drafting，修复后重新验证 | 首次失败保持 drafting，修复后重新验证 | 不允许失败后提前推进 |
| 失败诊断 | 返回阶段、命令、退出码、原始错误和可修复性 | 返回阶段、命令、退出码、原始错误和可修复性 | 字段语义一致，平台只影响入口呈现 |

## 证据规则

- 安装和发现测试使用临时目标项目与公开 `novel` CLI。
- 行为 parity 测试使用相同 fixture、输入、writer/verifier/repair 替身和修复上限。
- 测试结果必须保存命令输出和规范化比较结果；真实模型的文学质量不作为确定性 CI 判定。
