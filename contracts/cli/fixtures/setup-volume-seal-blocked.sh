#!/usr/bin/env bash
# Fixture: 三类封卷硬阻断并存的世界（init + 卷 1 + 未 approved 章 + pending 涌现 + 未达成必达目标）
#
# 用途：为封卷门禁「多个检查失败时报告所有失败项」契约测试提供 fixture。
# 卷 1 同时含未 approved 章节、pending 涌现、未达成必达目标，供
# `novel volume seal 1 --dry-run --json` 断言 checkResults 数组长度 3（三个检查项全部呈现），
# 或 `novel volume seal 1 --json` 断言 chapters_not_approved（退出码 3）。
#
# 注意（重要）：
#   主线节点和卷目标通过 novel CLI 创建；其余复杂阻断态沿用本 fixture 的既有构造方式。
#
# 用法：
#   ./setup-volume-seal-blocked.sh <目标目录>
#
# 依赖：novel、sqlite3、shasum。

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
"$NOVEL" init "fixture-volume-seal-blocked" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 1.5 通过 CLI 创建未达成主线节点。
"$NOVEL" mainplot node create 1 --title "主线节点A" --path "$DIR" >/dev/null

# 2. 设置必达目标（节点已存在）
"$NOVEL" volume goal set 1 mp-1-001 --description "完成主线A" --path "$DIR"

# 3. 未 approved 章节（drafting，含真实 sha256 正文 hash）
mkdir -p "$DIR/.novel/content/chapters"
CONTENT="第一章正文内容（fixture）"
CONTENT_FILE="$DIR/.novel/content/chapters/chapter-001.md"
printf '%s' "$CONTENT" > "$CONTENT_FILE"
HASH=$(shasum -a 256 "$CONTENT_FILE" | cut -d' ' -f1)
WORD_COUNT=$(printf '%s' "$CONTENT" | wc -m | tr -d ' ')

# 4. 三类硬阻断并存：未 approved 章 + pending 涌现（+ 已存在的未达成必达目标 mp-1-001）
sqlite3 "$DB" <<SQL
INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
VALUES ('chapter-001', '第一章', 'content/chapters/chapter-001.md', '${HASH}', ${WORD_COUNT}, 'drafting', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z');

INSERT INTO emergences (id, volume_id, title, description, discovered_at, status, created_at, updated_at)
VALUES ('em-1-1', 1, '涌现A', '未处置的涌现（fixture）', '2026-07-01T00:00:00Z', 'pending', '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z');
SQL

echo "✓ 三类封卷硬阻断 fixture 就绪：卷 1 含 drafting 章 + pending 涌现 + 未达成必达目标"
