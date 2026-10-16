#!/usr/bin/env bash
# Fixture: 含未 approved 章节的世界（init + 卷 1 + drafting/review 章节）
#
# 用途：为封卷门禁「存在未 approved 章节时阻断」契约测试提供 fixture。
# 卷 1 含 chapter-001（drafting）与 chapter-002（review），供 `novel volume seal 1`
# 断言 chapters_not_approved（退出码 3）。
#
# 注意（重要）：
#   drafting/review 章正常需走状态机（chapter create 默认 drafting，review 需
#   chapter status 推进）。本脚本跟随 setup-approved-chapter.sh 的测试 fixture 模式，
#   用 SQLite 直接插入未 approved 章（含真实 sha256 正文 hash）。这是**测试 fixture 的
#   非权威生成方式**，仅用于测试，不得作为 CLI 的替代入口。
#
# 用法：
#   ./setup-volume-unapproved-chapters.sh <目标目录>
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
"$NOVEL" init "fixture-volume-unapproved" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

mkdir -p "$DIR/.novel/content/chapters"

insert_chapter() {
  local id="$1" title="$2" status="$3"
  local content="${id}正文内容（fixture）"
  local file="$DIR/.novel/content/chapters/${id}.md"
  printf '%s' "$content" > "$file"
  local hash wordcount
  hash=$(shasum -a 256 "$file" | cut -d' ' -f1)
  wordcount=$(printf '%s' "$content" | wc -m | tr -d ' ')
  sqlite3 "$DB" <<SQL
INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
VALUES ('${id}', '${title}', 'content/chapters/${id}.md', '${hash}', ${wordcount}, '${status}', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z');
SQL
}

# 2. SQL 直插 drafting + review 章
insert_chapter "chapter-001" "第一章" "drafting"
insert_chapter "chapter-002" "第二章" "review"

echo "✓ 未 approved 章节 fixture 就绪：卷 1 含 chapter-001（drafting）+ chapter-002（review）"
