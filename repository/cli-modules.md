# CLI 内部模块职责

## 分层调用链

推荐调用链：

```text
bin/novel
  → command parser (clap)
  → command/view/serve adapter
  → service/use-case
  → validator / guard
  → repository
  → storage transaction
  → renderer (human-readable / JSON)
```

## 模块职责

### `commands/`
- 参数解析（clap derive）
- 基础参数校验（非空、格式）
- 输出选择（`--json` flag）
- **不得**：直接访问 SQLite 或正文文件、执行业务判断、调用状态机逻辑

### `services/` / guard
- 统一执行状态机逻辑
- 权限矩阵与前置条件
- 跨实体一致性校验
- 业务规则编排
- 调用 validator 与 repository
- **核心职责**：确保所有业务规则在此层硬阻断，不允许绕过

### `repositories/`
- 封装 SQL 查询与正文文件读写（在 service 开启的事务句柄内执行）
- 数据访问抽象，可调用 `storage/file_ops` 做临时文件写入与原子替换
- **不得**：自行开启/提交事务、做状态机门禁判定（如 approved/归档冻结）、做跨实体一致性裁决——这些在 `services/` + `validators/guards`

### `storage/`
- SQLite connection pool 与 transaction 管理
- Migration runner（schema 演进）
- 锁管理（项目级写锁、实体级锁）
- 文件写入原子性（临时文件 + 原子替换）
- Hash 计算与检测
- **不得**：业务逻辑、状态推进判断

### `renderers/`
- 人读输出格式化（表格、键值对）
- JSON 输出序列化（稳定 schema + schemaVersion）
- **不得**：决定业务规则、修改数据

### `validators/`
- 引用完整性校验（外键存在性）
- 参数合法性检查（ID 格式、枚举值）
- schemaVersion 兼容性检查
- Hash 一致性验证
- **职责**：可复用的通用检查逻辑，不包含特定业务状态机判断

### `errors/`
- 统一错误类型定义（thiserror）
- 结构化错误对象
- 退出码映射
- **契约**：错误码、退出码、错误消息结构必须与 `contracts/cli/error-codes.md` 和 `contracts/cli/exit-codes.md` 对齐

### `domain/`
- 领域模型定义
- 实体结构（Chapter, Volume, Emergence, Event, Node, etc.）
- 状态枚举（ChapterState, EmergenceState, VolumeState）
- **不得**：包含数据访问逻辑、业务流程编排

## 关键约束

### 状态机与权限矩阵
- 必须在 `services/` 层统一执行
- 不得在 `commands/` 或 `repositories/` 层分散判断
- 所有状态推进都需要经过 guard 的前置条件检查

### 正文写入一致性保障
涉及正文的写操作按以下顺序实现（`services/` 层编排门禁与事务，正文写入与元数据更新在 `repositories/` 内于事务中执行）：

1. 校验输入正文文件（UTF-8、可读）（`services/` + `validators/`，锁外）
2. 获取项目级写锁（`services/` 调 `storage/` 的 `ProjectWriteLock`）
3. 打开数据库（`services/` 调 `storage/connection`）
4. 归档冻结门禁：章节所属卷 archived 时拒绝（`services/` 调 `validators/guards`）
5. approved 终态门禁：章节 `approved` 时拒绝修改正文（`services/` 调 `storage/chapters` 读状态）
6. 开启 SQLite transaction（`services/`）
7. 读取章节元数据、计算新 hash 与 word_count（`repositories/` 调 `validators` 的 hash/字数计算）
8. 写入正文临时文件 + 原子替换正式正文文件（`repositories/` 调 `storage/file_ops`）
9. 更新 SQLite 元数据（content_hash/word_count/updated_at）（`repositories/` 于事务内）
10. 提交 transaction（`services/`）；临时文件写入失败时清理（`repositories` 调 `storage/file_ops`）

> 注：hash 计算（`calculate_content_hash`/`calculate_word_count`）定义在 `validators/chapter_validator`，正文临时文件写入与原子替换在 `storage/file_ops`，二者由 `repositories/chapter_repository` 在事务内编排调用。`services` 层负责门禁（归档冻结 + approved 终态）与事务边界，不直接操作正文文件。

### 参数校验归属

参数合法性校验分两层，按校验形态归属：

- **枚举值校验**（有限离散集合，如 `<target-status>` ∈ `outline/drafting/review/approved`、`--severity` ∈ `soft/hard`、`--achieved` ∈ `0/1`、`--decision`/`--status`/`--type` 等）：在 `commands/` 层用 clap `value_parser` 限定，非法值在解析阶段即被拒绝（退出码 2），不进入 `services/`。
- **连续数值范围校验**（如 `--rating` 0-10、`--order` >= 1）：在 `services/` 层校验。clap `value_parser!(i64)` 仅约束为整数，范围断言由 service 返回 `usage`（退出码 2）。
- **跨实体引用与状态机校验**（章节存在、卷未归档、状态转换合法等）：在 `services/` + `validators/`/guard 层，不进 `commands/`。

边界：形态校验（枚举/类型）尽早前置到 `commands/`，业务校验（范围/引用/状态）后置到 `services/`。

### 错误处理与退出码
- 所有业务错误必须通过 `errors/` 模块返回结构化错误对象
- 命令适配层负责将错误映射到退出码
- 退出码语义必须与 `contracts/cli/exit-codes.md` 对齐

### JSON 输出稳定性
- 所有 `--json` 输出必须通过 `renderers/` 模块序列化
- JSON schema 必须包含 `schemaVersion` 字段
- Schema 演进必须保持向后兼容或明确版本升级策略
- 不得在 JSON 输出中混入 ANSI 颜色、解释性旁白或非结构化提示

## 技术栈对应

| 模块 | 主要依赖 |
|---|---|
| `commands/` | `clap` (derive) |
| `services/` | `rusqlite`, `serde`, `time` |
| `repositories/` | `rusqlite`, `serde` |
| `storage/` | `rusqlite`, `uuid`, `libc` (Unix) |
| `renderers/` | `serde_json`, `comfy-table`, `serde` |
| `validators/` | `rusqlite`, `sha2` |
| `errors/` | `thiserror`, `serde` |
| `domain/` | `serde` (derive) |

## 测试策略

- **单元测试**：`services/`、`validators/`、`domain/` 模块内逻辑
- **集成测试**：`crates/novel-cli/tests/` 通过命令行接口测试端到端行为
- **契约测试**：`contracts/cli/contract-tests/` 验证命令契约、JSON schema、退出码、错误码

测试覆盖重点：
- 状态机所有合法转换与非法转换
- 并发写入与锁竞争
- 正文外部修改检测（hash 失配）
- Stale 裁决与检查结果拒绝
- 未初始化、资源不存在、权限不足等边界场景
- 临时文件清理与失败后一致性
