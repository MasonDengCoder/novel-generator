---
name: extract-residue
description: 提取章末残余——未解决的悬念、新生线索、待升级标记，逐条录入以驱动下一章。写完一章后登记遗留悬念与线索时使用。
---

# extract-residue

提取章末残余——未解决的悬念、新生线索、待升级标记，驱动下一章。

## 步骤

1. **写完正文后**（在 `prose end` 之前或之后），通读本章末尾，识别：
   - 未解决的悬念或开放性问题
   - 新出现的线索、角色、设定暗示
   - 需要在后续章节处理的情绪/因果余波

2. **逐条录入残余**：
   ```
   scripts/novel-cli prose residue add \
     --chapter-id <id> \
     --type <unresolved_mystery|new_clue|pending_escalation|other> \
     --content "<残余内容描述>"
   ```

3. **展示残余台账**（供下章写作参考）：
   ```
   scripts/novel-cli prose list-residue --chapter-id <id>
   ```

4. 提示：残余将在 `pre-write-chapter` hook 注入下一章 context，确保跨章续接。
