# CLI JSON Output Contract

> 约定 `novel --json` 输出的纯净性、`schemaVersion` 规则与结构化错误对象字段。来源：`CLAUDE.md` §9、cli-contract spec。

## 1. JSON 输出纯净性

带 `--json` 的命令输出 MUST 满足：

- stdout 全部内容为**单一合法 JSON 对象**（或数组，视命令），可被标准 JSON 解析器无报错解析。
- 不得混入 ANSI 颜色码（如 `\x1b[...m`）。
- 不得混入解释性旁白、人读提示或多余文本。
- 诊断信息、进度提示进入 stderr，不进 stdout JSON。
- 人读输出（无 `--json`）与 JSON 输出语义一致，只是格式不同。

## 2. schemaVersion 契约（SSOT）

> **SSOT 声明**：本节是 `schemaVersion` 的唯一权威（出现规则 + 单一数据源 + 演进策略）。
> 仓库内其余文档（含 `docs/`、`contracts/cli/` 其他文件）MUST NOT 重复定义 `schemaVersion` 演进策略表（版本号格式、bump 条件、旧版本消费方检测拒绝），SHALL 以引用方式指向本节。

### 2.1 出现规则

- 所有面向机器消费的 `--json` 输出，顶层对象 MUST 包含 `schemaVersion` 字段。
- `schemaVersion` 值为**非空字符串**（如 `"1"`）。
- `schemaVersion` 与 SQLite 中记录的版本保持**单一数据源**一致：CLI 直接读 `PRAGMA user_version`，不得双轨维护。
- Spec 层消费机器输出时 SHOULD 校验 `schemaVersion` 是否为其支持的版本。

### 2.2 版本号格式

- `schemaVersion` 值为**十进制整数字符串**（匹配 `^[0-9]+$`，如 `"1"`、`"2"`、`"3"`）。
- MUST NOT 使用带前缀形态（如 `"v1"`）。
- MUST NOT 使用多段形态（如 `"1.0"`、`"cli-context.v1"`）。
- 依据：实现侧 `PRAGMA user_version` 返回整数，字符串化后自然为十进制整数；已固化 golden（`examples/info.golden.json` 的 `"schemaVersion": "1"`）已隐含此形态，本节把隐含形态显式化为契约约束。

### 2.3 bump 条件

判定原则：契约演进的核心信号是"旧消费方会不会断"。

**破坏性变更**（MUST bump `schemaVersion`）：

- `--json` 顶层结构形态改变（如顶层从对象改为数组、顶层对象拆分）。
- 必填字段集缩减（移除已有必填字段）或必填字段语义重定义（同一字段名含义不兼容变更）。
- 字段类型跨入不兼容域（如字符串字段改为对象、数值改为字符串且消费方按数值解析）。

**非破坏性变更**（不 bump，可在文档/changelog 记录）：

- 新增可选字段（消费方忽略未知字段即可）。
- 新增可选枚举值（消费方按"未知值当不认识"处理）。
- 向后兼容的语义细化（字段语义在原域内扩展，不破坏旧消费方）。

**bump 动作**：bump 时 `PRAGMA user_version` 递增 + `schemaVersion` 字符串随之递增（单一数据源保证二者一致）。bump 不跳号、不复用已废弃号。具体边界由各切片按上述原则判定，本节不枚举全量清单。

### 2.4 旧版本消费方检测拒绝策略

