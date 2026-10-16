#!/usr/bin/env bash
# Fixture: 含 sealed 卷的世界（init + 卷1 active + 卷2 sealed）
#
# 用途：为 novel chapter create --volume 的"卷非 active 态拒绝加章"契约测试提供 fixture。
# 卷 1 保持 active，卷 2 置为 sealed 状态（volume_id=2），供 spec 的 WHEN 用 --volume 2
# 断言 volume_already_sealed（退出码 3）。
#
# 注意（重要）：
#   sealed 卷正常需走完整封卷流程（volume seal，前置含涌现处置、卷目标达成、卷内章节
#   状态等门禁），为单一契约测试构造该前置过于冗长，故本脚本跟随
#   setup-approved-chapter.sh 的测试 fixture 模式，用 SQLite 直接把卷 2 置为 sealed。
#   这是**测试 fixture 的非权威生成方式**，仅用于测试，不得作为 CLI 的替代入口。
#
# 用法：
#   ./setup-sealed-volume.sh <目标目录>
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

# 1. 初始化世界 + 2 个卷（卷 ID 自增为 1、2）
"$NOVEL" init "fixture-sealed-volume" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" volume create "第二卷" --order 2 --path "$DIR"

# 2. SQLite 直接把卷 2 置为 sealed（测试 fixture 模式，绕过封卷门禁）
sqlite3 "$DB" <<SQL
UPDATE volumes SET status='sealed', sealed_at='2026-07-01T00:00:00Z', updated_at='2026-07-01T00:00:00Z'
WHERE id=2;
SQL

echo "✓ sealed 卷 fixture 就绪：卷 1 active，卷 2 sealed（volume_id=2）"
