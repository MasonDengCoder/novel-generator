---
name: design-destiny
description: 设计旁门——支线、伏笔、爽点、承诺，构成主线外的命运网络，逐条入库。规划角色弧线、埋伏笔、设定爽点或向读者许下叙事契约时使用。
---

# design-destiny

设计旁门——800 条支线、伏笔、爽点、承诺，构成主线外的命运网络。

## 四类元素

| 类型 | 作用 | CLI |
|------|------|-----|
| 支线（subplot） | 角色/世界的独立弧线，可与主线交织 | `db add-subplot` |
| 伏笔（foreshadowing） | 埋下必须收束的线索，开放 ≤2 卷、延期 ≤1 次 | `db add-foreshadowing` |
| 爽点（payoff） | 伏笔的收束，指定收束章 | `db add-payoff` |
| 承诺（promise） | 向读者许下的叙事契约，需在 deadline 前兑现 | `db add-promise` |

## 步骤

1. **引导作者列举旁门**：
   - 主线之外，哪些角色/势力有自己的故事弧线？（支线）
   - 哪些细节/能力/谜题需要在后文解答？（伏笔）
   - 读者期待什么高潮/反转？（爽点）
   - 对读者许下了哪些"一定会给你的"？（承诺）

2. **逐条入库**（举例）：
   ```
   novel-cli db add-subplot --title "<标题>" --description "<简述>"
   novel-cli db add-foreshadowing --content "<伏笔内容>" --planted-at <chapter_seq>
   novel-cli db add-payoff --foreshadowing-id <id> --content "<收束方式>"
   novel-cli db add-promise --content "<承诺内容>" --deadline-chapter <seq>
   ```

3. **展示全貌** — 输出旁门台账概览：
   ```
   novel-cli db list --type subplot
   novel-cli db list --type foreshadowing
   novel-cli db list --type payoff
   novel-cli db list --type promise
   ```

4. 确认完整，推进阶段，提示下一步：
   ```
   novel-cli stage complete destiny
   ```
   提示：`/weave-fate` 开始编织卷/事件/章结构。
