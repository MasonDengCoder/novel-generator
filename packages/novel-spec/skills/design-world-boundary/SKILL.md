---
name: design-world-boundary
description: 设计四极——世界的硬/模糊边界，作为整个创作的坐标系，逐条入库并委派 world-boundary-validator 校验。定义世界规则、能力上限、道德物理社会边界时使用。
---

# design-world-boundary

设计四极——世界的硬边界与模糊边界，作为整个创作的坐标系。

## 约束

- 至少 3 条**硬边界**（`is_fuzzy=0`）：不可突破，突破即崩
- 至少 1 条**模糊边界**（`is_fuzzy=1`）：可探索，但需有意识地跨越
- 每条边界须具体可判断（"任何事都可以发生"不是边界）

## 步骤

1. **引导作者定义四极**：
   - 这个世界里什么是绝对不可能的？（硬边界）
   - 什么是有代价的可能？（模糊边界）
   - 主人公的能力/认知上限在哪里？
   - 世界的道德/物理/社会规则是什么？

2. **逐条入库**（每条边界一次）：
   ```
   scripts/novel-cli stage add-boundary \
     --type "<分类：能力|道德|物理|社会|其他>" \
     --content "<边界描述>" \
     --is-fuzzy <0|1>
   ```

3. **委派 `world-boundary-validator`** subagent 校验完整性和一致性

   - 校验通过 → validator 写 `finalcheck_signal(boundary_validated, pass)` → 继续步骤 4
   - 校验不通过 → 根据 validator 输出修正，重新委派，直到通过

4. **推进阶段**（门禁在 CLI 端，`finalcheck_signal` 缺失则阻断）：
   ```
   scripts/novel-cli stage complete world-boundary
   ```

5. 提示下一步：`/design-style-guard` 定义文风红线。
