---
name: design-style-guard
description: 定义文笔风格（视角/语体/节奏/情绪基调）与敏感词红线并入库，渲染写作期常驻 .claude/rules/style.md。设定文风或列出禁用词时使用。
---

# design-style-guard

定义文笔风格与敏感词红线，生成写作期常驻 rule。

## 步骤

1. **引导作者定义文风**（按维度逐一确认）：
   - 叙述视角（第一人称 / 第三人限知 / 全知等）
   - 语体（文雅 / 口语 / 混合）
   - 节奏（短句密集 / 长句舒缓 / 混合）
   - 情绪基调（克制 / 热烈 / 荒诞等）
   - 其他特殊要求

2. **入库文风定义**（每个维度一条）：
   ```
   novel-cli style set-voice \
     --aspect "<维度名>" \
     --content "<描述>"
   ```

3. **引导作者列出敏感词 / 禁用词红线**：
   - 政治敏感词、价值观红线
   - 特定人名/地名/品牌（需规避）
   - 其他风格禁忌词

4. **入库敏感词**（逐条）：
   ```
   novel-cli style add-sensitive \
     --word "<词>" \
     --category "<政治|人名|风格|其他>" \
     --severity "<error|warning>"
   ```

5. **渲染常驻 rule** — 从 DB 生成 `.claude/rules/style.md`：
   ```
   novel-cli style render-rule --output .claude/rules/style.md
   ```
   > 此文件写作期每轮常驻，LLM 写正文时自动遵守。

6. 确认生成内容，提示下一步：`/design-mainplot`。
