#!/bin/bash
# Test script for progress-leak-guard.py
# 契约：拦截 = exit 2，放行 = exit 0

GUARD_SCRIPT=".claude/hooks/progress-leak-guard.py"
fail=0

echo "Testing progress-leak-guard..."

# Test 1: 应拦截 docs/ 中写入命令实现计数
echo '{"tool_name": "Write", "tool_input": {"file_path": "docs/test.md", "content": "CLI 已实现 46 条命令"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 1 failed: should DENY command count in docs/"
    fail=1
else
    echo "✅ Test 1 passed: blocked command count leak"
fi

# Test 2: 应拦截 docs/ 中规划态实体整段枚举（≥3 个实体名）
echo '{"tool_name": "Write", "tool_input": {"file_path": "docs/test.md", "content": "规划态实体：event、section、residue、phase-log、character 当前未实现"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 2 failed: should DENY entity enumeration in docs/"
    fail=1
else
    echo "✅ Test 2 passed: blocked entity enumeration leak"
fi

# Test 3: 应拦截状态声明快照句
echo '{"tool_name": "Edit", "tool_input": {"file_path": "docs/Spec架构设计.md", "old_string": "x", "new_string": "Spec 当前为规划态，0 行实现"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 3 failed: should DENY status snapshot sentence"
    fail=1
else
    echo "✅ Test 3 passed: blocked status snapshot leak"
fi

# Test 4: 应放行 dev-progress.md 本身（进度正当落点）
echo '{"tool_name": "Write", "tool_input": {"file_path": "repository/dev-progress.md", "content": "已实现 46 条命令，规划态实体：event section residue phase-log character"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 4 failed: should ALLOW progress content in dev-progress.md"
    fail=1
else
    echo "✅ Test 4 passed: allowed progress in dev-progress.md"
fi

# Test 5: 应放行含 dev-progress 指针的内容（白名单：已指针化）
echo '{"tool_name": "Write", "tool_input": {"file_path": "reference/参考.md", "content": "当前为规划态，详见 repository/dev-progress.md §7"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 5 failed: should ALLOW already-pointered content (whitelist)"
    fail=1
else
    echo "✅ Test 5 passed: allowed already-pointered content"
fi

# Test 6: 应放行 contracts/cli/（§A/§B 豁免）
echo '{"tool_name": "Write", "tool_input": {"file_path": "contracts/cli/exit-codes.md", "content": "✅ 0 成功 ⏳ 8 预留"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 6 failed: should ALLOW contracts/cli/ (§A/§B exempt)"
    fail=1
else
    echo "✅ Test 6 passed: allowed contracts/cli/ §A/§B"
fi

# Test 7: 应放行不含进度关键词的设计语义
echo '{"tool_name": "Write", "tool_input": {"file_path": "docs/state.md", "content": "章节状态机：outline → drafting → review → approved"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 7 failed: should ALLOW pure design semantics"
    fail=1
else
    echo "✅ Test 7 passed: allowed pure design semantics"
fi

# Test 8: 应放行非敏感路径（如 crates/ 源码）
echo '{"tool_name": "Write", "tool_input": {"file_path": "crates/novel-cli/src/main.rs", "content": "已实现 46 条命令"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 8 failed: should ALLOW non-sensitive paths (crates/)"
    fail=1
else
    echo "✅ Test 8 passed: allowed non-sensitive path"
fi

# Test 9: 应拦截逐码状态表（✅ ⏳ 并列）
echo '{"tool_name": "Write", "tool_input": {"file_path": "docs/codes.md", "content": "✅ 0 Success\n⏳ 8 LockFailed 预留"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 9 failed: should DENY per-code status table"
    fail=1
else
    echo "✅ Test 9 passed: blocked per-code status table"
fi

echo ""
if [ $fail -eq 0 ]; then
    echo "✅ All progress-leak-guard tests passed"
else
    echo "❌ Some tests failed"
fi
exit $fail
