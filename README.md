# Novel Generator

`novel-generator` 是面向小说创作项目的生成器与运行层工程。它提供两部分能力：

- `novel` CLI：管理世界状态、章节正文、规则检查、裁决和状态机，是权威状态的唯一入口。
- Spec 运行层：向 Claude Code 等 Agent 工具提供 Skills、规则、提示词和工作流，通过 `novel` CLI 完成创作编排。

本仓库不存放具体小说的正文、世界数据库或运行报告。这些产物始终位于用户自己的目标小说项目中。

## 构建 CLI

```bash
cargo build -p novel-cli
```

开发构建产物位于 `target/debug/novel`。也可以把已安装的 `novel` 放入 `PATH`，或在安装 Spec 时通过 `NOVEL_BIN` 指定路径。

## 安装 Claude Code 创作原语

先创建目标小说项目目录，再从本仓库执行安装器：

```bash
mkdir -p /path/to/my-novel
NOVEL_BIN="$PWD/target/debug/novel" bash spec/install.sh /path/to/my-novel
```

安装器会先通过公开 CLI 初始化或预检目标世界，然后按显式清单写入：

```text
my-novel/
  .novel/                         # CLI 管理的权威世界状态
  .novel-spec/                    # 平台中立的运行资产，非权威状态
    skills/chapter-writing/
    rules/
    prompts/
    workflows/
    runtime/
  .claude/                        # Claude Code 薄适配
    skills/chapter-writing/
    agents/
    rules/
```

重复执行安装器会同步清单内文件，不删除或覆盖清单外的用户文件，也不会改写目标项目的 `CLAUDE.md`。

安装完成后，在目标项目中让 Claude Code 使用 `chapter-writing` Skill，并提供待写章节 ID、项目目录、上下文预算、最大修复次数和本章创作意图。Skill 会读取公共工作流，分离 writer、verifier 与 repair 角色，并只通过 `novel ... --json` 读取或提交权威状态。

## 状态与边界

```text
Agent 工具
  │  加载 Skills / rules / prompts / workflows
  ▼
Spec 运行层
  │  仅通过子进程调用 novel --json
  ▼
novel CLI
  │
  ▼
SQLite + 章节正文文件
```

- `.novel/` 由 CLI 管理，Spec、Skill 和 Agent 不得直接读取或修改其数据库与正文文件。
- `.novel-spec/` 只保存可重新安装的公共运行资产，不保存正文、数据库、报告或权威状态。
- `.claude/` 只保存 Claude Code 的发现入口和角色映射，不复制公共创作流程。
- CLI 非零退出、错误信封、版本不兼容或状态门禁失败都会停止后续推进。

CLI 命令语法、JSON 输出、错误协议、退出码、受控 view/scope 和版本策略以 [CLI Contract](contracts/cli/README.md) 为准。Spec 运行层的结构与使用方式见 [spec/README.md](spec/README.md)，稳定架构见 [repository/architecture.md](repository/architecture.md)，实现进度见 [repository/dev-progress.md](repository/dev-progress.md)。

## 开发验证

```bash
cargo test -p novel-cli
scripts/check-generated-drift.sh
bash spec/tests/install-claude.sh
bash spec/tests/check-claude-primitives.sh
bash spec/tests/chapter-loop.sh
```

项目长期执行约束见 [AGENTS.md](AGENTS.md)。
