#!/usr/bin/env bash
# Fixture: 含 approved 章节的世界
#
# 用途：为需要 approved 前置态的端到端测试（如 volume seal/archive）提供 fixture。
#
# 注意（重要）：
#   approved 章节正常需走完整状态机（drafting→review→approved），而 review→approved
#   前置条件包含残余、四相日志、主线落地裁决等尚未实现的域，故正常路径当前无法
#   造出 approved 章。本脚本跟随 archive_export_e2e.rs 的测试 fixture 模式，用
#   SQLite 直接插入 approved 章（含真实 sha256 正文 hash）。这是**测试 fixture 的
#   非权威生成方式**，仅用于测试，不得作为 CLI 的替代入口。
#
# 用法：
#   ./setup-approved-chapter.sh <目标目录>
#   目标目录须不存在 .novel/（脚本会 init 新世界 + 卷 + approved 章）。
#   不要与 setup-minimal-world.sh 混用（二者都会建 chapter-001，主键冲突）。
#
# 依赖：novel、sqlite3、shasum。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录（须不存在 .novel/）>}"
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

# 1. 初始化世界 + 卷（不通过 chapter create 建章，避免主键冲突）
"$NOVEL" init "fixture-approved" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

CHAPTER_ID="chapter-001"
TITLE="第一章"
CONTENT="第一章正文内容（fixture）"
CONTENT_FILE="$DIR/.novel/content/chapters/${CHAPTER_ID}.md"

# 2. 写正文文件
mkdir -p "$DIR/.novel/content/chapters"
printf '%s' "$CONTENT" > "$CONTENT_FILE"

# 3. 计算真实 sha256（与 CLI 内部算法一致：对正文内容字节取 SHA-256）
HASH=$(shasum -a 256 "$CONTENT_FILE" | cut -d' ' -f1)
WORD_COUNT=$(printf '%s' "$CONTENT" | wc -m | tr -d ' ')

# 4. SQLite 直接插入 approved 章（测试 fixture 模式）
sqlite3 "$DB" <<SQL
INSERT INTO chapters (id, title, content_path, content_hash, word_count, status, volume_id, created_at, updated_at)
VALUES ('${CHAPTER_ID}', '${TITLE}', 'content/chapters/${CHAPTER_ID}.md', '${HASH}', ${WORD_COUNT}, 'approved', 1, '2026-07-01T00:00:00Z', '2026-07-01T00:00:00Z');
SQL

echo "✓ approved 章节已插入: ${CHAPTER_ID}"
echo "--- hash 校验（novel chapter get 应返回一致 hash）---"
"$NOVEL" chapter get "$CHAPTER_ID" --json --path "$DIR" | python3 -c "
import sys, json
d = json.load(sys.stdin)
print('status=', d.get('status'))
print('contentHash=', d.get('contentHash'))
assert d.get('status') == 'approved', '状态应为 approved'
print('✓ approved fixture 就绪')
"
