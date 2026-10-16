#!/usr/bin/env bash
# Fixture: review 态、正文已被外部修改的章节世界（自包含）
#
# 用途：为「正文在 review 期间被外部修改时阻断 approved」「回退时正文被外部修改仍阻断」
# 提供前置态。流程：门禁全满足推进至 review → 绕过 CLI 改写正文文件。
#
# 这是测试 fixture 的非权威生成方式（仅用于测试），不是 CLI 替代入口。
# 沿用 setup-chapter-with-modified-content.sh 的 hash 失配构造模式。
#
# 自包含：在空目录上独立完成全部前置，不依赖其他 fixture。
#
# 用法：
#   ./setup-chapter-review-modified.sh <目标目录（须为空目录）>
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

"$NOVEL" init "fixture-chapter-review-modified" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

CONTENT="$DIR/chapter-001-content.md"
cat > "$CONTENT" <<'EOF'
第一章正文：经 CLI 写入并在 review 后被外部改写。
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

# 绕过 CLI 直接改写正文文件（使 SQLite content_hash 失配）
cat > "$DIR/.novel/content/chapters/chapter-001.md" <<'EOF'
第一章正文：已被外部编辑器改写，hash 与 SQLite 不一致。
EOF

echo "✓ review-modified fixture 就绪（chapter-001 status=review，正文已外部修改）"
