#!/bin/sh
# check-bible-budget.sh
# PostToolUse(update-bible 等圣经相关写操作)
# active 层超限 → 仅警告建议 /close-volume，不阻断
# 脚本故障 → exit 0

NOVEL_CLI="${NOVEL_CLI:-novel-cli}"
ACTIVE_LIMIT=3000

# 取 active 层字数（中文字符数），故障则放行
ACTIVE_CONTENT=$($NOVEL_CLI bible active 2>/dev/null) || exit 0
[ -z "$ACTIVE_CONTENT" ] && exit 0

CHAR_COUNT=$(printf '%s' "$ACTIVE_CONTENT" | python3 -c "
import sys, re
t = sys.stdin.read()
print(len(re.findall(r'[一-鿿]', t)))
" 2>/dev/null) || exit 0

[ "$CHAR_COUNT" -le "$ACTIVE_LIMIT" ] && exit 0

python3 -c "
import sys, json
count, limit = int(sys.argv[1]), int(sys.argv[2])
msg = f'⚠️  Bible active 层当前 {count} 字，超过上限 {limit} 字。建议执行卷级封卷（\`/close-volume\`）触发 active→dormant 轮换以释放预算。'
print(json.dumps({'additionalContext': msg}))
" "$CHAR_COUNT" "$ACTIVE_LIMIT" 2>/dev/null || exit 0
