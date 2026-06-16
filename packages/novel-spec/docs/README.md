# novel-spec docs

软层文档目录，面向 skill / agent / hook / rules 开发。

| 文档 | 用途 |
|------|------|
| [`CLI使用手册.md`](./CLI使用手册.md) | 从硬层主副本同步的只读契约副本；软层开发只参考此文件调用 CLI |
| [`软层架构.md`](./软层架构.md) | skill / agent / hook / rules 组织与运行流 |
| [`方法论映射.md`](./方法论映射.md) | 《创作即创世》方法论到 Claude Code 原语的映射 |
| [`开发指南.md`](./开发指南.md) | 契约隔离规则、CLI 调用最佳实践、违规后果 |
| [`参考-CC原语能力边界.md`](./参考-CC原语能力边界.md) | Claude Code 四原语能力边界参考 |

## 同步 CLI 使用手册

硬层 CLI 命令面变更后，在仓库根目录运行：

```bash
npm run sync-cli-manual
npm run verify-isolation
```

不得直接修改本目录下的 `CLI使用手册.md`；应修改硬层主副本后同步。
