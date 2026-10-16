#!/usr/bin/env bash
# Fixture: 正文已被外部修改的世界
#
# 用途：content_modified_externally 场景——正常建世界 + chapter-001 写正文后，
# 绕过 CLI 直接改写正文文件，使 SQLite 存储的 content_hash 与正文实际 hash 不一致。
# novel chapter context 应检测到并返回退出码 21 + content_modified_externally。
#
# 这是测试 fixture 的非权威生成方式（仅用于测试），不是 CLI 替代入口。
# 沿用 setup-approved-chapter.sh 与 tests/e2e/archive_export_e2e.rs 的 fixture 模式。
#
# 用法：
#   ./setup-chapter-with-modified-content.sh <目标目录>
#
# 依赖：novel CLI + 标准工具（cat/echo）。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 1. 正常建世界 + 写正文（与 minimal-with-content 同构）
"$NOVEL" init "fixture-mismatch" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
CONTENT_FILE="$DIR/chapter-001-content.md"
cat > "$CONTENT_FILE" <<'EOF'
# 第一章
原始正文。
EOF
"$NOVEL" chapter create "第一章" --path "$DIR"
"$NOVEL" chapter update chapter-001 --content "$CONTENT_FILE" --path "$DIR"

# 2. 绕过 CLI 直接改写正文文件（使 SQLite content_hash 失配）
# 正文文件实际路径由 CLI 写入 .novel/content/chapters/chapter-001.md
CONTENT_REAL="$DIR/.novel/content/chapters/chapter-001.md"
cat > "$CONTENT_REAL" <<'EOF'
# 第一章
正文已被外部编辑器篡改，hash 与 SQLite 不一致。
EOF

# 缺陷3 修复：verdict 外部修改场景也会先做节点存在校验（早于外部修改检查），
# 故须先建真实节点 mp-1-001（对齐 3 位补零 ID），否则会先于外部修改返回 node_not_found。
"$NOVEL" mainplot node create 1 --title "节点A" --path "$DIR"

echo "✓ 失配 fixture 就绪（chapter-001 正文已外部修改，mp-1-001 已建）"
