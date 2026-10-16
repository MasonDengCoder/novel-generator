# CLI Contract 稳定边界

## 定义

**CLI Contract** 是软硬层（Spec 运行层与 CLI 硬层）之间的唯一稳定边界。

位置：`contracts/cli/`

## 契约内容

### 核心契约文档

| 文件 | 职责 |
|---|---|
| `commands.md` | 命令语法与参数约定 |
| `exit-codes.md` | 退出码语义与分类 |
| `error-codes.md` | 结构化错误码定义 |
| `json-output.md` | `--json` 输出约定与 schemaVersion 策略 |
| `entity-ids-sourcerefs.md` | 实体 ID 格式与引用规则 |
| `state-machine.md` | 状态机定义与转换约束 |
| `verdicts.md` | 裁决类型与语义 |
| `views-scopes.md` | 视图与作用域约定（如 `--view writer`、`--scope chapter`） |

### 机器可验证产物

| 目录/文件 | 职责 |
|---|---|
| `json-schemas/` | JSON schema 定义（如 `info-output.schema.json`） |
| `examples/` | Golden examples（稳定输出示例） |
| `contract-tests/` | 契约测试（Rust 集成测试） |

## 稳定承诺

### 稳定边界（Spec 可依赖）

✅ `novel` 命令名称与参数语义
✅ `--json` 输出结构（包含 `schemaVersion`）
✅ 结构化错误对象 schema
✅ 退出码类别与语义
✅ 结构化错误码
✅ 错误协议版本（默认 V2：`V2ErrorEnvelope`，`errorId`/`category`/`exitCode`；显式选择 V1 为弃用兼容路径）
✅ 实体 ID 与 sourceRefs 格式（`<prefix>-<number>`，如 `chapter-001`、`mp-1-001`（3 位零填充，与 `chapter-001` 对齐）；非 UUID）
✅ 受控 view/scope 名称及其机器语义
✅ 状态机状态枚举与转换约束

### 非稳定实现细节（Spec 不得依赖）

❌ SQLite schema（表名、列名、索引）
❌ `.novel` 内部路径（除契约明确承诺的顶层路径，见下）
❌ 正文文件命名格式（CLI 承诺稳定引用接口，不承诺路径格式）

> **`.novel` 路径分层**：`novel init` 承诺创建的顶层结构（`.novel/novel.db`、`.novel/content/chapters/`、`.novel/content/residues/`、`.novel/content/reports/`、`.novel/tmp/`、`.novel/locks/`）属稳定契约行为，Spec 可依赖其存在。但 `commands.md` 中出现的 SQLite 表名（`chapters`/`volumes`/`settings`/`constraints` 等）与列名（`content_hash`/`content_path`/`word_count` 等）属**说明性描述**，用于解释命令实现机制，**非 Spec 可依赖的契约**——Spec 只应消费 `--json` 输出的稳定字段名（如 `contentHash`、`storedHash`），不得直接依赖表名/列名。
❌ Rust module 名称（`commands::`, `services::`, etc.）
❌ repository/storage 实现细节
❌ migration 具体文件名或顺序
❌ Spec prompt 形态或 LLM 调用策略

## Schema 演进策略

### `schemaVersion` 字段
- 所有 `--json` 输出必须包含顶层 `schemaVersion` 字段
- 格式：十进制整数字符串（如 `"8"`），禁止带前缀或多段形态（`"v1"`、`"1.0"`、`"1.0.0"` 均非法）
- Spec 消费 JSON 输出时必须检查 `schemaVersion`
- 详细格式与 bump 条件见 `contracts/cli/json-output.md` §2.2–2.3（SSOT）

### 向后兼容原则
- **破坏性变更**（MUST bump `schemaVersion`）：顶层结构形态改变、必填字段缩减/重定义、字段类型跨入不兼容域
- **非破坏性变更**（不 bump）：新增可选字段、新增可选枚举值、向后兼容的语义细化

### 破坏性变更流程
1. 在新 major 版本中引入变更
2. 更新 `contracts/cli/json-schemas/` 与 golden examples
3. 更新契约测试
4. 文档中标注废弃字段与迁移路径
5. Spec 运行层必须适配新版本或明确拒绝不支持的 schemaVersion

