# CLI Contract

> CLI Contract 是 `novel` CLI 硬层与 Spec 运行层之间的**唯一稳定边界**。来源：`CLAUDE.md` §9。

本目录固化 CLI 的稳定契约面：命令语法、`--json` 输出结构、结构化错误对象、退出码语义、JSON Schema、golden example 与契约测试。Spec 运行层只能通过子进程调用 `novel` 命令并消费这里的稳定机器输出。

## 稳定契约（跨层承诺）

- `novel` 命令名称与参数语义。
- `--json` 输出结构（顶层对象、字段名）。
- `schemaVersion` 字段（所有 `--json` 输出必含，单一数据源）。
- 退出码类别语义。
- 结构化错误码。
- 受控 view / scope 名称及其机器语义。

## 非稳定实现细节（不作为契约，可自由演进）

- SQLite schema、迁移文件名与表结构。
- `.novel/` 内部路径与正文文件布局。
- Rust module 名称、repository / storage 实现。
- migration 具体文件名与内部存储格式。

> 判断标准：Spec 是否依赖某项内容？若依赖，则它属于稳定契约面，应在本目录固化；若仅 CLI 内部使用，则为实现细节，不写入契约。

## 目录内容

| 路径 | 职责 |
|---|---|
| `commands.md` | 各命令的语法、参数、`--json`/`--path` 语义 |
| `json-output.md` | `--json` 输出纯净性、`schemaVersion` 规则、结构化错误对象字段 |
| `exit-codes.md` | 退出码类别定义与各命令使用哪些退出码（退出码 SSOT） |
| `error-codes.md` | 错误码字符串语义定义与错误码↔退出码映射（错误码 SSOT） |
| `views-scopes.md` | 受控 view/scope 名称及其机器语义定义（受控名称 SSOT） |
| `json-schemas/` | 机器可验证的 JSON Schema |
| `examples/` | 固化的 golden example 期望样本 |
| `contract-tests/` | 契约测试，校验实际输出符合 schema 且匹配 golden |

## 当前覆盖范围

- **`cli-spine-init-info` 切片**：固化 `novel init` 与 `novel info` 两个命令的契约面，建立退出码与错误码体系框架。
- **`s01-project-bootstrap` 切片**：固化 `novel setting set/get/list` 与 `novel constraint define/render/list` 六个命令的契约面。
- **`s02-chapter-content` 切片**：固化 `novel chapter create/update/get/list/reimport/status` 六个命令的契约面。
- **`s03-rule-checking-engine` 切片**：固化 `novel constraint check`（写/查两种形态）的契约面。
- **`s04-state-machine-and-verdicts` 切片**：固化 `novel chapter status` 状态机与 `novel verdict set-mainplot-landing` 的契约面。
- **`s05-volume-management` 切片**：固化 `novel volume create/info/list/seal/archive` 与 `novel volume goal set/get/remove` 的契约面。
- **`s06-destiny-weaving` 切片**：固化 `novel mainplot node *`、`novel thread *`、`novel emergence *` 的契约面。
- **`s07-archive-and-export` 切片**：固化 `novel export volume/chapter/full` 与 `novel verify integrity` 的契约面。
- **`setting-constraint-render` 切片**（2026-07-21）：固化 `novel setting render --view` 与 `novel constraint render --view/--scope` 三组受控承载命令的契约面。
- **`chapter-context-packet` 切片**（2026-07-22）：固化 `novel chapter context --view`（P0/P1/audit/overflow 只读上下文包）的契约面。

当前 `commands.md` 共记录 46 条命令的完整契约（命令语法、JSON schema、golden examples、退出码、错误码）。`views-scopes.md §A` 冻结的三组承载命令（`setting render --view`、`constraint render --view/--scope`、`chapter context --view`）均已实现可调用，Spec 现在可依赖。实现进度详情（切片归档、命令实现矩阵、未实现设计实体清单）见 `repository/dev-progress.md`。
