# 存储模型与一致性保障

## 目标小说项目存储布局

```text
目标小说项目/
  .novel/
    novel.db               # SQLite：权威元数据、状态机、索引、引用、hash、归档状态
    content/
      chapters/
        <chapter-id>.md    # 章节正文 Markdown 文件
      residues/
        <chapter-id>.md    # 可选：长残余文本
      reports/
        <volume-id>.md     # 可选：大轮回/卷末报告
    tmp/                   # 临时文件
    locks/                 # 并发锁
```

## 混合存储策略

### SQLite（权威元数据）
记录：
- 项目级元数据、设置、约束
- 实体索引（卷、章节、涌现、事件、节点）
- 状态机状态（章节状态、涌现状态、卷状态）
- 主线落地裁决与规则检查结果
- **正文文件路径引用**
- **正文 hash**
- 字数、更新时间
- 归档/冻结状态
- schemaVersion

### 文件存储（正文内容）
- 使用稳定、人可读的 Markdown 或纯文本格式
- 章节正文路径：`.novel/content/chapters/<chapter-id>.md`
- 残余路径：`.novel/content/residues/<chapter-id>.md`
- 报告路径：`.novel/content/reports/<volume-id>.md`

### 为什么混合存储？
- **人可读**：正文使用 Markdown，用户可直接打开查看、备份、版本控制
- **事务性**：元数据与状态机状态在 SQLite 中享受 ACID 保证
- **Hash 保护**：SQLite 记录正文 hash，防止绕过 CLI 直接编辑
- **查询性能**：元数据查询不需要扫描大量正文文件

## 正文写入一致性保障

### 写入流程（10 步）
涉及正文的写操作按以下顺序实现（`services/` 层编排门禁与事务，正文写入与元数据更新在 `repositories/` 内于事务中执行）：

1. **校验输入正文文件**（`services/` + `validators/`，锁外；UTF-8 与可读性）
2. **获取项目级写锁**（`services/` 调 `storage/` 的 `ProjectWriteLock`，锁文件 `.novel/.write.lock`）
3. **打开数据库**（`services/` 调 `storage/connection`）
4. **归档冻结门禁**（`services/` 调 `validators/guards`：章节所属卷 archived 时拒绝）
5. **approved 终态门禁**（`services/` 调 `storage/chapters` 读状态：approved 时拒绝修改正文）
6. **开启 SQLite transaction**（`services/`）
7. **读取章节元数据、计算新 hash 与 word_count**（`repositories/` 调 `validators` 的 `calculate_content_hash`/`calculate_word_count`）
8. **写入正文临时文件 + 原子替换正式正文文件**（`repositories/` 调 `storage/file_ops`：`write_temp_file` → `atomic_replace`，临时文件 `.novel/tmp/<uuid>.md`）
9. **更新 SQLite 元数据**（`repositories/` 于事务内：`content_hash`/`word_count`/`updated_at`）
10. **提交 transaction**（`services/`）；临时文件写入失败时清理（`repositories` 调 `storage/file_ops::cleanup_temp_file`）

> hash 计算定义在 `validators/chapter_validator`，临时文件写入与原子替换在 `storage/file_ops`，由 `repositories/chapter_repository` 在事务内编排。`services` 不直接操作正文文件。

### 半提交场景处理
**问题**："正文文件已原子替换，但 SQLite commit 失败"

**检测**：CLI 启动时检查正文 hash 与 SQLite 记录是否一致

**修复策略**：
- 如检测到 hash 失配且无法确定权威来源，拒绝继续并要求用户确认
- 提供 `novel chapter reimport <id> --content <file>` 命令重新计算 hash 并同步元数据

### Hash 检测机制
CLI 在以下时机必须检查正文 hash：
- 读取正文用于状态推进
- 进入 `review` 或 `approved` 状态
- 执行归档操作
- 绑定裁决或检查结果

**外部变更检测**：
- 用户绕过 CLI 直接编辑 `.novel/content/chapters/<id>.md`
- CLI 重新计算 hash，发现与 SQLite 记录不一致
- **拒绝继续**：返回非零退出码（默认 V2 下为 `external_modification` 类别码 `5`——进程退出码由 `ErrorCatalog` 类别派生，退出码/错误码字符串取值见 `contracts/cli/exit-codes.md`/`error-codes.md` SSOT；错误码 `content_modified_externally`）
- **提示**：要求用户使用 `novel chapter update` 重新导入或确认变更

## Stale 保护机制

### 裁决与检查结果绑定
- **主线落地裁决**必须记录适用的正文 hash
- **规则检查结果**必须记录适用的正文 hash

### Stale 标记
正文修复或重写后：
1. 重新计算正文 hash
2. 将旧裁决和旧检查结果标记为 `stale=true`
3. 状态推进前检查：如裁决或检查结果 stale，拒绝进入 `review`/`approved`

### 重新验证流程
正文修改后，用户必须：
1. 重新执行规则检查（`novel constraint check`）
2. 重新确认主线落地裁决（`novel verdict set-mainplot-landing`）
3. 新裁决与检查结果绑定新正文 hash
4. 此时状态机才允许推进

## 归档与冻结

### 章节归档
- `approved` 是章节内容状态终态
- 已归档是冻结/归档维度（与状态机状态正交）
- `approved` 后允许执行受控归档
- 归档后不得修改：正文、裁决、残余、四相日志、thread binding

### 卷封卷
- 卷末关闭大轮回前必须：
  - 处置所有 pending 涌现
  - 验证卷目标达成
  - 处理卷内事件终态
  - 写入监测报告
- 封卷后卷内所有实体冻结，不可修改

## 并发与锁

### 项目级写锁
- 所有涉及正文写入的操作必须获取项目级写锁
- 实现：文件锁（Unix: `flock`, Windows: `LockFileEx`）
- 锁文件路径：`.novel/.write.lock`

### SQLite 并发
- SQLite 默认支持多读单写
- 长时间写操作使用 `BEGIN IMMEDIATE` 提前获取写锁
- 避免 `SQLITE_BUSY` 错误：设置合理的 busy timeout

### 临时文件清理
- 写入失败时必须清理 `.novel/tmp/` 下的临时文件
- CLI 启动时可选清理孤儿临时文件（创建时间超过阈值）

## Migration 策略

### Schema 演进
- 使用 `refinery` 或项目内置 migration runner
- Migration 文件命名：`V<version>__<description>.sql`
- 每次 migration 更新 `schemaVersion`

### 向后兼容性
- CLI 启动时检查 SQLite `schemaVersion`
- 如 schema 版本过旧，拒绝继续并提示升级
- 如 schema 版本过新，拒绝继续并提示 CLI 版本不匹配

### Migration 测试
- 测试从旧 schema 升级到新 schema
- 测试数据完整性保持
- 测试回滚策略（如适用）

## 导出与备份

### 导出命令
`novel export` 应能从 SQLite 元数据和正文文件组合出完整、可审计的世界数据：
- 所有实体元数据
- 章节正文（从文件读取）
- 裁决、检查结果、四相日志
- 状态机状态、归档状态
- schemaVersion

### 备份建议
- 定期备份整个 `.novel/` 目录
- SQLite: 使用 `VACUUM INTO` 或 `.backup` 命令
- 正文文件：标准文件系统备份即可
- 版本控制：可选将 `.novel/` 纳入 git（注意 `.novel/tmp` 和 `.novel/locks` 应忽略）
