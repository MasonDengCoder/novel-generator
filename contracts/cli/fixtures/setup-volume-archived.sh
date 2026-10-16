#!/usr/bin/env bash
# Fixture: 含已归档卷的世界（init + 卷 1 archived）
#
# 用途：为「已归档卷不可重复归档 / 归档卷不可设、删目标」契约测试提供 fixture。
# 卷 1 置为 archived（SQL 直插），供 `novel volume archive 1` 断言
# volume_already_archived（退出码 3）、`novel volume goal set/remove 1` 断言
# volume_already_archived / volume_already_sealed（退出码 3）。
#
# 注意（重要）：
#   archived 卷正常需走完整封卷→归档流程，为单一契约测试构造该前置过于冗长。本脚本跟随
#   setup-sealed-volume.sh 的测试 fixture 模式，用 SQLite 直接把卷 1 置为 archived。
#   这是**测试 fixture 的非权威生成方式**，仅用于测试，不得作为 CLI 的替代入口。
#
# 用法：
#   ./setup-volume-archived.sh <目标目录>
#
# 依赖：novel、sqlite3。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"
DB="$DIR/.novel/novel.db"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi
if [ -e "$DIR/.novel" ]; then
  echo "❌ 目标已含 .novel/，请用空目录" >&2
  exit 4
fi

# 1. 初始化世界 + 卷 1
"$NOVEL" init "fixture-volume-archived" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 2. SQL 直插把卷 1 置为 archived
sqlite3 "$DB" <<SQL
UPDATE volumes SET status='archived', archived_at='2026-07-01T00:00:00Z', updated_at='2026-07-01T00:00:00Z'
WHERE id=1;
SQL

echo "✓ archived 卷 fixture 就绪：卷 1 archived"
