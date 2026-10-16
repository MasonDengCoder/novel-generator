# CLI Entity IDs & sourceRefs Format Contract

> **本文件是 `novel` CLI sourceRefs 字符串格式与实体 ID 前缀表的唯一事实来源（SSOT）。** 仓库内其余文档（含 `docs/` 与 `reference/`）不得重复定义 sourceRefs 字符串格式契约或实体类型前缀表，只能引用本文件。
> sourceRefs 字符串格式是稳定契约（Spec 消费链最底层引用契约）。来源：`CLAUDE.md` §9"实体 ID 与引用规则属稳定契约"，cli-contract spec。
> 每个 fragment 的 `sourceRefs` 字段必须指向 CLI 稳定实体 ID，而不是内部文件路径。

sourceRefs 格式分两层：**§A 生效格式契约**（字符串格式与实体类型前缀表，从 docs JSON 示例推断已稳定，Spec 可依赖）与 **§B 留实现**（实体 ID 生成规则，属 domain 层实体 ID 策略，依附存储实现，对应切片实现并通过测试前不固化，Spec 不得依赖）。

> 本文件与 `contracts/cli/exit-codes.md`、`contracts/cli/error-codes.md` 并列对称：三者都是把散落在 docs / 实现的契约抽到 `contracts/cli/` 固化的 SSOT。本文件管 sourceRefs 引用字符串格式与前缀表，不重复定义退出码或错误码语义。

## §A 生效格式契约

> 以下格式与六类前缀从 `docs/CLI架构设计.md` §9 的多个 JSON 示例（章契约示例、context packet audit/overflow 示例）反复出现且跨示例一致推断已稳定，Spec 可依赖其形状与前缀。格式契约先于实现固化：sourceRefs 相关命令尚未实现，无法靠实现反推，但格式在 docs 示例已稳定，先行冻结是防止切片各自发明的唯一方式。

### sourceRefs 字符串格式

sourceRefs 字段中的每个元素是一个实体引用字符串，格式冻结为：

```
<entityType>:<prefix>-<number>
```

即 `<实体类型>:<前缀>-<数字>`。其中 `entityType` 与 `prefix` 取自下方前缀表六类之一，`number` 是数字部分（其位数、是否零填充、取值域属 §B 生成规则，不属本格式契约——见 §B）。

> 例：`mainplot:mp-003` 合法（`mainplot` 是登记实体类型，`mp` 是其前缀，`003` 是数字部分）；`mp-003` 不合法（缺 `<entityType>:`）；`mainplot:mp3` 不合法（缺 `-`，数字与 `<prefix>` 之间须有连字符）。

### 实体类型与前缀表

> 登记命令参数 ID 与 sourceRefs 共用的实体类型与前缀。前缀须与 commands.md 及实现中实际使用的 ID 前缀一致（命令参数 ID 形如 `<prefix>-<number>`，sourceRefs 形如 `<entityType>:<prefix>-<number>`）。后续切片实现新实体时同步把类型与前缀登记进本表。

| 实体类型 (`entityType`) | 前缀 (`prefix`) | 示例（命令参数 ID） | 示例（sourceRefs） |
|---|---|---|---|
| `mainplot` | `mp` | `mp-1-001` | `mainplot:mp-1-001` |
| `section` | `sec` | `sec-009` | `section:sec-009` |
| `event` | `ev` | `ev-011` | `event:ev-011` |
| `chapter` | `chapter` | `chapter-001` | `chapter:chapter-001` |
| `residue` | `res` | `res-019` | `residue:res-019` |
| `thread` | `th` | `th-1-001` | `thread:th-1-001` |
| `emergence` | `em` | `em-1-1` | `emergence:em-1-1` |

> §A 格式契约规定 `<prefix>-<number>` 形状与前缀表的类型，number 的位数、是否零填充、取值域属 §B 不属 §A。`chapter` 前缀以命令参数 ID 实际形态为准（`chapter-001`），sourceRefs 中 `chapter:chapter-001` 的 entityType 与 prefix 同名属合法（格式为 `<entityType>:<prefix>-<number>`，二者碰巧相同不违反形状契约）。`setting` 与 `constraint` 无数字 ID（settings 是自由 KV、constraints 是复合键），不登记于本数字前缀表，其 sourceRefs 形态见下方"按复合键的字符串引用形态"表。

### 按复合键的字符串引用形态（并列补充形态）