## 契约测试

### 测试位置
`contracts/cli/contract-tests/` （Rust 集成测试）

### 测试覆盖
- ✅ 命令语法正确性（参数解析、必填参数、互斥参数）
- ✅ `--json` 输出符合 JSON schema
- ✅ 退出码语义正确（成功 0、参数错误 2、资源不存在 3、etc.）
- ✅ 结构化错误对象格式正确
- ✅ 状态机转换约束（合法转换成功、非法转换失败）
- ✅ 输出稳定性验证，分两类：
  - **Golden examples 比对**（输出与 `examples/` 中示例一致）：核心命令直接比对（如 `e2e.rs`、`setting_commands.rs`、`constraint_commands.rs`、`verdict.rs`）
  - **JSON schema 校验**：其余命令按 `json-schemas/` 校验输出结构（不逐一比对 golden）

### 测试策略
- 使用 `assert_cmd` 测试命令行接口
- 使用 `predicates` 验证输出
- 使用 `tempfile` 创建隔离测试环境
- 每个契约变更必须同步更新契约测试

## 契约文档维护原则

### 单一真相来源（SSOT）
- 契约文档是 SSOT，其他文档应引用而非复制
- `docs/` 可引用契约，不得复制退出码表、错误码表、view/scope 列表
- `CLAUDE.md` 可引用契约，不得自行维护命令大全

### 更新流程
新增或修改 CLI 命令时：
1. 更新 `contracts/cli/commands.md`
2. 如涉及新 JSON 输出，更新 `json-schemas/` 和 golden examples
3. 如涉及新退出码/错误码，经 `ErrorCatalog` / 命令契约注册表声明，再运行 `cargo run -p spec-gen -- generate` 重生成 `exit-codes.md` 和 `error-codes.md`（生产物，勿手改；`scripts/check-generated-drift.sh` 锁漂移）
4. 如涉及新状态或转换，更新 `state-machine.md`
5. 添加或更新契约测试
6. 运行 `cargo test -p novel-cli` 确保契约测试通过

### 契约文档质量要求
- 每个命令必须有语法定义、参数说明、退出码列表
- 每个 JSON 输出必须有 schema 或 golden example
- 每个状态转换必须有前置条件与失败退出码
- 契约文档必须机器可验证（通过 contract tests）

## Spec 运行层消费契约的最佳实践

### 命令调用
```bash
# 正确：使用 --json 消费稳定输出
novel info --json

# 错误：解析人读输出（不稳定）
novel info | grep "Project Name"
```

### 错误处理
```bash
# 正确：检查退出码并处理（默认 V2 协议，进程退出码为类别派生码）
if ! novel chapter get chapter-001 --json > output.json; then
  exit_code=$?
  # 根据退出码决定重试、报错或回退（类别码：6=not_found、5=external_modification，完整取值见 contracts/cli/exit-codes.md SSOT）
  case $exit_code in
    6) echo "Chapter not found" ;;
    5) echo "Content modified externally (hash mismatch)" ;;
    *) echo "Unknown error" ;;
  esac
fi

# 错误：忽略退出码继续
novel chapter get ch_123 --json > output.json || true
```

### Schema 验证
```python
import json

output = json.loads(subprocess.check_output(["novel", "info", "--json"]))

# 正确：检查 schemaVersion（十进制整数字符串）
if output.get("schemaVersion") != "8":
    raise ValueError("Unsupported schema version")

# 正确：使用已知字段，忽略未知字段
project_name = output["projectName"]
```

### 不得绕过契约
```python
# 错误：直接读取 SQLite
import sqlite3
conn = sqlite3.connect(".novel/novel.db")
cursor = conn.execute("SELECT * FROM chapters")

# 正确：通过 CLI 命令
chapters = json.loads(subprocess.check_output(["novel", "chapter", "list", "--json"]))
```

## 契约变更通知机制

当前策略：
- 契约变更记录在对应契约文档的 CHANGELOG 章节
- 重大变更在 `contracts/cli/README.md` 中突出标注
- 未来可考虑：semver 标签、deprecation warnings、migration guides
