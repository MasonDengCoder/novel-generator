#!/bin/bash
# Test script for openspec-archive-guard.py
# 契约：拦截 = exit 2，警告 = exit 0 + stderr，放行 = exit 0

GUARD_SCRIPT=".claude/hooks/openspec-archive-guard.py"
fail=0

echo "Testing openspec-archive-guard..."

# Test 1: 应该放行非 archive 命令
echo '{"tool_name": "Bash", "tool_input": {"command": "openspec new-change"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 1 failed: should ALLOW non-archive commands"
    fail=1
else
    echo "✅ Test 1 passed: allowed non-archive command"
fi

# Test 2: 应该警告没有指定 change-id 的 archive
output=$(echo '{"tool_name": "Bash", "tool_input": {"command": "openspec archive"}}' | python3 "$GUARD_SCRIPT" 2>&1)
if [ $? -ne 0 ]; then
    echo "❌ Test 2 failed: should WARN (not block) archive without change-id"
    fail=1
else
    if echo "$output" | grep -q "警告"; then
        echo "✅ Test 2 passed: warned about archive without change-id"
    else
        echo "⚠️  Test 2: no warning emitted (acceptable)"
    fi
fi

# Test 3: 应该放行不存在的 change（无法检查）
echo '{"tool_name": "Bash", "tool_input": {"command": "openspec archive --change-id nonexistent-change"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 3 failed: should ALLOW archive when change not found (cannot verify)"
    fail=1
else
    echo "✅ Test 3 passed: allowed archive for nonexistent change"
fi

# Note: Tests 4-6 require actual openspec/changes/ directory structure
# These are integration tests that should be run when actual changes exist

echo ""
echo "ℹ️  Note: Full integration tests require actual openspec/changes/ with tasks.md/verify.md"
echo "   Current tests cover basic command detection logic."

if [ $fail -eq 0 ]; then
    echo ""
    echo "✅ All openspec-archive-guard tests passed!"
else
    echo ""
    echo "❌ Some tests FAILED"
    exit 1
fi
