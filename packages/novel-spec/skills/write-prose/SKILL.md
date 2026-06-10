---
name: write-prose
description: 写一章正文——prose start 到 prose end 的完整写作流程，含加载上下文、逐相登记、委派 mainplot-landing-verifier 裁决 must 落地、收章门禁。写某章正文、续写或重写被阻断的章时使用。
---

# write-prose

写一章正文——从 prose start 到 prose end 的完整写作流程。

## 铁律（写作期不可违反）

- **写验分离**：写作者不自评 must 落地，必须委派 `mainplot-landing-verifier`
- **不越级修正**：章节级只能标记，禁止 judge/close 遁出、禁止修正主线定数
- **正文不入库**：DB 只存 `prose_ref` 和 `word_count`

## 步骤

### 1. 开章

```
novel-cli prose start <chapter_id>
```

输出本章 presummary 三档契约（must / may_deviate / encourage_surprise）。

### 2. 加载上下文

```
novel-cli bible core
novel-cli bible active
```

> PreToolUse hook 写章文件时自动注入本章 threads + 到期预警 + 重激活快照（无需手动）

### 3. 若有 dormant 角色重出场（`chapter_threads` 含 `action=reactivate`）

**先委派** `reactivation-inferer` subagent 写 `is_inferred=1` 推测态快照，再开始写作。

### 4. 写正文

写 `manuscripts/v<卷编号>/c<seq>.md`（hook 自动注入 context）。

### 5. 逐相登记

```
novel-cli prose phase <chapter_id> birth
# ... 写作中，tiandao-monitor 在旁监测（可按需委派）...
novel-cli prose phase <chapter_id> dwell
novel-cli prose phase <chapter_id> decay
novel-cli prose phase <chapter_id> death
```

### 6. 委派 mainplot-landing-verifier（必须，不可跳过）

**显式委派** `mainplot-landing-verifier` subagent，裁决本章所有 `type=must` 约束落地并写 `target_convergence_signal`。

等待 subagent 完成，不自行判定。

### 7. 收章

```
novel-cli prose end <chapter_id>
```

CLI 读裁决位：
- `satisfied` 全 null → **阻断**（未委派 verifier）
- 任一 `satisfied=0` → **阻断**（章进入 `must_failed`，须重写后复验）
- 全 `satisfied=1` → 章状态 `done`，字数遥测落库

---

重写流程：修改正文后，重新委派 `mainplot-landing-verifier`，再执行 `prose end`。
