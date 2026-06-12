---
name: seal-creation
description: 封印创世——终检（主线节点覆盖性校验+世界圣经校验）、处置周期级剩余涌现、封档入只读。当前卷已封卷、所有章 done 后完成并封存整个创作周期时使用。
---

# seal-creation

封印创世——终检、处置孤儿、封档，完成创作周期。

## 前置条件

- 当前卷已封卷（`cycle close-volume` 完成）
- 所有章状态为 `done`（无 `must_failed`、无 `writing`）

## 步骤

### 1. 终检

```
novel-cli finalize check
```

检查内容：
- **主线节点覆盖性校验**：`is_immutable=1` 的节点是否全部被卷/事件承接；孤儿节点非空 → 阻断，须补绑或降级
- **世界圣经校验**：Bible 视图是否一致
- 结果写 `finalcheck_signal`（`check_type=mainplot_orphan_coverage` 等）

若阻断：
```
# 处置：补绑节点到某卷/事件，或降级 is_immutable=0（孤儿清单已包含在 finalize check 输出中）
novel-cli finalize check               # 复验
```

### 2. 处置剩余涌现（周期级）

```
novel-cli finalize dispose --content "<归档内容>"
```

处置所有未在卷级处置的 story_wide 级涌现。

### 3. 封档

```
novel-cli finalize seal
```

封档后 DB 进入只读模式，任何写操作将被 `core/db.py` 拦截。

输出终检报告与封档确认。
