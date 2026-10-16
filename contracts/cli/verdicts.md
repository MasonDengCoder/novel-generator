# CLI Verdicts Contract

> **本文件是 `novel` CLI 主线落地裁决数据结构与语义的唯一事实来源（SSOT）。** 记录裁决数据结构、hash 绑定语义、UPSERT 覆盖语义与 `approved` 后裁决不可改约束。
> 裁决语义是稳定契约（来源：`CLAUDE.md` §7、§10，s04-state-machine-and-verdicts spec）。Spec 运行层只能通过 `novel verdict set-mainplot-landing <chapter-id> --node-id <id> --achieved <0|1> --json` 写入裁决并消费结果，不得直接读写裁决存储。
> 任何硬阻断 MUST 以非零退出码 + 结构化错误对象返回，不得静默降级。

## 裁决数据结构

裁决记录章节对主线节点的落地判定，绑定写入时刻的正文 hash，作为状态推进门禁的输入之一。

| 字段 | 类型 | 说明 |
|---|---|---|
| `chapterId` | string | 章节 ID（格式 `chapter-NNN`）。 |
| `nodeId` | string | 主线节点 ID（文本，由 Spec/外部定义）。 |
| `achieved` | integer | `0`（未达成）或 `1`（已达成）。 |
| `contentHash` | string | 裁决绑定的正文 SHA-256 hash（写入时刻的 `chapters.content_hash`）。 |
| `createdAt` | string | ISO-8601 写入时间戳。 |
| `verdictType` | string | 裁决类型。本切片仅 `mainplot-landing`，预留扩展。 |

> `verdictType` 列与表结构预留以支持后续切片扩展裁决类型，但 CLI 命令本切片只暴露 `set-mainplot-landing`，不预留未实现命令入口。SQLite 表结构、迁移文件名均为非稳定实现细节（`CLAUDE.md` §9），不作为跨层契约。

## hash 绑定语义

- 裁决写入时 SHALL 绑定写入时刻的 `chapters.content_hash`，作为裁决的版本锚点。
- 状态推进门禁（进入 `review`/`approved`）验证裁决的 `contentHash` 与当前 `chapters.content_hash` 一致；不一致视为 stale，硬阻断（退出码 `3`，错误码 `precondition_blocked`）。
- 正文修改后重写裁决，新裁决的 `contentHash` SHALL 绑定为更新后的正文 hash，而非旧 hash。
- 外部修改正文（重算 hash 与 `chapters.content_hash` 不一致）时拒绝写入裁决（退出码 `21`，错误码 `content_modified_externally`）。

## UPSERT 覆盖语义

- 同一 `(chapterId, verdictType, nodeId)` 的新裁决 SHALL 覆盖旧裁决（UPSERT 语义）。
- 覆盖时绑定的 `contentHash` SHALL 反映当前正文版本（读取当前 `chapters.content_hash` 填入，不沿用旧行值）。
- 同节点重复写入覆盖旧裁决后返回退出码 `0`，新裁决的 `achieved` 与 `contentHash` 反映本次写入。

## `approved` 后裁决不可改约束

- 章节状态为 `approved` 时拒绝写入裁决（`CLAUDE.md` §7「approved 后不得修改裁决」）。
- 返回退出码 `3`，错误码 `precondition_blocked`。
- 此约束在 service 层检查 `chapter.status != approved`，不依赖门禁。

## 事务一致性

裁决写入在 SQLite 事务中执行：校验章节存在 → 检测外部修改 → 绑定当前 hash → UPSERT 裁决 → 提交。任一步失败回滚。裁决不写正文文件，无跨存储半提交问题；但事务提交失败仍需回滚（退出码 `70`，错误码 `storage`）。