- Spec 层消费 `--json` 时 MUST 校验 `schemaVersion` 是否为其支持的版本集合（集合外的版本，无论高于上限或低于下限，均属不支持）。
- 检测到不支持的版本时 MUST 停止后续流程推进，MUST NOT 静默降级、MUST NOT 猜测语义、MUST NOT 对旧版本输出做隐式字段补全。
- CLI 自身 MUST NOT 对旧版本输出做向后兼容的隐式转换或伪造旧版本输出——`schemaVersion` 字段如实反映当前 `PRAGMA user_version`。
- 边界：`schemaVersion` 不匹配在 CLI 侧不是错误（CLI 输出了正确版本），在 Spec 侧才是 hard 阻断。Spec 拒绝表现为 Spec 流程中断信号（非 CLI 退出码）——CLI 退出码管的是 CLI 自身命令的成功/失败，版本不匹配的处置由 Spec 侧拒绝承担，不由 CLI 侧降级承担。
- 与 `CLAUDE.md` §8（Spec 不得忽略 CLI 非零退出码继续推进）、§10（任何 hard 阻断应通过非零退出码返回，不允许静默降级）一致：`schemaVersion` 不匹配属 hard 阻断。

## 3. 结构化错误对象

命令失败（非零退出码）时 SHALL 输出结构化错误对象。`--json` 时为 JSON 对象，否则人读。

缺省错误协议为 **V2**（`contract-registry-v2-05` 默认化，`ErrorProtocolVersion::default()` 为 V2）：`--json` 默认输出 `V2ErrorEnvelope`，`exitCode` 由错误类别推导。V1 `ErrorObject` 为**弃用兼容路径**，仅在显式选择 `ErrorProtocolVersion::V1` 时输出且逐字保持 V1 回归基线。版本选择契约与字段/取值语义见 `error-protocol-v2` spec 与 `ErrorCatalog`（权威 SSOT），本文件不重复定义表。

### 错误对象字段

| 字段 | 必填 | 说明 |
|---|---|---|
| `code` | 是 | 结构化错误码（稳定字符串，snake_case，如 `"resource_conflict"`，码值定义见 `error-codes.md`）。 |
| `message` | 是 | 人读错误说明。 |
| `exitCode` | 是 | 对应退出码（与进程退出码一致）。 |
| `suggestion` | 否 | 修复建议。 |
| `details` | 否 | 结构化附加信息。 |

### 约束

- 错误对象的 `exitCode` MUST 与进程实际退出码一致。
- 错误对象不得混入非结构化提示或 ANSI。
- 成功输出的 JSON 与错误对象的 JSON 字段集合不同，消费方据 `exitCode` 区分。
- **输出流分配**：`--json` 模式下结构化错误对象输出到 **stdout**（与成功 JSON 同流，消费方据 `exitCode` 区分成功/失败）；无 `--json` 时人读错误输出到 **stderr**。诊断信息、进度提示一律进 stderr，不进 stdout JSON。
- **`novel verify integrity` hard 违规的 `details` 承载**：完整性验证存在 hard 违规（hash 失配/正文缺失/悬空引用/状态不一致任一）时，错误对象 `details` 字段承载完整验证报告（与成功路径报告同构：`schemaVersion` / `verifiedAt` / `hashIntegrity` / `referenceIntegrity` / `stateConsistency`，结构定义见 `json-schemas/verify-integrity.schema.json`），让 Spec 经错误对象获取全部失败项而不依赖裸退出码、不需二次查询。hard 违规时 stdout 只输出结构化错误对象，不重复输出验证报告 JSON（报告失败详情由 `details` 承载）。`code` / `exitCode` 取值与退出码语义见 `exit-codes.md` / `error-codes.md` 的 `verify integrity` 段，本约定不改其取值或优先级。
- **V2 失败响应（版本化信封，缺省）**：`--json` 缺省输出 V2 错误信封（`V2ErrorEnvelope`：`schemaVersion` / `status=failed` / `errorId` / `category` / `exitCode` / `message`，可选 `recovery` / `details`；`exitCode` 由错误类别推导）。**V1 `ErrorObject` 为弃用兼容路径**：显式选择 `ErrorProtocolVersion::V1` 时输出，其 `code`/`message`/`exitCode`/`suggestion` 逐字保持 V1 回归基线；未登记的 V2 错误 ID 退化输出 V1 对象（不损坏机器契约）。字段语义、类别与退出码映射以 `error-protocol-v2` spec 与 `ErrorCatalog` 为权威源，本文件不重复定义。

