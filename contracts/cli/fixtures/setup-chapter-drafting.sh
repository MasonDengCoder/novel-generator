#!/usr/bin/env bash
# Fixture: drafting 态、无正文的章节世界（自包含）
#
# 用途：为 state-transition-guards「正文未写入时阻断进入 review」提供前置态。
# 流程：init + 卷 + chapter-001（outline，未写正文）→ status drafting。
# 终态：chapter-001 status=drafting，content_hash 为空正文 hash（未写正文）。
#
# 自包含：在空目录上独立完成全部前置，不依赖其他 fixture。
#
# 用法：
#   ./setup-chapter-drafting.sh <目标目录（须为空目录）>
#
# 依赖：novel CLI（PATH 或 NOVEL_BIN）。

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

"$NOVEL" init "fixture-chapter-drafting" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"
"$NOVEL" chapter status chapter-001 drafting --path "$DIR"

echo "✓ drafting 无正文 fixture 就绪（chapter-001 status=drafting，未写正文）"
