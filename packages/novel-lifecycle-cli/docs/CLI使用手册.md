<!-- 本文件是软硬层契约，更新后须同步到软层 docs。 -->
<!-- 本文件从硬层主副本同步；软层只读副本不得直接修改，须改硬层主副本后同步。 -->

# CLI 使用手册

本手册是 `novel-cli` 命令行工具的权威参考文档，定义所有子命令的语法、参数、输出格式和退出码。

## 目录

- [全局约定](#全局约定)
- [退出码规范](#退出码规范)
- [输出格式约定](#输出格式约定)
- [生命周期管理](#生命周期管理)
- [元素管理](#元素管理)
- [设定管理](#设定管理)
- [诊断与工具](#诊断与工具)

---

## 全局约定

### 工作目录

`novel-cli` 命令默认对**当前工作目录**下的 `index/novel.db` 进行操作。所有子命令（除 `init` 外）均假设在项目根目录执行。

### 数据库路径

- **默认路径**: `index/novel.db`（相对于当前工作目录）
- **环境变量**: `NOVEL_DB` — 设置后作为默认路径
- **命令行参数**: `--db-path <path>` — 覆盖默认路径（适用于除 `init` 外的所有子命令）

### 通用参数

以下参数适用于所有支持 JSON 输出的子命令：

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--json` | flag | 否 | false | 启用 JSON 输出模式（机器可读） |
| `--db-path` | string | 否 | `index/novel.db` | 指定数据库文件路径 |

---

## 退出码规范

`novel-cli` 使用分段退出码明确区分不同类型的错误：

| 退出码范围 | 含义 | 示例场景 |
|------------|------|----------|
| `0` | 成功 | 命令正常完成 |
| `1` | 通用失败（未分类） | 未知错误或未捕获异常 |
| `2-5` | 参数错误 | 缺失必需参数、类型错误、非法值、冲突参数 |
| `11-15` | 数据库错误 | 连接失败、查询错误、完整性约束违反 |
| `21-99` | 业务逻辑错误 | 状态机非法转换、元素冲突、不可越级修正 |

### 常见退出码

| 退出码 | 说明 |
|--------|------|
| `0` | 成功 |
| `1` | 通用失败 |
| `2` | 缺失必需参数 |
| `3` | 参数类型错误 |
| `4` | 参数值非法 |
| `21` | 状态机非法转换 |
| `22` | 业务规则违反（如越级修正） |
| `23` | 门禁检查失败 |

---

## 输出格式约定

### 默认输出（人类可读）

子命令默认输出简洁、人类可读的文本格式：
- **成功操作**: 简短确认消息（如 `Chapter started: C001`）
- **列表操作**: 表格或简洁文本列表
- **错误消息**: 输出到 `stderr`，格式为 `ERROR: <category>: <message>`

### JSON 输出模式

通过 `--json` 标志启用结构化输出：
- **成功操作**: 返回完整对象（包含 `ok: true` 和操作结果）
- **列表操作**: 返回对象数组
- **错误消息**: JSON 对象包含 `error` 字段，仍输出到 `stderr`

**示例**:
```bash
# 默认输出
$ novel-cli init
Database initialized: index/novel.db

# JSON 输出
$ novel-cli init --json
{"ok": true, "db_path": "index/novel.db"}
```

### 错误输出格式

所有错误消息输出到 `stderr`，格式固定：

```
ERROR: <category>: <message>
```

**JSON 模式下错误格式**:
```json
{"error": "<category>: <message>"}
```

---

## 生命周期管理

### `init` - 初始化项目

初始化新的小说项目，创建数据库文件和基础表结构。

**语法**:
```bash
novel-cli init [--db-path <path>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--db-path` | string | 否 | `index/novel.db` | 指定数据库文件路径 |

**输出格式**:

- **默认**: `Database initialized: <path>`
- **JSON**: `{"ok": true, "db_path": "<path>"}`

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功创建数据库 |
| `11` | 数据库创建失败 |

**示例**:

```bash
# 使用默认路径初始化
novel-cli init

# 指定数据库路径
novel-cli init --db-path /path/to/custom.db

# JSON 输出模式
novel-cli init --json
```

---

### `stage` - 阶段管理

管理创作阶段（灵感/世界边界/主线/天命四阶段）的进度和数据。

**语法**:
```bash
novel-cli stage <action> [options]
```

**子动作**:

#### `stage list` - 列出所有阶段

```bash
novel-cli stage list [--json]
```

**输出**: 阶段列表，包含 `stage_name`、`status`、`completed_at`

#### `stage complete <stage>` - 标记阶段完成

```bash
novel-cli stage complete <stage>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `stage` | string | 是 | - | 阶段名称：`inspiration`/`world-boundary`/`main-plot`/`destiny` |

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功标记完成 |
| `4` | 未知阶段名称 |
| `23` | 门禁检查失败（如 world-boundary 阶段需要先通过 boundary_validated 检查） |

#### `stage set-inspiration` - 设置灵感内容

```bash
novel-cli stage set-inspiration --content <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--content` | string | 是 | - | 灵感内容文本 |

#### `stage get-inspiration` - 获取灵感内容

```bash
novel-cli stage get-inspiration [--json]
```

**输出**: `{"key": "inspiration", "content": "<text>"}`

#### `stage set-premise` - 设置前提内容

```bash
novel-cli stage set-premise --content <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--content` | string | 是 | - | 前提内容文本 |

#### `stage get-premise` - 获取前提内容

```bash
novel-cli stage get-premise [--json]
```

**输出**: `{"key": "premise", "content": "<text>"}`

**示例**:

```bash
# 列出所有阶段
novel-cli stage list --json

# 设置灵感
novel-cli stage set-inspiration --content "一个关于时间旅行的故事..."

# 标记灵感阶段完成
novel-cli stage complete inspiration

# 标记世界边界阶段完成（需先通过边界校验）
novel-cli stage complete world-boundary
```

---

## 元素管理

<!-- 子命令详细文档将在下面补充 -->

## 设定管理

### `bible` - 世界设定管理

查看世界圣经（World Bible）的不同视图。世界圣经是数据库的派生渲染，无独立存储。

**语法**:
```bash
novel-cli bible <view> [--json]
```

**视图类型**:

| 视图 | 说明 | 包含内容 |
|------|------|----------|
| `core` | 核心设定 | 世界边界、不可变主线节点、风格声音 |
| `active` | 活跃元素 | 活跃支线、待处理涌现元素 |
| `dormant` | 休眠元素 | 非活跃支线、已处理涌现元素 |
| `full` | 完整视图 | 所有上述内容的组合 |

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|------|------|
| `view` | string | 是 | - | 视图类型：`core`/`active`/`dormant`/`full` |

**输出格式**:

所有视图均输出 JSON 格式，包含相应表的查询结果。

**core 视图输出**:
```json
{
  "world_boundaries": [...],
  "mainplot_nodes": [...],
  "style_voice": [...]
}
```

**active 视图输出**:
```json
{
  "subplots": [...],
  "emergent": [...]
}
```

**dormant 视图输出**:
```json
{
  "subplots": [...],
  "emergent": [...]
}
```

**full 视图输出**:
```json
{
  "world_boundaries": [...],
  "mainplot_nodes": [...],
  "style_voice": [...],
  "subplots": [...],
  "emergent": [...],
  "dormant": {
    "subplots": [...]
  }
}
```

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功返回视图 |
| `4` | 未知视图类型 |

**示例**:

```bash
# 查看核心设定
novel-cli bible core

# 查看活跃元素
novel-cli bible active --json

# 查看完整世界圣经
novel-cli bible full
```

---

## 诊断与工具

### `monitor` - 监控信号管理

记录和管理创作过程中的监控信号（警告、异常、检查结果等）。

**语法**:
```bash
novel-cli monitor <action> [options]
```

**子动作**:

#### `monitor signal` - 记录监控信号

```bash
novel-cli monitor signal --signal-type <type> [--chapter-id <id>] [--detail <text>] [--severity <level>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--signal-type` | string | 是 | - | 信号类型（如 `plot_drift`、`constraint_violation` 等） |
| `--chapter-id` | int | 否 | `null` | 关联的章节 ID |
| `--detail` | string | 否 | `null` | 信号详细描述 |
| `--severity` | string | 否 | `null` | 严重程度（如 `warning`、`error`） |

**输出格式**:

```json
{
  "ok": true,
  "id": <signal_id>,
  "signal_type": "<type>"
}
```

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功记录信号 |
| `2` | 缺失必需参数 `--signal-type` |

**示例**:

```bash
# 记录通用监控信号
novel-cli monitor signal --signal-type plot_drift --detail "主线偏离预期"

# 记录章节相关信号
novel-cli monitor signal --signal-type constraint_violation --chapter-id 5 --severity error

# JSON 输出
novel-cli monitor signal --signal-type timeline_conflict --json
```

---

### `prose` - 章节写作管理

管理章节的写作生命周期（开始、进度、结束）。

**语法**:
```bash
novel-cli prose <action> [options]
```

**子动作**:

#### `prose start` - 开始写作章节

```bash
novel-cli prose start <chapter_id> [--prose-ref <path>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `chapter_id` | int | 是 | - | 章节 ID |
| `--prose-ref` | string | 否 | `null` | 正文文件引用路径 |

**输出**: 返回章节约束信息 `{"ok": true, "chapter_id": <id>, "constraints": [...]}`

#### `prose phase` - 记录创作阶段

```bash
novel-cli prose phase <chapter_id> <phase>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `chapter_id` | int | 是 | - | 章节 ID |
| `phase` | string | 是 | - | 阶段：`birth`/`dwell`/`decay`/`death` |

**输出**: `{"ok": true, "chapter_id": <id>, "phase": "<phase>"}`

#### `prose end` - 结束章节写作

```bash
novel-cli prose end <chapter_id> [--prose-ref <path>] [--word-count <count>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `chapter_id` | int | 是 | - | 章节 ID |
| `--prose-ref` | string | 否 | `null` | 正文文件引用路径 |
| `--word-count` | int | 否 | `null` | 字数统计 |

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功结束章节 |
| `22` | 未委派 mainplot-landing-verifier，must 裁决全为 null |
| `23` | must 约束未落地，章节状态设为 `must_failed` |

#### `prose residue` - 添加章节残留物

```bash
novel-cli prose residue add --chapter-id <id> --type <type> --content <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--chapter-id` | int | 是 | - | 章节 ID |
| `--type` | string | 是 | - | 残留物类型 |
| `--content` | string | 是 | - | 残留物内容 |

#### `prose list-residue` - 列出章节残留物

```bash
novel-cli prose list-residue --chapter-id <id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--chapter-id` | int | 是 | - | 章节 ID |

**示例**:

```bash
# 开始写作章节 1
novel-cli prose start 1 --prose-ref chapters/ch001.md

# 记录创作阶段
novel-cli prose phase 1 birth

# 结束章节写作
novel-cli prose end 1 --word-count 3500

# 添加残留物
novel-cli prose residue add --chapter-id 1 --type plot_thread --content "未解决的线索"

# 列出残留物
novel-cli prose list-residue --chapter-id 1 --json
```

---

### `weave` - 结构编织

创建和管理小说的层级结构（卷/事件/节/章）。

**语法**:
```bash
novel-cli weave <action> [options]
```

**子动作**:

#### `weave volume` - 创建卷

```bash
novel-cli weave volume --title <title> [--mainplot-target-node-id <id>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--title` | string | 是 | - | 卷标题 |
| `--mainplot-target-node-id` | int | 否 | `null` | 关联的主线目标节点 ID |

**输出**: 返回创建的卷对象

#### `weave event` - 创建事件

```bash
novel-cli weave event --volume-id <id> --title <title> [--mainplot-node-id <id>] [--is-mainplot-critical <0|1>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--volume-id` | int | 是 | - | 所属卷 ID |
| `--title` | string | 是 | - | 事件标题 |
| `--mainplot-node-id` | int | 否 | `null` | 关联的主线节点 ID |
| `--is-mainplot-critical` | int | 否 | `0` | 是否为主线必达事件（1=是，0=否） |

**输出**: 返回创建的事件对象

#### `weave section` - 创建节

```bash
novel-cli weave section --event-id <id> --title <title> [--mainplot-must-beat <beat>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--event-id` | int | 是 | - | 所属事件 ID |
| `--title` | string | 是 | - | 节标题 |
| `--mainplot-must-beat` | string | 否 | `null` | 主线必达节拍 |

**输出**: 返回创建的节对象

#### `weave chapter` - 创建章节

```bash
novel-cli weave chapter --section-id <id> --seq <num> --title <title>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--section-id` | int | 是 | - | 所属节 ID |
| `--seq` | int | 是 | - | 章节序号 |
| `--title` | string | 是 | - | 章节标题 |

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功创建章节 |
| `22` | 第三卷及以后只允许事件级编织，不可添加章节 |

**示例**:

```bash
# 创建卷
novel-cli weave volume --title "第一卷：启程" --mainplot-target-node-id 5

# 创建事件
novel-cli weave event --volume-id 1 --title "初次相遇" --is-mainplot-critical 1

# 创建节
novel-cli weave section --event-id 1 --title "第一节"

# 创建章节
novel-cli weave chapter --section-id 1 --seq 1 --title "第一章：序幕"
```

---

### `cycle` - 卷周期管理

管理卷的封闭、事件修复、收敛信号等周期操作。

**语法**:
```bash
novel-cli cycle <action> [options]
```

**子动作**:

#### `cycle close-event` - 封闭事件

```bash
novel-cli cycle close-event <event_id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `event_id` | int | 是 | - | 事件 ID |

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功封闭事件 |
| `1` | 存在未完成章节，事件状态设为 `repairing` |

#### `cycle close-volume` - 封闭卷

```bash
novel-cli cycle close-volume <volume_id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `volume_id` | int | 是 | - | 卷 ID |

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功封闭卷 |
| `1` | 欠标嫌疑事件（有 mainplot_node_id 但未标 critical） |
| `23` | 门禁失败：缺失 target_convergence_verdict、重激活快照或存在未封闭必达事件 |

#### `cycle repair-plan` - 修复计划

```bash
novel-cli cycle repair-plan <level> <id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `level` | string | 是 | - | 层级（目前仅支持 `event`） |
| `id` | int | 是 | - | 事件 ID |

**输出**: 返回需要修复的章节列表

#### `cycle converge-signal` - 记录收敛信号

```bash
novel-cli cycle converge-signal --chapter-id <id> --volume-id <id> --converging <0|1> [--pulled-by-emergent-id <id>] [--detail <text>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--chapter-id` | int | 是 | - | 章节 ID |
| `--volume-id` | int | 是 | - | 卷 ID |
| `--converging` | int | 是 | - | 是否收敛（1=是，0=否） |
| `--pulled-by-emergent-id` | int | 否 | `null` | 牵引的涌现元素 ID |
| `--detail` | string | 否 | `null` | 详细描述 |

#### `cycle convergence-verdict` - 收敛裁决

```bash
novel-cli cycle convergence-verdict --volume-id <id> --verdict <result> [--drift-note <text>] [--target-achieved <0|1>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--volume-id` | int | 是 | - | 卷 ID |
| `--verdict` | string | 是 | - | 裁决结果 |
| `--drift-note` | string | 否 | `null` | 偏移说明 |
| `--target-achieved` | int | 否 | `null` | 目标是否达成（1=是，0=否） |

#### `cycle list-convergence` - 列出收敛信号

```bash
novel-cli cycle list-convergence --volume-id <id>
```

#### `cycle show-volume` - 显示卷详情

```bash
novel-cli cycle show-volume <volume_id>
```

#### `cycle list-events` - 列出事件

```bash
novel-cli cycle list-events --volume-id <id> [--has-node-id] [--not-critical]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--volume-id` | int | 是 | - | 卷 ID |
| `--has-node-id` | flag | 否 | false | 仅列出有 mainplot_node_id 的事件 |
| `--not-critical` | flag | 否 | false | 仅列出非必达事件 |

**示例**:

```bash
# 封闭事件
novel-cli cycle close-event 3

# 封闭卷
novel-cli cycle close-volume 1

# 查看修复计划
novel-cli cycle repair-plan event 3

# 记录收敛信号
novel-cli cycle converge-signal --chapter-id 10 --volume-id 1 --converging 1

# 收敛裁决
novel-cli cycle convergence-verdict --volume-id 1 --verdict "pass" --target-achieved 1
```

---

### `mainplot` - 主线管理

管理主线节点的创建和查询。

**语法**:
```bash
novel-cli mainplot <action> [options]
```

**子动作**:

#### `mainplot add-node` - 添加主线节点

```bash
novel-cli mainplot add-node --content <text> [--immutable <0|1>] [--order <num>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--content` | string | 是 | - | 节点内容 |
| `--immutable` | int | 否 | `0` | 是否不可变（1=是，0=否） |
| `--order` | int | 否 | `null` | 节点顺序 |

**输出**: 返回创建的主线节点对象

#### `mainplot list` - 列出主线节点

```bash
novel-cli mainplot list [--immutable-only]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--immutable-only` | flag | 否 | false | 仅列出不可变节点 |

**示例**:

```bash
# 添加不可变主线节点
novel-cli mainplot add-node --content "主角离开家乡" --immutable 1 --order 1

# 列出所有主线节点
novel-cli mainplot list --json

# 仅列出不可变节点
novel-cli mainplot list --immutable-only
```

---

### `world-boundary` - 世界边界管理

管理世界设定的硬边界和软边界。

**语法**:
```bash
novel-cli world-boundary <action> [options]
```

**子动作**:

#### `world-boundary add` - 添加边界

```bash
novel-cli world-boundary add --type <type> --content <text> [--fuzzy <0|1>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--type` | string | 是 | - | 边界类型 |
| `--content` | string | 是 | - | 边界内容描述 |
| `--fuzzy` | int | 否 | `0` | 是否为模糊边界（1=是，0=否） |

**输出**: 返回创建的边界对象

#### `world-boundary list` - 列出所有边界

```bash
novel-cli world-boundary list [--json]
```

**示例**:

```bash
# 添加硬边界
novel-cli world-boundary add --type "physics" --content "无法时间旅行" --fuzzy 0

# 添加模糊边界
novel-cli world-boundary add --type "magic" --content "魔法存在但罕见" --fuzzy 1

# 列出所有边界
novel-cli world-boundary list --json
```

---

### `presummary` - 章节前摘管理

管理章节的约束条件和裁决。

**语法**:
```bash
novel-cli presummary <action> [options]
```

**子动作**:

#### `presummary set` - 设置章节约束

```bash
novel-cli presummary set --chapter-id <id> --constraints <json>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--chapter-id` | int | 是 | - | 章节 ID |
| `--constraints` | string | 是 | - | 约束 JSON 字符串 |

#### `presummary get` - 获取章节约束

```bash
novel-cli presummary get --chapter-id <id>
```

#### `presummary update-verdict` - 更新约束裁决

```bash
novel-cli presummary update-verdict --chapter-id <id> --constraint-idx <num> --satisfied <0|1> --reason <text> --verified-by <who>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--chapter-id` | int | 是 | - | 章节 ID |
| `--constraint-idx` | int | 是 | - | 约束索引 |
| `--satisfied` | int | 是 | - | 是否满足（1=是，0=否） |
| `--reason` | string | 是 | - | 裁决理由（至少 10 字符） |
| `--verified-by` | string | 是 | - | 验证者标识 |

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功更新裁决 |
| `4` | reason 少于 10 字符或 verified-by 为空 |
| `11` | 章节不存在或索引越界 |

#### `presummary update-constraint` - 更新约束内容

```bash
novel-cli presummary update-constraint --chapter-id <id> --constraint-idx <num> --content <text>
```

**示例**:

```bash
# 设置章节约束
novel-cli presummary set --chapter-id 1 --constraints '[{"type":"must","content":"主角必须遇见导师"}]'

# 获取约束
novel-cli presummary get --chapter-id 1

# 更新裁决
novel-cli presummary update-verdict --chapter-id 1 --constraint-idx 0 --satisfied 1 --reason "已在第三场景落地" --verified-by "mainplot-verifier"
```

---

### `style` - 风格管理

管理写作风格声音和敏感词。

**语法**:
```bash
novel-cli style <action> [options]
```

**子动作**:

#### `style set-voice` - 设置风格声音

```bash
novel-cli style set-voice --aspect <aspect> --content <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--aspect` | string | 是 | - | 风格方面（如 `tone`、`pacing`） |
| `--content` | string | 是 | - | 风格描述 |

#### `style add-sensitive` - 添加敏感词

```bash
novel-cli style add-sensitive --word <word> [--category <cat>] [--severity <level>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--word` | string | 是 | - | 敏感词 |
| `--category` | string | 否 | `""` | 分类 |
| `--severity` | string | 否 | `""` | 严重程度 |

#### `style list-sensitive` - 列出敏感词

```bash
novel-cli style list-sensitive [--json]
```

#### `style render-rule` - 渲染风格规则文档

```bash
novel-cli style render-rule --output <path>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--output` | string | 是 | - | 输出文件路径 |

**示例**:

```bash
# 设置语调
novel-cli style set-voice --aspect tone --content "幽默诙谐，偶尔严肃"

# 添加敏感词
novel-cli style add-sensitive --word "禁词" --category "政治" --severity "high"

# 渲染风格规则文档
novel-cli style render-rule --output rules/style.md
```

---

### `finalize` - 创作周期封闭

管理创作周期的终检和封闭。

**语法**:
```bash
novel-cli finalize <action> [options]
```

**子动作**:

#### `finalize check` - 孤儿主线节点检查

```bash
novel-cli finalize check
```

检查是否存在未分配到卷或事件的不可变主线节点。

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 检查通过 |
| `1` | 存在孤儿节点 |

#### `finalize dispose` - 记录处置日志

```bash
novel-cli finalize dispose --content <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--content` | string | 是 | - | 处置内容 |

#### `finalize seal` - 封闭创作周期

```bash
novel-cli finalize seal
```

封闭整个创作周期，需先通过 `finalize check`。

**退出码**:

| 退出码 | 条件 |
|--------|------|
| `0` | 成功封闭 |
| `23` | 未通过终检或存在孤儿节点 |

**示例**:

```bash
# 执行终检
novel-cli finalize check

# 记录处置决定
novel-cli finalize dispose --content "所有主线节点已分配完毕"

# 封闭创作周期
novel-cli finalize seal
```

---

### `finalcheck` - 终检信号

记录终检信号（边界校验、孤儿节点检查等）。

**语法**:
```bash
novel-cli finalcheck signal --check-type <type> --result <result> [--detail <text>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--check-type` | string | 是 | - | 检查类型（如 `boundary_validated`、`mainplot_orphan_coverage`） |
| `--result` | string | 是 | - | 检查结果（如 `pass`、`fail`） |
| `--detail` | string | 否 | `null` | 详细信息 |

**输出**: `{"ok": true, "id": <id>, "check_type": "<type>", "result": "<result>"}`

**示例**:

```bash
# 记录边界校验通过
novel-cli finalcheck signal --check-type boundary_validated --result pass

# 记录检查失败
novel-cli finalcheck signal --check-type some_check --result fail --detail "原因说明"
```

---

## 元素管理

### `emergent` - 涌现元素管理

管理创作过程中涌现的元素（角色、情节、主题等）。

**语法**:
```bash
novel-cli emergent <action> [options]
```

**子动作**:

#### `emergent add` - 添加涌现元素

```bash
novel-cli emergent add --type <type> --name <name> --created-chapter-id <id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--type` | string | 是 | - | 元素类型 |
| `--name` | string | 是 | - | 元素名称 |
| `--created-chapter-id` | int | 是 | - | 创建时的章节 ID |

**输出**: 返回创建的涌现元素对象（状态默认为 `pending`）

#### `emergent list` - 列出涌现元素

```bash
novel-cli emergent list [--status <status>] [--scored] [--volume-id <id>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--status` | string | 否 | `null` | 过滤状态 |
| `--scored` | flag | 否 | false | 仅列出已评分的元素 |
| `--volume-id` | int | 否 | `null` | 过滤所属卷 |

#### `emergent score` - 评分涌现元素

```bash
novel-cli emergent score --id <id> --causal <score> --character <score> --theme <score> --surprise <score> --chapter-id <id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--id` | int | 是 | - | 涌现元素 ID |
| `--causal` | float | 是 | - | 因果性评分 |
| `--character` | float | 是 | - | 角色性评分 |
| `--theme` | float | 是 | - | 主题性评分 |
| `--surprise` | float | 是 | - | 意外性评分 |
| `--chapter-id` | int | 是 | - | 评分章节 ID |

**输出**: 返回综合评分（`causal*0.4 + character*0.3 + theme*0.2 + surprise*0.1`）

#### `emergent update-disposition` - 更新处置决策

```bash
novel-cli emergent update-disposition --id <id> --disposition <decision> --level <level> [--reason <text>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--id` | int | 是 | - | 涌现元素 ID |
| `--disposition` | string | 是 | - | 处置决策 |
| `--level` | string | 是 | - | 处置层级 |
| `--reason` | string | 否 | `null` | 处置理由 |

**示例**:

```bash
# 添加涌现元素
novel-cli emergent add --type character --name "神秘剑客" --created-chapter-id 3

# 列出待评分元素
novel-cli emergent list --status pending

# 评分
novel-cli emergent score --id 1 --causal 0.8 --character 0.9 --theme 0.6 --surprise 0.7 --chapter-id 5

# 更新处置
novel-cli emergent update-disposition --id 1 --disposition "升格为主线" --level event --reason "角色冲突强烈"
```

---

### `arc` - 弧线管理

管理涌现元素的发展弧线。

**语法**:
```bash
novel-cli arc <action> [options]
```

**子动作**:

#### `arc add` - 添加弧线节点

```bash
novel-cli arc add --emergent-id <id> --stage <stage> --summary <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--emergent-id` | int | 是 | - | 涌现元素 ID |
| `--stage` | string | 是 | - | 弧线阶段 |
| `--summary` | string | 是 | - | 阶段摘要 |

#### `arc list` - 列出弧线

```bash
novel-cli arc list --emergent-id <id>
```

#### `arc add-inferred` - 添加推测弧线

```bash
novel-cli arc add-inferred --emergent-id <id> --stage <stage> --summary <text> --chapter-id <id>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--emergent-id` | int | 是 | - | 涌现元素 ID |
| `--stage` | string | 是 | - | 弧线阶段 |
| `--summary` | string | 是 | - | 阶段摘要 |
| `--chapter-id` | int | 是 | - | 推测时的章节 ID |

#### `arc list-dormant-reactivating` - 列出待重激活弧线

```bash
novel-cli arc list-dormant-reactivating
```

#### `arc set-reactivation` - 设置重激活标记

```bash
novel-cli arc set-reactivation --id <id> --flag <0|1> [--last-active-chapter-id <id>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--id` | int | 是 | - | 弧线 ID |
| `--flag` | int | 是 | - | 重激活标记（1=需重激活，0=否） |
| `--last-active-chapter-id` | int | 否 | `null` | 最后活跃章节 ID |

**示例**:

```bash
# 添加弧线节点
novel-cli arc add --emergent-id 1 --stage "觉醒" --summary "角色意识到真相"

# 列出某元素的所有弧线
novel-cli arc list --emergent-id 1

# 添加推测弧线（用于重激活）
novel-cli arc add-inferred --emergent-id 1 --stage "复仇" --summary "角色回归复仇" --chapter-id 20

# 标记需要重激活
novel-cli arc set-reactivation --id 3 --flag 1 --last-active-chapter-id 15
```

---

### `context` - 上下文管理

管理章节上下文（线索绑定、即将到来的承诺等）。

**语法**:
```bash
novel-cli context <action> [options]
```

**子动作**:

#### `context chapter` - 获取章节上下文

```bash
novel-cli context chapter <chapter_id> [--limit <num>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `chapter_id` | int | 是 | - | 章节 ID |
| `--limit` | int | 否 | `15` | 返回线索数量上限 |

**输出**: 返回按动作优先级和相关性排序的线索列表

#### `context upcoming` - 获取即将到期的承诺

```bash
novel-cli context upcoming <chapter_id> [--limit <num>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `chapter_id` | int | 是 | - | 章节 ID |
| `--limit` | int | 否 | `10` | 返回承诺数量上限 |

**输出**: 返回 5 章内到期的承诺（按截止章节升序）

#### `context reactivations` - 获取重激活弧线

```bash
novel-cli context reactivations <chapter_id>
```

**输出**: 返回标记为重激活的推测弧线

#### `context bind` - 绑定线索到章节

```bash
novel-cli context bind --chapter-id <id> --element-type <type> --element-id <id> --relevance <score> --action <action>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--chapter-id` | int | 是 | - | 章节 ID |
| `--element-type` | string | 是 | - | 元素类型（如 `promise`、`arc`） |
| `--element-id` | int | 是 | - | 元素 ID |
| `--relevance` | float | 是 | - | 相关性评分 |
| `--action` | string | 是 | - | 动作（如 `payoff`、`advance`、`plant`、`reactivate`） |

**示例**:

```bash
# 获取章节上下文
novel-cli context chapter 10 --limit 20

# 获取即将到期的承诺
novel-cli context upcoming 10

# 获取重激活弧线
novel-cli context reactivations 10

# 绑定承诺到章节
novel-cli context bind --chapter-id 10 --element-type promise --element-id 3 --relevance 0.9 --action payoff
```

---

### `db` - 数据库元素操作

管理支线、伏笔、兑现、承诺等非线性元素。

**语法**:
```bash
novel-cli db <action> [options]
```

**子动作**:

#### `db add-subplot` - 添加支线

```bash
novel-cli db add-subplot --title <title> [--description <text>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--title` | string | 是 | - | 支线标题 |
| `--description` | string | 否 | `""` | 支线描述 |

**输出**: 返回创建的支线对象（状态默认为 `active`）

#### `db add-foreshadowing` - 添加伏笔

```bash
novel-cli db add-foreshadowing --content <text> [--planted-at <chapter_id>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--content` | string | 是 | - | 伏笔内容 |
| `--planted-at` | int | 否 | `null` | 埋设章节 ID |

#### `db add-payoff` - 添加兑现

```bash
novel-cli db add-payoff --foreshadowing-id <id> --content <text>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--foreshadowing-id` | int | 是 | - | 伏笔 ID |
| `--content` | string | 是 | - | 兑现内容 |

#### `db add-promise` - 添加承诺

```bash
novel-cli db add-promise --content <text> [--deadline-chapter <id>]
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--content` | string | 是 | - | 承诺内容 |
| `--deadline-chapter` | int | 否 | `null` | 截止章节 ID |

#### `db list` - 列出元素

```bash
novel-cli db list --type <type>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--type` | string | 是 | - | 元素类型：`subplot`/`foreshadowing`/`payoff`/`promise` |

#### `db update` - 更新元素状态

```bash
novel-cli db update --type <type> --id <id> --status <status>
```

**参数**:

| 参数 | 类型 | 必需 | 默认值 | 说明 |
|------|------|------|--------|------|
| `--type` | string | 是 | - | 元素类型：`subplot`/`foreshadowing`/`payoff`/`promise` |
| `--id` | int | 是 | - | 元素 ID |
| `--status` | string | 是 | - | 新状态 |

#### `db get` - 获取元素详情

```bash
novel-cli db get --type <type> --id <id>
```

**示例**:

```bash
# 添加支线
novel-cli db add-subplot --title "复仇线" --description "主角寻找仇人"

# 添加伏笔
novel-cli db add-foreshadowing --content "墙上的划痕" --planted-at 3

# 添加承诺
novel-cli db add-promise --content "必将揭开真相" --deadline-chapter 20

# 列出所有支线
novel-cli db list --type subplot

# 更新支线状态
novel-cli db update --type subplot --id 1 --status completed

# 获取支线详情
novel-cli db get --type subplot --id 1
```

---
