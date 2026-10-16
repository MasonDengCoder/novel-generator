#!/usr/bin/env bash
# Fixture: drafting 态、检查与裁决已 stale 的章节世界（自包含）
#
# 用途：为「裁决或检查 stale 时阻断进入 review」提供前置态。
# 流程：init + 卷 + chapter-001 → 写正文 v1 → 计算 hash v1 →
#   constraint check（绑定 hash v1）+ verdict achieved=1（绑定 hash v1）→
#   经 CLI 重写正文 v2（hash 变化，旧检查/裁决变 stale）→ status drafting。
#
# 自包含：在空目录上独立完成全部前置，不依赖其他 fixture。
#
# 用法：
#   ./setup-chapter-stale.sh <目标目录（须为空目录）>
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

"$NOVEL" init "fixture-chapter-stale" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

CONTENT_V1="$DIR/chapter-001-content-v1.md"
cat > "$CONTENT_V1" <<'EOF'
第一章正文 v1：检查与裁决将绑定此版本的 hash。
EOF
"$NOVEL" chapter update chapter-001 --content "$CONTENT_V1" --path "$DIR"

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

# 缺陷3 修复：verdict set-mainplot-landing 现在会校验主线节点存在，
# 故先建真实节点 mp-1-001（对齐 3 位补零 ID）再写裁决。
"$NOVEL" mainplot node create 1 --title "节点A" --path "$DIR"
"$NOVEL" verdict set-mainplot-landing chapter-001 --node-id mp-1-001 --achieved 1 --path "$DIR"

# 重写正文（hash 变化）→ 上述检查与裁决变 stale
CONTENT_V2="$DIR/chapter-001-content-v2.md"
cat > "$CONTENT_V2" <<'EOF'
第一章正文 v2：正文已重写，既有检查与裁决绑定旧 hash（stale）。
EOF
"$NOVEL" chapter update chapter-001 --content "$CONTENT_V2" --path "$DIR"

"$NOVEL" chapter status chapter-001 drafting --path "$DIR"

echo "✓ stale fixture 就绪（chapter-001 status=drafting，检查/裁决绑定旧 hash）"
