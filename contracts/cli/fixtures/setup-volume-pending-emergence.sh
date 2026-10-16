#!/usr/bin/env bash
# Fixture: 含未处置涌现的世界（init + 卷 1 + pending 涌现）
#
# 用途：为封卷门禁「存在未处置涌现时阻断」契约测试提供 fixture。
# 卷 1 含一条 status='pending' 的涌现，供 `novel volume seal 1` 断言
# chapters_not_approved（退出码 3，seal_volume 对所有前置失败统一折叠为该码）。
#
# 注意（重要）：
#   pending 涌现正常需走 emergence 状态机（rate→decide→execute→archived）。本脚本跟随
#   测试 fixture 模式，用 SQLite 直接插入 pending 涌现。这是**测试 fixture 的非权威
#   生成方式**，仅用于测试，不得作为 CLI 的替代入口。
#
# 用法：
#   ./setup-volume-pending-emergence.sh <目标目录>
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
"$NOVEL" init "fixture-volume-pending-emergence" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 2. SQL 直插一条 pending 涌现
sqlite3 "$DB" <<SQL
INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at)
VALUES ('em-1-1', 1, '涌现A', '未处置的涌现（fixture）', '2026-07-01T00:00:00Z', 'pending', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z');
SQL

echo "✓ pending 涌现 fixture 就绪：卷 1 含 em-1-1（pending）"