## 4. 本切片已固化的 JSON Schema 与 golden

| 路径 | 说明 |
|---|---|
| `json-schemas/info.schema.json` | `novel info --json` 成功输出结构约束 |
| `examples/info.golden.json` | `novel info --json` 成功输出固化样本 |
| `examples/init-conflict.error.golden.json` | `novel init` 冲突错误对象固化样本 |
| `json-schemas/chapter.schema.json` | `novel chapter` 相关命令 JSON 输出结构约束 |
| `examples/chapter-create-ok.json` | `novel chapter create --json` 成功输出固化样本（不传 `--volume`，无 `volumeId`） |
| `examples/chapter-create-with-volume.golden.json` | `novel chapter create --volume <id> --json` 成功输出固化样本（含 `volumeId`） |
| `examples/chapter-update-ok.golden.json` | `novel chapter update --json` 成功输出固化样本 |
| `examples/chapter-get-ok.json` | `novel chapter get --json` 成功输出固化样本 |
| `examples/chapter-list-ok.json` | `novel chapter list --json` 成功输出固化样本 |
| `examples/chapter-status-ok.golden.json` | `novel chapter status --json` 成功输出固化样本 |

契约测试对 golden 的匹配严格度区分两类：

- **成功输出 golden**（`examples/info.golden.json`、`examples/chapter-*.json`）：字段级严格匹配，仅忽略易变字段（`createdAt`、`updatedAt` 等）。
- **错误对象 golden**（`examples/init-conflict.error.golden.json`）：仅严格校验结构化字段 `code` 与 `exitCode`；`message` / `suggestion` 含人读说明与动态信息（如冲突目录路径，随运行环境变化），不作为严格匹配字段，契约测试不断言其具体文本。

> 原因：错误对象的 `message` 为人读说明且常含运行期动态值，无法做值级固化；结构化契约由 `code` / `exitCode` 承载，二者稳定可严格匹配。

## 5. Chapter 相关 JSON Schema

### 5.1 `novel chapter create --json`

```json
{
  "schemaVersion": "<string>",
  "id": "<string>",
  "title": "<string>",
  "volumeId": "<stringified integer, 仅传入 --volume 时出现>",
  "createdAt": "<ISO8601 timestamp>"
}
```

> `volumeId` 为可选字段：传入 `--volume <id>` 且成功时输出（字符串化整数，如 `"1"`）；不传 `--volume` 时不出现该字段。

### 5.2 `novel chapter update --json`

Schema definition: `json-schemas/chapter.schema.json#/definitions/chapterUpdate`

```json
{
  "schemaVersion": "<string>",
  "id": "<string>",
  "contentHash": "<string: 写入后正文 SHA-256 hex>",
  "wordCount": <integer>,
  "updatedAt": "<ISO8601 timestamp>"
}
```

成功回执只在正文原子替换、元数据 transaction 与既有一致性处理全部成功后输出；不回显正文或内部文件路径。不带 `--json` 时保持静默成功。

### 5.3 `novel chapter get --json`

```json
{
  "schemaVersion": "<string>",
  "id": "<string>",
  "title": "<string>",
  "content": "<string>",
  "contentHash": "<string: 正文文件 SHA-256 hex>",
  "wordCount": <integer>,
  "status": "<outline|drafting|review|approved>",
  "volumeId": "<stringified integer, 仅章节已绑定卷时出现>",
  "createdAt": "<ISO8601 timestamp>",
  "updatedAt": "<ISO8601 timestamp>"
}
```

> `contentHash` 为正文内容的 SHA-256（十六进制），与 SQLite 存储的 hash 一致，用于外部修改检测。
> `status` 为章节状态机当前状态（四态之一，见 `state-machine.md`）。
> `volumeId` 为可选字段：仅当章节已绑定卷时输出（字符串化整数，如 `"1"`）；未绑卷时不出现。

