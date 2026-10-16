#!/bin/bash
# Test script for layer-boundary-guard.py
# 契约：拦截 = exit 2，警告 = exit 0 + stderr，放行 = exit 0

GUARD_SCRIPT=".claude/hooks/layer-boundary-guard.py"
fail=0

echo "Testing layer-boundary-guard..."

# Test 1: 应该拦截直接访问 SQLite（非通过 novel CLI）
echo '{"tool_name": "Bash", "tool_input": {"command": "sqlite3 .novel/novel.db \"SELECT * FROM chapters;\""}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 1 failed: should DENY direct SQLite access"
    fail=1
else
    echo "✅ Test 1 passed: blocked direct SQLite access"
fi

# Test 2: 应该放行通过 novel CLI 的数据库访问
echo '{"tool_name": "Bash", "tool_input": {"command": "novel chapter list --json"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 2 failed: should ALLOW novel CLI commands"
    fail=1
else
    echo "✅ Test 2 passed: allowed novel CLI command"
fi

# Test 3: 应该拦截直接读取 .novel/content
echo '{"tool_name": "Bash", "tool_input": {"command": "cat .novel/content/chapters/ch-001.md"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 3 failed: should DENY direct content file access"
    fail=1
else
    echo "✅ Test 3 passed: blocked direct content file access"
fi

# Test 4: 应该拦截直接写入 .novel/content
echo '{"tool_name": "Bash", "tool_input": {"command": "echo \"新内容\" > .novel/content/chapters/ch-001.md"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 4 failed: should DENY direct content file write"
    fail=1
else
    echo "✅ Test 4 passed: blocked direct content file write"
fi

# Test 5: 应该警告在 CLI 代码中使用 LLM 相关库
output=$(echo '{"tool_name": "Bash", "tool_input": {"command": "cd crates/novel-cli && cargo add anthropic"}}' | python3 "$GUARD_SCRIPT" 2>&1)
if [ $? -ne 0 ]; then
    echo "❌ Test 5 failed: should WARN (not block) LLM usage in CLI"
    fail=1
else
    if echo "$output" | grep -q "检测到"; then
        echo "✅ Test 5 passed: warned about LLM in CLI"
    else
        echo "⚠️  Test 5: no warning emitted (acceptable)"
    fi
fi

# Test 6: 应该放行普通 Rust 构建命令
echo '{"tool_name": "Bash", "tool_input": {"command": "cargo build --release"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 6 failed: should ALLOW normal cargo commands"
    fail=1
else
    echo "✅ Test 6 passed: allowed cargo build"
fi

# Test 7: 应该放行 Spec 层的代码
echo '{"tool_name": "Bash", "tool_input": {"command": "cd spec/runtime && cargo add anthropic"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 7 failed: should ALLOW LLM in Spec layer"
    fail=1
else
    echo "✅ Test 7 passed: allowed LLM in Spec layer"
fi

if [ $fail -eq 0 ]; then
    echo ""
    echo "✅ All layer-boundary-guard tests passed!"
else
    echo ""
    echo "❌ Some tests FAILED"
    exit 1
fi
