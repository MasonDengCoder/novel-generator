#!/usr/bin/env bash
# Fixture: 带 active 卷的最小世界（init + 1 个 active 卷，卷 ID=1）
#
# 用途：为 novel chapter create --volume 命令的契约测试提供最小世界。
# 第一个卷的 ID 为 1（autoincrement 自增），供 spec 的 WHEN 用 --volume 1 断言归属。
# 本 fixture 只建卷、不建章节——章节由被测命令 chapter create 自行创建。
#
# 用法：
#   ./setup-minimal-world-with-volume.sh <目标目录>
#
# 依赖：novel CLI 在 PATH 中，或通过 NOVEL_BIN 指定。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"
DB="$DIR/.novel/novel.db"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 1. 初始化世界
"$NOVEL" init "fixture-chapter-volume" --path "$DIR" --json

# 2. 创建一个 active 卷（ID 自增为 1）
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

echo "✓ 带 active 卷的最小世界已生成于 $DIR/.novel/（volume_id=1，状态 active，仅含卷、不含节点）"
