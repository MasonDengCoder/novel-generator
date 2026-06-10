# weave-fate

编织命运结构——卷→事件→小节→章，逐层将主线必达分解落地。

## 滚动窗口规则

| 窗口 | 细化粒度 | 批次大小 |
|------|---------|---------|
| 当前卷 | 章节级（每次 1–5 章） | 简单 5 章/批，复杂多伏笔 3 章/批 |
| 下一卷 | 章节级 | 每次 1–5 章 |
| 第三卷及以后 | 事件级（标题+简要描述） | — |

卷完成后窗口前移：下一卷升为当前卷，第三卷升为下一卷，以此类推。

## 主线分解原则

每个 immutable 主线节点必须被承接：

```
卷绑定 mainplot_target_node_id
  └─ 承接事件 is_mainplot_critical=1, mainplot_node_id=<node>
       └─ 小节 mainplot_must_beat="<本节须落地的节拍>"
            └─ 章 presummary.constraints[type=must, mainplot_node_id=<node>]
```

## 步骤

### 新建卷

```
scripts/novel-cli weave volume \
  --title "<卷标题>" \
  [--mainplot-target-node-id <node_id>]
```

### 规划事件（卷内）

```
scripts/novel-cli weave event \
  --volume-id <id> --title "<事件标题>" \
  [--mainplot-node-id <node_id>] \
  [--is-mainplot-critical 1]
```

> 同步写审计：含 `mainplot_node_id` 的卷/事件自动写 `mainplot_decomposition_audit`

### 规划小节（事件内）

```
scripts/novel-cli weave section \
  --event-id <id> --title "<小节标题>" \
  [--mainplot-must-beat "<必达节拍>"]
```

### 规划章（小节内）

```
scripts/novel-cli weave chapter \
  --section-id <id> --title "<章标题>" --seq <全书顺序编号>
```

> 含 `mainplot_must_beat` 的小节下的章，自动生成带溯源的 `presummary.type=must` 约束

### 生成章预摘要（presummary）

```
scripts/novel-cli presummary generate <chapter_id>
```

编织结束后提示：通过 `/write-prose` 开始正文写作。