### 5.4 `novel chapter list --json`

```json
{
  "schemaVersion": "<string>",
  "chapters": [
    {
      "id": "<string>",
      "title": "<string>",
      "wordCount": <integer>,
      "createdAt": "<ISO8601 timestamp>",
      "updatedAt": "<ISO8601 timestamp>"
    }
  ]
}
```

当使用 `--verify-hash` 时，每个章节对象增加：

```json
{
  "id": "<string>",
  "title": "<string>",
  "wordCount": <integer>,
  "createdAt": "<ISO8601 timestamp>",
  "updatedAt": "<ISO8601 timestamp>",
  "hashVerified": <boolean>,
  "hashMismatch": <boolean>
}
```

### 5.5 `novel constraint check --list --json`

Schema: `json-schemas/constraint-check-list.schema.json`

```json
{
  "schemaVersion": "<string>",
  "chapterId": "<string>",
  "currentContentHash": "<string>",
  "checks": [
    {
      "id": <integer>,
      "constraintType": "<string>",
      "constraintScope": "<string>",
      "constraintName": "<string>",
      "contentHash": "<string>",
      "severity": "<info|warning|hard>",
      "passed": <boolean>,
      "description": "<string|null>",
      "location": "<string|null>",
      "checkedAt": "<ISO8601 timestamp>",
      "stale": <boolean>
    }
  ]
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（来自 `PRAGMA user_version`）。
- `chapterId`：章节 ID。
- `currentContentHash`：章节当前正文 hash。
- `checks`：检查结果数组（按 `checkedAt` 降序）。
  - `id`：检查结果 ID（自增主键）。
  - `constraintType`：约束类型（如 `plot`、`character`）。
  - `constraintScope`：约束作用域（如 `world`、`volume:v1`）。
  - `constraintName`：约束名称。
  - `contentHash`：执行检查时的正文 hash。
  - `severity`：严重等级（`info`、`warning`、`hard`）。
  - `passed`：是否通过检查（`true` 通过，`false` 违规）。
  - `description`：检查结果描述或违规信息（可选）。
  - `location`：违规位置（如行号、段落索引，可选）。
  - `checkedAt`：检查时间戳（ISO 8601 格式）。
  - `stale`：检查结果是否过期（`stale = (contentHash != currentContentHash)`）。

使用 `--verify-hash` 时，如果存在任何 `stale = true` 的检查结果，命令返回退出码 5，错误对象 `code: "stale_rule_checks"`。

### 5.6 `novel chapter status --json`

`novel chapter status <id> <target-status> --json` 的输出结构由 `json-schemas/chapter.schema.json#/definitions/chapterStatus` 覆盖。输出顶层对象含：`schemaVersion`、`id`、`status`、`updatedAt`。

### 5.7 `novel init --json`

`novel init` 的成功输出非 Spec 机器消费面（Spec 通过 `novel info` 探测世界，不依赖 init 的成功输出），不为其固化独立 JSON schema；仅固化冲突错误对象 golden（`examples/init-conflict.error.golden.json`）。`novel init --json` 成功输出字段（`schemaVersion`、`projectName`、`dbPath`、`createdAt`、`worldRoot`）记录于 `commands.md`，但不作为机器可验证契约面。

> **契约边界（既定设计）**：Spec 层 MUST 通过 `novel info --json` 探测世界状态，MUST NOT 依赖 `novel init --json` 的成功输出形态作为机器消费依据。`init` 成功无独立 JSON schema 是既定契约设计（非缺失），本节显性化确认该边界；仅 `init` 冲突 error golden 被契约化。

## 6. Volume Management JSON Output

### 6.1 `novel volume info <id> --json`

Schema: `json-schemas/volume-info.schema.json`

