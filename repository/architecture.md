# 核心架构边界

## 项目定位

`novel-generator` 是面向小说创作项目的生成器与运行层工程，**不是某一本小说正文、数据库或世界状态的存放处**。

本仓库负责提供：
- `novel` CLI（硬层内核）
- Spec 运行层（创作流程引擎）
- Claude Code 原语（rules/agents/hooks/skills）
- CLI Contract（稳定契约）
- 安装能力

具体小说项目的 `.novel/` 数据库、章节正文、报告和运行产物应保存在**目标小说项目**中，不应写入本仓库。

## 分层模型

### 依赖方向（单向）

```text
Spec 运行层 (spec/)
  │
  │  仅通过子进程调用 novel 命令
  ▼
CLI 硬层 (crates/novel-cli/)
  │
  ▼
SQLite + 正文文件 (.novel/)
```

### 职责边界

| 层 | 职责 | 不可做 |
|---|---|---|
| **Spec 运行层** | 创作流程、阶段编排、提示词、规则注入、LLM/代理调用、验证流程、对子进程 `novel` 命令的调用 | ❌ 直接读写 SQLite<br>❌ 直接读写 `.novel/content`<br>❌ import CLI 内部模块<br>❌ 自行判断状态机推进条件<br>❌ 忽略非零退出码 |
| **CLI 硬层** | 世界状态日志唯一入口、状态机、权限边界、事务一致性、错误码、封卷门禁、归档与防绕过 | ❌ 反向依赖 Spec<br>❌ 调度 LLM<br>❌ 决定 prompt 形态<br>❌ 保存 Spec 运行时缓存 |
| **存储层** | SQLite transaction、migration、锁、文件写入原子性、hash 计算与检测 | ❌ 业务判断<br>❌ 状态机逻辑 |

### 稳定边界

**CLI Contract** 是软硬层唯一稳定边界：
- 命令语法与参数约定
- `--json` 输出结构
- 结构化错误对象 schema
- 退出码语义
- 实体 ID 与引用规则
- golden examples、contract tests

**非稳定边界**（不得跨层依赖）：
- SQLite schema
- `.novel` 内部路径
- 正文文件布局
- Rust module 名称
- repository/storage 实现
- migration 文件名
- Spec prompt 形态

## 目录结构

### 已存在目录

```text
novel-generator/
  crates/
    novel-cli/              # Rust CLI：硬层、SQLite、状态机、事务、命令
      src/
        commands/           # 命令解析与适配
        services/           # 业务逻辑与状态机
        repositories/       # 数据访问
        storage/            # SQLite、migration、锁
        renderers/          # 输出格式化
        validators/         # 完整性校验
        errors/             # 错误类型
        domain/             # 领域模型
  contracts/
    cli/
      README.md             # CLI 契约总览
      commands.md           # 命令契约
      exit-codes.md         # 退出码契约
      error-codes.md        # 错误码契约
      json-output.md        # JSON 输出约定
      state-machine.md      # 状态机契约
      verdicts.md           # 裁决契约
      views-scopes.md       # 受控 view/scope 名契约
      entity-ids-sourcerefs.md  # 实体 ID 与 sourceRefs 契约
      json-schemas/         # 机器可验证 schema
      examples/             # golden examples
      contract-tests/       # CLI 契约测试
  docs/                     # 项目文档
  reference/                # 参考文档
  openspec/                 # OpenSpec 配置与变更
  repository/               # 项目知识库（本目录）
```

### 目标目录（未建）

```text
  spec/                     # Spec 运行层（目标目录）
    runtime/                # 运行引擎
    prompts/                # 提示词模板
    rules/                  # 规则定义
    workflows/              # 流程编排
    fixtures/               # 测试 fixture
```

## 防绕过原则

- 外部系统只能通过 `novel` 命令访问世界状态
- 不提供外部 SDK/API 绕过 CLI 内部模块
- CLI 负责状态机、权限边界、事务一致性、错误码、封卷门禁、归档与防绕过
- Spec、Skill、Subagent、Web 或脚本不得绕过 service/use-case 与 guard
- Web/view/serve 等适配层不得绕过 service/use-case 与 guard

## 安装模型

Spec 安装脚本的目标：
1. 把本仓库中的 Claude Code 原语（rules/agents/hooks/skills）复制到目标小说项目的 `.claude/` 目录
2. 调用 `novel init` 在目标项目内初始化 `.novel/` 数据库与必要目录
3. 目标小说项目从此成为独立创作环境，世界状态与正文保存在目标项目内

**边界**：本仓库不保存具体小说项目的 `.novel/`、章节正文、世界数据库或运行报告。
