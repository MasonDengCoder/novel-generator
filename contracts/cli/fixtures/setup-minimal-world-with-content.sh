#!/usr/bin/env bash
# Fixture: 带正文的最小世界（init + 1 卷 + 1 章 + 写真实 sha256 正文）
#
# 用途：为 novel chapter context 命令的契约测试提供最小可渲染世界。
# spec 的 WHEN 以 chapter-001 为被测章节，故本章即 chapter-001。
#
# 用法：
#   ./setup-minimal-world-with-content.sh <目标目录>
#
# 依赖：novel CLI 在 PATH 中，或通过 NOVEL_BIN 指定。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 1. 初始化世界
"$NOVEL" init "fixture-context" --path "$DIR" --json

# 2. 创建一卷
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 3. 创建本章 chapter-001 + 写真实 sha256 正文
CONTENT_FILE="$DIR/chapter-001-content.md"
cat > "$CONTENT_FILE" <<'EOF'
# 第一章

这是本章正文内容，用于 novel chapter context 命令的渲染契约测试。
EOF
"$NOVEL" chapter create "第一章" --path "$DIR"
"$NOVEL" chapter update chapter-001 --content "$CONTENT_FILE" --path "$DIR"

# 缺陷3 修复：verdict set-mainplot-landing 现在会校验主线节点存在，
# 故建真实节点 mp-1-001（对齐 3 位补零 ID），供 verdict 契约测试引用。
"$NOVEL" mainplot node create 1 --title "节点A" --path "$DIR"

echo "✓ 带正文最小世界已生成于 $DIR/.novel/（chapter-001 含正文，mp-1-001 已建）"