```json
{
  "schemaVersion": "<string>",
  "volumeId": <integer>,
  "name": "<string>",
  "description": "<string|null>",
  "order": <integer>,
  "status": "<active|sealing|sealed|archived>",
  "createdAt": "<ISO8601 timestamp>",
  "updatedAt": "<ISO8601 timestamp>",
  "sealedAt": "<ISO8601 timestamp|null>",
  "archivedAt": "<ISO8601 timestamp|null>"
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（十进制整数字符串，来自 `PRAGMA user_version`，遵循 §2.2）。
- `volumeId`：卷 ID（整数，自增主键）。
- `name`：卷名称（唯一）。
- `description`：卷描述（可选）。
- `order`：卷序号（用于排序，允许重复）。
- `status`：卷状态（`active`/`sealing`/`sealed`/`archived`）。
- `createdAt`：创建时间戳（ISO 8601 格式）。
- `updatedAt`：更新时间戳（ISO 8601 格式）。
- `sealedAt`：封卷时间戳（仅 `sealing`/`sealed`/`archived` 状态有值）。
- `archivedAt`：归档时间戳（仅 `archived` 状态有值）。

### 6.2 `novel volume list --json`

Schema: `json-schemas/volume-list.schema.json`

```json
{
  "schemaVersion": "<string>",
  "volumes": [
    {
      "schemaVersion": "<string>",
      "volumeId": <integer>,
      "name": "<string>",
      "description": "<string|null>",
      "order": <integer>,
      "status": "<active|sealing|sealed|archived>",
      "createdAt": "<ISO8601 timestamp>",
      "updatedAt": "<ISO8601 timestamp>",
      "sealedAt": "<ISO8601 timestamp|null>",
      "archivedAt": "<ISO8601 timestamp|null>"
    }
  ]
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（十进制整数字符串，来自 `PRAGMA user_version`，遵循 §2.2）。
- `volumes`：卷数组，按 `order` 升序排序。每个元素结构同 `novel volume info --json`。

### 6.3 `novel volume goal get <volume-id> --json`

Schema: `json-schemas/volume-goals.schema.json`

```json
{
  "schemaVersion": "<string>",
  "volumeId": <integer>,
  "goals": [
    {
      "id": <integer>,
      "mainplotNodeId": "<string>",
      "description": "<string|null>",
      "isMandatory": <boolean>,
      "createdAt": "<ISO8601 timestamp>"
    }
  ]
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（十进制整数字符串，来自 `PRAGMA user_version`，遵循 §2.2）。
- `volumeId`：卷 ID。
- `goals`：目标数组，按 `id` 升序排序。
  - `id`：目标 ID（自增主键）。
  - `mainplotNodeId`：主线节点 ID（引用 s06 主线节点表）。
  - `description`：目标描述（可选）。
  - `isMandatory`：是否必达目标（`true` 必达，`false` 可选）。
  - `createdAt`：创建时间戳（ISO 8601 格式）。

### 6.4 `novel volume seal <id> [--dry-run] --json`

Schema: `json-schemas/volume-seal-check.schema.json`

```json
{
  "schemaVersion": "<string>",
  "volumeId": <integer>,
  "dryRun": <boolean>,
  "canSeal": <boolean>,
  "checkResults": [
    {
      "checkType": "<string>",
      "passed": <boolean>,
      "message": "<string>",
      "details": ["<string>"]
    }
  ]
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（十进制整数字符串，来自 `PRAGMA user_version`，遵循 §2.2）。
- `volumeId`：卷 ID。
- `dryRun`：是否为干运行模式（`true` 干运行，`false` 正式封卷）。
- `canSeal`：是否可以封卷（`true` 所有检查通过，`false` 存在阻断）。
- `checkResults`：检查结果数组。
  - `checkType`：检查类型（如 `"章节状态"`、`"涌现处置"`、`"卷目标达成"`）。
  - `passed`：检查是否通过。
  - `message`：检查结果摘要信息。
  - `details`：具体阻断详情数组（如未 approved 的章节列表）。

当 `canSeal = false` 时，命令返回退出码 3（前置条件阻断）。

## 7. Export & Verify JSON Output

### 7.1 `novel export volume <id> --json`

Schema: `json-schemas/export-volume.schema.json`

```json
{
  "schemaVersion": "<string>",
  "scope": "volume",
  "exportedAt": "<ISO8601 timestamp>",
  "volume": {
    "volumeId": <integer>,
    "name": "<string>",
    "description": "<string|null>",
    "order": <integer>,
    "status": "<active|sealing|sealed|archived>",
    "createdAt": "<ISO8601 timestamp>",
    "updatedAt": "<ISO8601 timestamp>",
    "sealedAt": "<ISO8601 timestamp|null>",
    "archivedAt": "<ISO8601 timestamp|null>"
  },
  "chapters": [
    {
      "id": "<string>",
      "title": "<string>",
      "content": "<string>",
      "contentHash": "<string>",
      "wordCount": <integer>,
      "status": "<outline|drafting|review|approved>",
      "createdAt": "<ISO8601 timestamp>",
      "updatedAt": "<ISO8601 timestamp>"
    }
  ],
  "emergences": [<emergence object>],
  "mainplotNodes": [<mainplot node object>],
  "threadBindings": [<thread binding object>],
  "verdicts": [<verdict object>],
  "ruleChecks": [<rule check object>],
  "warnings": ["<string>"]
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（十进制整数字符串，来自 `PRAGMA user_version`，遵循 §2.2）。
- `scope`：导出范围，固定为 `"volume"`。
- `exportedAt`：导出操作时间戳（ISO 8601）。导出为只读操作，不写入 `updated_at`。
- `volume`：卷元数据，结构同 `novel volume info --json` 的卷体（无重复 `schemaVersion`）。
- `chapters`：卷内章节数组，每个章节 `content` 字段以字符串嵌入正文，`contentHash` 为存储的正文 hash。
- `emergences`/`mainplotNodes`/`threadBindings`/`verdicts`/`ruleChecks`：卷内下属实体数组，结构对应各自 `--json` 输出体。
- `warnings`：提示数组。卷非 `archived` 状态时含 `"卷未归档，导出为非冻结快照"` 类提示；归档卷为空数组。

### 7.2 `novel export chapter <id> --json`

Schema: `json-schemas/export-chapter.schema.json`

```json
{
  "schemaVersion": "<string>",
  "scope": "chapter",
  "exportedAt": "<ISO8601 timestamp>",
  "chapter": {
    "id": "<string>",
    "title": "<string>",
    "content": "<string>",
    "contentHash": "<string>",
    "wordCount": <integer>,
    "status": "<outline|drafting|review|approved>",
    "createdAt": "<ISO8601 timestamp>",
    "updatedAt": "<ISO8601 timestamp>"
  },
  "verdicts": [<verdict object>],
  "ruleChecks": [<rule check object>]
}
```

字段说明：

- `scope`：固定为 `"chapter"`。
- `chapter.content`：正文以字符串嵌入。导出时重算正文 SHA-256 与 `contentHash` 对比，失配返回退出码 21（`content_modified_externally`），不输出本对象。
- `verdicts`/`ruleChecks`：章节关联的裁决与规则检查结果数组。

### 7.3 `novel export full --json`

Schema: `json-schemas/export-full.schema.json`

```json
{
  "schemaVersion": "<string>",
  "scope": "full",
  "exportedAt": "<ISO8601 timestamp>",
  "volumes": [<volume export body, 同 7.1 的 volume-level 结构>],
  "warnings": [
    {
      "chapterId": "<string>",
      "storedHash": "<string>",
      "actualHash": "<string>"
    }
  ]
}
```

字段说明：

- `scope`：固定为 `"full"`。
- `volumes`：所有卷的导出体数组，每个元素结构同 7.1 的卷导出体。
- `warnings`：hash 失配项数组。全量遍历时逐章节重算 hash，失配项列入此处（含 `chapterId`/`storedHash`/`actualHash`）；全量导出不因 hash 失配阻断（退出码仍为 0），需 hard 阻断式校验请用 `novel verify integrity`。空世界 `volumes: []`、`warnings: []`，退出码 0。

### 7.4 `novel verify integrity --json`

Schema: `json-schemas/verify-integrity.schema.json`

```json
{
  "schemaVersion": "<string>",
  "verifiedAt": "<ISO8601 timestamp>",
  "hashIntegrity": {
    "passed": <boolean>,
    "mismatches": [
      {
        "chapterId": "<string>",
        "storedHash": "<string>",
        "actualHash": "<string>"
      }
    ],
    "missingFiles": [
      {
        "chapterId": "<string>",
        "expectedPath": "<string>"
      }
    ]
  },
  "referenceIntegrity": {
    "passed": <boolean>,
    "danglingReferences": [
      {
        "entityType": "<string>",
        "entityId": "<string>",
        "referencedEntityType": "<string>",
        "referencedId": "<string>"
      }
    ]
  },
  "stateConsistency": {
    "passed": <boolean>,
    "inconsistencies": [
      {
        "type": "<stale_verdict|archived_volume_has_non_approved_chapter|...>",
        "chapterId": "<string|null>",
        "volumeId": "<integer|null>",
        "boundHash": "<string|null>",
        "currentHash": "<string|null>"
      }
    ]
  }
}
```

字段说明：

- `schemaVersion`：当前 schema 版本（十进制整数字符串，来自 `PRAGMA user_version`，遵循 §2.2）。
- `verifiedAt`：验证时间戳（ISO 8601）。验证为只读操作，不更新任何 `updated_at`。
- `hashIntegrity.passed`：正文 hash 全部一致时为 `true`。
- `hashIntegrity.mismatches`：hash 失配项（`chapterId`/`storedHash`/`actualHash`）。
- `hashIntegrity.missingFiles`：正文文件缺失项（`chapterId`/`expectedPath`）。
- `referenceIntegrity.passed`：所有实体间引用有效时为 `true`。
- `referenceIntegrity.danglingReferences`：悬空引用项（实体类型 + 实体 ID + 被引用实体类型 + 被引用 ID）。
- `stateConsistency.passed`：状态机全部一致时为 `true`。
- `stateConsistency.inconsistencies`：不一致项（`type` + 相关实体 ID 与状态/hash）。`type` 取值如 `stale_verdict`（裁决绑定旧 hash）、`archived_volume_has_non_approved_chapter`（归档卷内含非 approved 章节）。
- 任一分项 `passed=false` 即非零退出：取最严重的一项作为进程退出码（hash 失配 21、正文缺失 11、引用悬空 70、状态不一致 5）。

## 8. 其他命令族输出结构

以下命令族的 `--json` 输出结构未在本文件 inline，字段定义与 schema 见：

- **mainplot node**（create/list/get/update）：`commands.md` 各条目字段表 + `json-schemas/mainplot-node.schema.json`、`mainplot-node-list.schema.json`
- **thread**（create/bind/list/query/update）：`commands.md` 各条目 + `json-schemas/thread.schema.json`、`thread-list.schema.json`、`thread-binding-list.schema.json`
- **emergence**（create/rate/decide/execute/archive/list/get）：`commands.md` 各条目 + `json-schemas/emergence.schema.json`、`emergence-list.schema.json`
- **verdict**（set-mainplot-landing）：`commands.md` 条目 + `json-schemas/verdict.schema.json`

> 这些族与 §5–§7 遵循同一 JSON 纯净性（§1）、schemaVersion（§2）、错误对象（§3）约定；字段级结构以 `commands.md` + `json-schemas/` 为准，golden 见 `examples/`。
