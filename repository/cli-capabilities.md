# CLI 能力导航

> 本文件是「哪些 CLI 能力已落地 + 它们的契约/进度指针在哪」的导航地图。
> **只给指针，不复制契约取值**：退出码/错误码/schemaVersion/view-scope 的实际取值以 `contracts/cli/` 为 SSOT。
> **不记进度时间线**：实现状态与测试数属 `repository/dev-progress.md` §4，本文件只标落地状态 + 指针。
> 条目结构见 `.claude/skills/openspec-extract-facts/references/capabilities-template.md`。

---

## cli-contract

- **状态**：已落地（四块 SSOT 已收口）
- **命令组**：受契约约束的全部命令稳定面（`init`/`info`、`setting`、`constraint`、`chapter`、`verdict`、`volume`、`mainplot`、`thread`、`emergence`、`export`、`verify`）
- **契约指针**：`contracts/cli/{commands,exit-codes,error-codes,json-output,entity-ids-sourcerefs,views-scopes,state-machine,verdicts}.md`（SSOT——参数/输出/退出码/错误码/schemaVersion/view-scope 取值以这些文件为准）
- **进度指针**：`repository/dev-progress.md` §4（各命令组实现状态与测试数）
- **来源 change**：`openspec/changes/archive/2026-06-25-align-exit-codes-ssot`（exit-codes SSOT 收口）等 5 个 align-* 治理 change + `cli-spine-init-info`（业务脊柱）
