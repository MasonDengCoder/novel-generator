---
name: design-mainplot
description: 设计主线节点（大势定数）——逐节点入库并标注 is_immutable，委派 mainplot-minor-guardian 防过标。要规划主线、设定终局与不可绕过的里程碑时使用。
---

# design-mainplot

设计主线节点——3000 大道，故事的不可更改的定数。

## 概念

**immutable 节点**：标为 `is_immutable=1` 的主线节点是大势定数，整个创作过程中禁止越级修改。一旦冻结，只能靠正文去"实现"它，不能靠修改它来"逃避"。

## 步骤

1. **引导作者规划主线节点**：
   - 故事的终局是什么？（终点节点，必须 immutable）
   - 从起点到终局，有哪些不可绕过的转折/里程碑？
   - 哪些节点是"大势"（不可逆），哪些是"小势"（可调整）？

2. **逐节点入库**：
   ```
   novel-cli stage add-mainplot-node \
     --content "<节点描述>" \
     --is-immutable <1|0> \
     --order <排序序号>
   ```

3. **防过标检查** — 委派 `mainplot-minor-guardian` 检查是否有 minor 节点被误标为 immutable

4. **确认节点清单**，作者逐一确认 immutable 标注正确

5. **推进阶段**：
   ```
   novel-cli stage complete main-plot
   ```

6. 提示下一步：`/design-destiny` 设计旁门（支线/伏笔/爽点/承诺）。
