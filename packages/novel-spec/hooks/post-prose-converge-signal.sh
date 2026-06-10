#!/bin/sh
# post-prose-converge-signal.sh
# PostToolUse(Write, manuscripts/.../c<N>.md)
# 软提示主 LLM 委派 mainplot-landing-verifier
# 不自算 converging，脚本故障 → exit 0

set -e

NOVEL_CLI="${NOVEL_CLI:-scripts/novel-cli}"

INPUT=$(cat)
FILE_PATH=$(printf '%s' "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null) || exit 0

echo "$FILE_PATH" | grep -qE 'manuscripts/.*c[0-9]+\.md$' || exit 0

CHAPTER_SEQ=$(basename "$FILE_PATH" .md | sed 's/^c//')
RESOLVE=$($NOVEL_CLI context _resolve-chapter "$CHAPTER_SEQ" 2>/dev/null) || exit 0
CHAPTER_ID=$(printf '%s' "$RESOLVE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('chapter_id',''))" 2>/dev/null) || exit 0
[ -z "$CHAPTER_ID" ] && exit 0

MSG="章节 c${CHAPTER_SEQ}（id=${CHAPTER_ID}）正文已写完。
请显式委派 \`mainplot-landing-verifier\` subagent 裁决本章 must 约束落地并写入 target_convergence_signal，
完成后才能执行 \`prose end ${CHAPTER_ID}\`。"

python3 -c "
import sys, json
print(json.dumps({'additionalContext': sys.argv[1]}))
" "$MSG" 2>/dev/null || exit 0