> 上述 `<entityType>:<prefix>-<number>` 形态适用于有数字 ID 的实体（mainplot/section/event/chapter/residue/thread/emergence）。`settings` 表是自由 KV（`name` 主键，无数字 ID）、`constraints` 表是复合键 `(type, scope, name)`（无数字 ID），套不进 `<prefix>-<number>` 形状，故 `setting` 与 `constraint` 不登记于上方数字前缀表，其 sourceRefs 形态见下方复合键表。对这类无数字 ID 的自由 KV / 复合键实体，sourceRefs 采用**按复合键的字符串引用**形态，作为 §A 既有 `<prefix>-<number>` 形态的**并列补充形态**（不替换既有形态、不改既有实体 sourceRefs）。

| 实体类型 (`entityType`) | 引用形态 | 示例（sourceRefs） | 适用命令 |
|---|---|---|---|
| `setting` | `setting:<name>` | `setting:world-setting` | `novel setting render --view` |
| `constraint` | `constraint:<type>:<scope>:<name>` | `constraint:plot:world:时间线一致` | `novel constraint render --view/--scope` |
| `verdict` | `verdict:<chapter-id>:<node-id>` | `verdict:chapter-001:mp-1-001` | `novel chapter context --view` |
| `volume_goal` | `volume_goal:<volume-id>:<goal-id>` | `volume_goal:1:3` | `novel chapter context --view` |

> `verdict:<chapter-id>:<node-id>` 中 `chapter-id` 与 `node-id` 是裁决的复合定位键（`verdicts` 表 `(chapter_id, verdict_type, node_id)` 复合主键的一部分，`verdict_type` 当前固定 `mainplot-landing`，引用时省略 type 段以 `chapter-id:node-id` 定位，Spec 可据 chapter-id 与 node-id 解析）。`volume_goal:<volume-id>:<goal-id>` 中 `volume-id` 为卷整数 ID，`goal-id` 为卷目标整数 ID。两者均不含 rowid 与内部路径。
>
> `chapter context --view` 的其他 sourceRefs 复用既有 §A 形态：`chapter:<chapter-id>`、`thread:<thread-id>`、`mainplot:<node-id>`（均为 `<prefix>-<number>` 数字 ID 形态，已在前缀表登记）；`constraint:<type>:<scope>:<name>` 复用上方复合键形态。

> `setting:<name>` 中 `name` 是 settings 表的主键字符串（可含连字符、中文等任意非路径分隔字符）。`constraint:<type>:<scope>:<name>` 中 `type` 是约束类型字符串（如 `plot`），`scope` 是存储的 scope 字符串（如 `world`/`volume:1`/`chapter:chapter-001`），`name` 是约束名称字符串。三者均不含 `:` 之外的特殊转义；调用方按 `:` 分隔解析时取首段为 entityType，其余按实体类型规则组合为复合键。此形态不含 SQLite rowid 与 `.novel/` 内部路径，与 §A 既有形态共同满足"任何受控 view/scope 名的机器输出 MUST 使用稳定实体 ID 与 sourceRefs"约束。

## §B 留实现

> 以下实体 ID 生成规则属 domain 层实体 ID 策略，依附存储实现，**对应切片实现并通过测试前不固化**，Spec MUST NOT 依赖具体生成算法。生成规则不是当前生效契约。

实体 ID 生成规则包括但不限于：

- 数字部分的位数与取值域（如是否三位零填充、是否自增序号、是否 UUID 段）。
- 生成策略（自增序号 / UUID / 确定性派生），依附存储实现（实体 ID 生成策略属实现层未完成项，进度以 `repository/dev-progress.md` 为准）。
- §A 格式契约只规定 `<prefix>-<number>` 形状，number 的位数、是否零填充、取值域属 §B 不属 §A。Spec 消费 sourceRefs 时只需识别 `<entityType>:<prefix>-<number>` 形状与前缀，不需知道数字算法，故 §B 留实现不阻塞 Spec 消费格式契约。

### 升格路径

某对应切片实现 domain 层实体 ID 生成策略并通过契约测试后，在归档该切片时可把该生成规则从 §B 移至 §A 标注生效（升格的是生成算法不是格式形状——`<entityType>:<prefix>-<number>` 与前缀表本身是更底层契约，不随生成规则升格而变）。升格后 Spec 方可依赖该生成规则。

> 升格时须在该切片同步补充实现、golden example 与契约测试，方可视为生效。新增实体类型前缀（如 `character`、`volume`）由对应切片实现并通过测试时同步登记进 §A 前缀表，登记遵循本表现有 `<entityType>:<prefix>` 约定；登记前 Spec MUST NOT 依赖未登记的前缀。
