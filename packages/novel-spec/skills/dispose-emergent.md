# dispose-emergent

处置涌现实体（遁去其一）——`cycle close-volume` 的**第一步，必须完成后才能封卷**。

## 四级处置阶梯（权限递增）

| 级别 | 命令 | 适用场景 |
|------|------|---------|
| 章标记 | `prose residue add --type pending_escalation` | 章级发现，暂记 |
| 事件轻修 | `emergent update-disposition --level event` | 局部调整，不影响主线 |
| 卷系统处置 | `emergent update-disposition --level volume` | 本技能执行层级 |
| 周期归档 | `finalize dispose-emergent` | 全书影响，封档时处理 |

## 步骤

### 1. 取待处置涌现列表（已评分）

```
scripts/novel-cli emergent list --status pending --scored
```

若存在未评分涌现，**先委派 `emergent-scorer`** 补分，再继续。

### 2. 逐条决策（每个涌现）

参考评分结果（≥70 positive / 50–69 neutral / <50 negative）：

```
scripts/novel-cli emergent update-disposition \
  --id <id> \
  --level volume \
  --disposition <integrate|suppress|transform|archive> \
  --reason "<处置理由>"
```

### 3. 修订受影响章节的 presummary（若处置改变后续约束）

```
scripts/novel-cli db update presummary \
  --chapter-id <id> \
  --constraint-idx <idx> \
  --content "<修订后的约束内容>"
```

### 4. 完成后确认清单

```
scripts/novel-cli emergent list --volume-id <volume_id> --status pending
```

所有 pending 涌现处置完毕后，提示继续执行 `cycle close-volume`。
