#!/usr/bin/env bash
# Fixture: review 态、门禁仍满足的章节世界（自包含）
#
# 用途：为「成功推进至 approved」「合法回退至 drafting/outline」提供前置态。
# 流程：与 setup-chapter-ready-for-review.sh 同（门禁全满足）后 status review。
# 终态：chapter-001 status=review，进入 approved 的整体验证仍满足。
#
# 自包含：在空目录上独立完成全部前置，不依赖其他 fixture。
#
# 用法：
#   ./setup-chapter-at-review.sh <目标目录（须为空目录）>
#
# 依赖：novel CLI、shasum。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi
if [ -e "$DIR/.novel" ]; then
  echo "❌ 目标已含 .novel/，请用空目录" >&2
  exit 4
fi

"$NOVEL" init "fixture-chapter-at-review" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

CONTENT="$DIR/chapter-001-content.md"
cat > "$CONTENT" <<'EOF'
第一章正文：本 fixture 写入的正文，供 review 态门禁与回退契约测试使用。
EOF
"$NOVEL" chapter update chapter-001 --content "$CONTENT" --path "$DIR"

HASH=$(shasum -a 256 "$DIR/.novel/content/chapters/chapter-001.md" | cut -d' ' -f1)
CHECKS_INPUT="$DIR/checks-input.json"
cat > "$CHECKS_INPUT" <<EOF
{
  "contentHash": "$HASH",
  "checks": [
    {"constraintType": "plot", "constraintScope": "world", "constraintName": "时间线一致性", "severity": "hard", "passed": true, "description": "时间线一致"}
  ]
}
EOF
"$NOVEL" constraint check --chapter chapter-001 --input "$CHECKS_INPUT" --path "$DIR"

# 缺陷3 修复：verdict 现在校验节点存在，先建真实节点 mp-1-001（对齐 3 位补零 ID）。
"$NOVEL" mainplot node create 1 --title "节点A" --path "$DIR"
"$NOVEL" verdict set-mainplot-landing chapter-001 --node-id mp-1-001 --achieved 1 --path "$DIR"
"$NOVEL" chapter status chapter-001 drafting --path "$DIR"
"$NOVEL" chapter status chapter-001 review --path "$DIR"

echo "✓ at-review fixture 就绪（chapter-001 status=review，门禁仍满足）"
