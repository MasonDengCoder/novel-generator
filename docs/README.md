# 文档索引 · novel-generator

文档按职责分为三层：

| 层 | 路径 | 职责 |
|----|------|------|
| 项目级共享 | `docs/` | 立项、需求、概要架构、设计理念、方法论内核、测试/验收等跨层资料 |
| 硬层 | `packages/novel-lifecycle-cli/docs/` | CLI 契约主副本、硬层架构、详细设计、schema 与门禁 |
| 软层 | `packages/novel-spec/docs/` | CLI 只读副本、软层架构、方法论映射、开发指南 |

## 常用入口

- 先读项目是什么：[`./项目启动书.md`](./项目启动书.md)
- 查需求边界：[`./需求说明书.md`](./需求说明书.md)
- 查项目级架构：[`./架构设计.md`](./架构设计.md)
- 查设计原因：[`./设计理念.md`](./设计理念.md)
- 查方法论内核：[`./创世观对应关系.md`](./创世观对应关系.md)
- 查 CLI 命令契约：硬层主副本 [`../packages/novel-lifecycle-cli/docs/CLI使用手册.md`](../packages/novel-lifecycle-cli/docs/CLI使用手册.md)，软层只读副本 [`../packages/novel-spec/docs/CLI使用手册.md`](../packages/novel-spec/docs/CLI使用手册.md)

## 访问规则

软层开发只允许参考 `packages/novel-spec/docs/` 与根 `docs/` 的项目级文档；不得依赖硬层内部文档。硬层内部变更只要不改变 CLI 使用手册，软层无需感知。
