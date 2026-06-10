# capture-inspiration

捕捉创作奇点——那个让这个故事必须被写出来的核心冲动。

## 步骤

1. **引导作者说出奇点**：用开放问题引导，不限定形式：
   - 什么画面、问题、情感或矛盾让你想写这个故事？
   - 如果这个故事只留下一句话，那句话是什么？

2. **提炼奇点**（1–3 句，不超过 100 字）：
   - 核心冲突或张力
   - 主人公面临的不可逃避的处境
   - 让读者必须继续读下去的钩子

3. **入库**：
   ```
   scripts/novel-cli stage set-inspiration --content "<奇点内容>"
   ```

4. **确认** — 展示入库内容，询问是否继续到 `draft-premise`。

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
