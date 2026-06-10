#!/bin/sh
# guard-db-write.sh
# PreToolUse(Bash)
# fail-closed：触碰 *.db / sqlite3 且不经 scripts/novel-cli → exit 2 阻断
# 脚本自身故障 → exit 0 静默放行（正常流永远是空操作）

INPUT=$(cat)
CMD=$(printf '%s' "$INPUT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
print(d.get('tool_input', {}).get('command', ''))
" 2>/dev/null) || exit 0

[ -z "$CMD" ] && exit 0

# 检测是否触碰数据库
echo "$CMD" | grep -qE '\.(db|sqlite3?)([[:space:]\"'"'"'>|;]|$)|sqlite3[[:space:]]' || exit 0

# 白名单：经由 scripts/novel-cli 或 novel-cli 执行
echo "$CMD" | grep -qE '(scripts/novel-cli|novel-cli)[[:space:]]' && exit 0

# 不在白名单且触碰 DB → 阻断
printf '{"decision":"block","reason":"直接写库违反铁律1，请使用 novel-cli 子命令替代直接操作 SQLite。"}\n'
exit 2
