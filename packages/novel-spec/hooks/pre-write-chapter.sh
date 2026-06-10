#!/bin/sh
# pre-write-chapter.sh
# PreToolUse(Write, manuscripts/.../c<N>.md)
# 注入本章 threads、未来到期预警、重激活推测态快照到 additionalContext
# 任一环节失败 → exit 0 静默降级，绝不阻断写作

set -e

NOVEL_CLI="${NOVEL_CLI:-scripts/novel-cli}"
MAX_CHARS=8500

# 从 hook 输入 JSON 提取写入路径（stdin）
INPUT=$(cat)
FILE_PATH=$(printf '%s' "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null) || exit 0

# 仅对 manuscripts/.../c<N>.md 生效
echo "$FILE_PATH" | grep -qE 'manuscripts/.*c[0-9]+\.md$' || exit 0

# 解析章序号
CHAPTER_SEQ=$(basename "$FILE_PATH" .md | sed 's/^c//')
echo "$CHAPTER_SEQ" | grep -qE '^[0-9]+$' || exit 0

# 解析章 ID（_resolve-chapter 输出 {"chapter_id": N}）
RESOLVE=$($NOVEL_CLI context _resolve-chapter "$CHAPTER_SEQ" 2>/dev/null) || exit 0
CHAPTER_ID=$(printf '%s' "$RESOLVE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('chapter_id',''))" 2>/dev/null) || exit 0
[ -z "$CHAPTER_ID" ] && exit 0

CONTEXT=""
CHARS=0

# 拼接内容，字符计数（仅中文字符）
append() {
    local block="$1"
    local ccount
    ccount=$(printf '%s' "$block" | python3 -c "import sys,re; t=sys.stdin.read(); print(len(re.findall(r'[一-鿿]',t)))" 2>/dev/null) || ccount=0
    if [ $((CHARS + ccount)) -le $MAX_CHARS ]; then
        CONTEXT="${CONTEXT}${block}"
        CHARS=$((CHARS + ccount))
        return 0
    fi
    return 1  # 触顶
}

# 本章精准 threads（按 action 优先级 + relevance 降序，limit 15）
THREADS=$($NOVEL_CLI context chapter "$CHAPTER_ID" --limit 15 2>/dev/null) || true
if [ -n "$THREADS" ]; then
    append "$(printf '\n## 本章线索\n%s\n' "$THREADS")" || {
        CONTEXT="${CONTEXT}
[context 已按相关度截断，完整数据：novel-cli context chapter ${CHAPTER_ID}]"
    }
fi

# 未来 5 章到期预警（按 deadline_chapter 升序，limit 10）
if [ $CHARS -lt $MAX_CHARS ]; then
    UPCOMING=$($NOVEL_CLI context upcoming "$CHAPTER_ID" --limit 10 2>/dev/null) || true
    if [ -n "$UPCOMING" ]; then
        append "$(printf '\n## 即将到期\n%s\n' "$UPCOMING")" || {
            CONTEXT="${CONTEXT}
[context 已按相关度截断，完整数据：novel-cli context upcoming ${CHAPTER_ID}]"
        }
    fi
fi

# 重激活推测态快照（action=reactivate，无需 limit）
if [ $CHARS -lt $MAX_CHARS ]; then
    REACTS=$($NOVEL_CLI context reactivations "$CHAPTER_ID" 2>/dev/null) || true
    if [ -n "$REACTS" ]; then
        append "$(printf '\n## 重激活角色推测态\n%s\n' "$REACTS")" || {
            CONTEXT="${CONTEXT}
[context 已按相关度截断，完整数据：novel-cli context reactivations ${CHAPTER_ID}]"
        }
    fi
fi

[ -z "$CONTEXT" ] && exit 0

# 输出 additionalContext
python3 -c "
import sys, json
ctx = sys.argv[1]
print(json.dumps({'additionalContext': ctx}))
" "$CONTEXT" 2>/dev/null || exit 0
