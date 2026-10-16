# CLI 模块对齐清单

> 来源：基于 `docs/CLI架构设计.md` 抽取，用于 CLI 开发时统一模块边界、调用方向、输出契约、错误处理与硬阻断责任归属。
>
> **维护说明**：本文件原 checkbox 跟踪已由 `repository/dev-progress.md`（实现进度：命令矩阵 + 未实现实体清单）与 `contracts/cli/` 各 SSOT（契约）取代。模块职责的稳定定义以 `repository/cli-modules.md` 为 SSOT，本文件只保留 docs 侧的"常见错位检查"与"一句话原则"作为开发期速查，不再重复模块职责全文与勾选状态。

---

## 1. 稳定知识引用（不在此自抄）

以下内容已由 SSOT 承载，本文件不重复定义，开发时直接查阅：

| 主题 | SSOT 位置 |
|---|---|
| 分层调用链（bin→parser→adapter→service→validator/guard→repository→storage→renderer） | `repository/cli-modules.md` §分层调用链 |
| 各模块职责（commands/services/repositories/storage/renderers/validators/errors/domain） | `repository/cli-modules.md` §模块职责 |
| 正文写入 10 步流程与层归属 | `repository/cli-modules.md` §正文写入一致性保障 |
| 参数校验归属（枚举/范围/引用分层） | `repository/cli-modules.md` §参数校验归属 |
| 技术栈对应表 | `repository/cli-modules.md` §技术栈对应 |
| 测试策略与覆盖重点 | `repository/cli-modules.md` §测试策略 |
| 状态机门禁清单与转换约束 | `contracts/cli/state-machine.md` |
| 退出码语义 | `contracts/cli/exit-codes.md`（§A 生效 / §B 预留） |
| 错误码字符串与错误对象字段 | `contracts/cli/error-codes.md`、`contracts/cli/json-output.md` |
| 受控 view/scope 名称 | `contracts/cli/views-scopes.md` |
| 实体 ID 与 sourceRefs 格式 | `contracts/cli/entity-ids-sourcerefs.md` |
| 实现进度（已实现/未实现实体清单） | `repository/dev-progress.md` §7 |

---

## 2. 常见错位检查（docs 侧速查）

开发期发现以下任一情况即为分层错位，须回退到 SSOT 对应层修复：

- command 层实现了复杂业务规则（应在 `services/`）。
- command 层直接访问 storage 或正文文件（应经 `services/` + `repositories/`）。
- repository 层实现了状态机或权限判断（应在 `services/` + `validators/guards`）。
- serve/view 绕过 service/use-case 与 guard。
- JSON 输出缺少统一 `schemaVersion`。
- 错误输出缺少统一错误码和退出码。
- 写操作没有事务或原子替换。
- 写操作失败后留下半写入状态（应通过 hash 失配检测兜底）。
- `approved` 章节被修改正文/裁决/残余/四相日志/thread binding。
- 未裁决或裁决 stale 的章节进入 `review`。
- 存在 hard 违规时章节进入 `review` / `approved`。
- 未归档涌现时关闭大轮回或归档整卷。

> 各门禁的生效前置条件与回退规则以 `contracts/cli/state-machine.md` 为 SSOT，本清单只做开发期错位提示，不重复门禁全文。

---

## 3. 一句话原则

> commands 只接命令，services 统一执行用例和硬阻断，repositories 只读写数据，storage 保证事务一致，renderers 统一输出契约，adapters 不绕过 service，errors 统一退出码；外部永远只能通过 `novel` 命令访问世界状态。

---

## 4. 与其他文档的关系

- `repository/cli-modules.md`：模块职责稳定知识库（本文件为其 docs 侧速查补充，不重复其全文）。
- `docs/CLI架构设计.md`：CLI 硬层架构设计（含权衡理由、`.novel/` 布局、正文写入流程的设计说明）。
- `contracts/cli/`：CLI 契约 SSOT（命令、退出码、错误码、view/scope、状态机、JSON schema）。
- `repository/dev-progress.md`：实现进度（命令矩阵 + 未实现实体清单）。
- `repository/storage-model.md`：存储模型与一致性保障细节。
