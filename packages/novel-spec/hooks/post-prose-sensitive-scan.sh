#!/bin/sh
# post-prose-sensitive-scan.sh
# PostToolUse(Write, manuscripts/.../c<N>.md)
# 取敏感词表 grep 章文件，命中 → 要求修正（正常生效，非故障）
# 脚本自身故障 → exit 0 静默放行

NOVEL_CLI="${NOVEL_CLI:-scripts/novel-cli}"

INPUT=$(cat)
FILE_PATH=$(printf '%s' "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null) || exit 0

echo "$FILE_PATH" | grep -qE 'manuscripts/.*c[0-9]+\.md$' || exit 0
[ -f "$FILE_PATH" ] || exit 0

# 取词表（故障则放行），list-sensitive 输出 JSON 数组
WORDS_JSON=$($NOVEL_CLI style list-sensitive 2>/dev/null) || exit 0
[ -z "$WORDS_JSON" ] && exit 0

# 从 JSON 提取 word 字段，构造 grep pattern（| 合并，正则转义）
PATTERN=$(printf '%s' "$WORDS_JSON" | python3 -c "
import sys, json, re
try:
    words = [w['word'] for w in json.load(sys.stdin) if w.get('word')]
except Exception:
    sys.exit(0)
if words:
    print('|'.join(re.escape(w) for w in words))
" 2>/dev/null) || exit 0
[ -z "$PATTERN" ] && exit 0

# grep 章文件
HITS=$(grep -onE "$PATTERN" "$FILE_PATH" 2>/dev/null) || true
[ -z "$HITS" ] && exit 0

# 命中：输出要求修正的 additionalContext（正常生效，exit 0 让 CC 看到消息）
python3 -c "
import sys, json
hits = sys.argv[1]
msg = '⚠️  敏感词命中，须修正后方可继续：\n' + hits + '\n请修改正文移除上述词汇，修正后重新检查。'
print(json.dumps({'additionalContext': msg}))
" "$HITS" 2>/dev/null || exit 0
