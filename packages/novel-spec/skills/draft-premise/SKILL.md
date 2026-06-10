---
name: draft-premise
description: 将奇点凝炼为一句话前提，完成灵感阶段（stage complete inspiration）。已有奇点、要把核心冲动收敛为可执行的一句话前提时使用。
---

# draft-premise

将奇点凝炼为一句话前提，完成灵感阶段。

## 步骤

1. **取已有奇点**：`scripts/novel-cli stage get-inspiration`

2. **起草一句话前提**，格式参考：
   > 一个 [主人公身份] 必须 [核心行动]，否则 [后果]；但 [核心矛盾/代价]。

3. **与作者确认或调整**

4. **入库并推进阶段**：
   ```
   scripts/novel-cli stage set-premise --content "<一句话前提>"
   scripts/novel-cli stage complete inspiration
   ```

5. 提示下一步：`/design-world-boundary` 开始四极设计。
