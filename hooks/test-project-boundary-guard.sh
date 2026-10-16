#!/bin/bash
# Test script for project-boundary-guard.py
# 契约：拦截 = exit 2，放行 = exit 0

GUARD_SCRIPT=".claude/hooks/project-boundary-guard.py"
fail=0

echo "Testing project-boundary-guard..."

# Test 1: 应该拦截在本仓库创建 .novel/ 文件
echo '{"tool_name": "Write", "tool_input": {"file_path": ".novel/novel.db"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 1 failed: should DENY writing .novel/ files in this repo"
    fail=1
else
    echo "✅ Test 1 passed: blocked .novel/ file creation"
fi

# Test 2: 应该拦截 mkdir .novel 命令
echo '{"tool_name": "Bash", "tool_input": {"command": "mkdir -p .novel/content/chapters"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 2 failed: should DENY mkdir .novel"
    fail=1
else
    echo "✅ Test 2 passed: blocked mkdir .novel"
fi

# Test 3: 应该拦截章节正文写入（非 docs/openspec）
echo '{"tool_name": "Write", "tool_input": {"file_path": "chapters/chapter-001.md"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 3 failed: should DENY writing chapter files"
    fail=1
else
    echo "✅ Test 3 passed: blocked chapter file creation"
fi

# Test 4: 应该拦截 SQLite 数据库操作
echo '{"tool_name": "Bash", "tool_input": {"command": "touch novel.db"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 2 ]; then
    echo "❌ Test 4 failed: should DENY SQLite db creation"
    fail=1
else
    echo "✅ Test 4 passed: blocked SQLite db creation"
fi

# Test 5: 应该放行 docs/ 下的 md 文件
echo '{"tool_name": "Write", "tool_input": {"file_path": "docs/新功能.md"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 5 failed: should ALLOW docs/ markdown files"
    fail=1
else
    echo "✅ Test 5 passed: allowed docs/ file"
fi

# Test 6: 应该放行 openspec/ 下的文件
echo '{"tool_name": "Write", "tool_input": {"file_path": "openspec/config.yaml"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 6 failed: should ALLOW openspec/ files"
    fail=1
else
    echo "✅ Test 6 passed: allowed openspec/ file"
fi

# Test 7: 应该放行 spec/fixtures/ 下的测试文件
echo '{"tool_name": "Write", "tool_input": {"file_path": "spec/fixtures/test-world/.novel/novel.db"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 7 failed: should ALLOW fixture files"
    fail=1
else
    echo "✅ Test 7 passed: allowed fixture file"
fi

# Test 8: 应该放行 crates/ 下的 Rust 代码
echo '{"tool_name": "Write", "tool_input": {"file_path": "crates/novel-cli/src/main.rs"}}' | python3 "$GUARD_SCRIPT" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "❌ Test 8 failed: should ALLOW Rust source files"
    fail=1
else
    echo "✅ Test 8 passed: allowed Rust file"
fi

if [ $fail -eq 0 ]; then
    echo ""
    echo "✅ All project-boundary-guard tests passed!"
else
    echo ""
    echo "❌ Some tests FAILED"
    exit 1
fi
