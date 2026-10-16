#!/usr/bin/env bash
# Fixture: 正文文件被删除的章节世界（自包含）
#
# 用途：为「检测到正文文件缺失」「重新导入时正文文件缺失」提供前置态。
# 流程：init + 卷 + chapter-001 → 写真实正文 → 删除
#   .novel/content/chapters/chapter-001.md（正文文件缺失，元数据仍在）。
#
# 这是测试 fixture 的非权威生成方式（仅用于测试），不是 CLI 替代入口。
#
# 自包含：在空目录上独立完成全部前置，不依赖其他 fixture。
#
# 用法：
#   ./setup-chapter-missing-content.sh <目标目录（须为空目录）>
#
# 依赖：novel CLI。

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

"$NOVEL" init "fixture-chapter-missing-content" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

CONTENT="$DIR/chapter-001-content.md"
cat > "$CONTENT" <<'EOF'
第一章正文：写入后被删除，供正文文件缺失场景断言。
EOF
"$NOVEL" chapter update chapter-001 --content "$CONTENT" --path "$DIR"

rm -f "$DIR/.novel/content/chapters/chapter-001.md"

echo "✓ missing-content fixture 就绪（chapter-001 元数据在，正文文件已删除）"
