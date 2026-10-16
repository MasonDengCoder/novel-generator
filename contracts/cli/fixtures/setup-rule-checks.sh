#!/usr/bin/env bash
# Fixture: 预置规则检查结果的世界（自包含，终态为 stale）
#
# 用途：为 rule-checking spec 的生成式契约测试（@codegen）提供可读回的世界。
# 流程：init + 卷 + 两章 → chapter-001 写真实正文 → 计算真实 sha256 →
#   构造含匹配 contentHash 的 --input JSON → 执行 constraint check 批量写入
#   （REPLACE 场景对同复合主键写两次，行数不变）→ 再改 chapter-001 正文
#   使已写检查结果变 stale（供 --verify-hash 读侧断言退出码 5）。
#
# 已知集合（供 @codegen 断言）：
#   check --list --chapter chapter-001 --json → checks 长度 2（REPLACE 后行数不变）
#   check --list --chapter chapter-002 --json → checks 为空（第二章无检查结果）
#   check --list --chapter chapter-001 --verify-hash --json → 退出码 5（stale）
#
# 自包含：在空目录上独立完成全部前置，不依赖其他 fixture。
#
# 用法：
#   ./setup-rule-checks.sh <目标目录（须为空目录）>
#
# 依赖：novel CLI（PATH 或 NOVEL_BIN）、shasum。

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

# 1. 初始化世界 + 卷 + 两章（chapter-002 无检查结果，供空 checks 断言）
"$NOVEL" init "fixture-rule-checks" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"
"$NOVEL" chapter create "第二章" --path "$DIR"

# 2. chapter-001 写真实正文（v1）
CONTENT_V1="$DIR/chapter-001-content-v1.md"
cat > "$CONTENT_V1" <<'EOF'
第一章正文内容，用于规则检查结果绑定 hash 的契约测试。
EOF
"$NOVEL" chapter update chapter-001 --content "$CONTENT_V1" --path "$DIR"

# 3. 计算真实 sha256（与 CLI 内部算法一致：对正文内容字节取 SHA-256）
HASH=$(shasum -a 256 "$DIR/.novel/content/chapters/chapter-001.md" | cut -d' ' -f1)

# 4. 构造含匹配 contentHash 的 --input JSON 并执行批量写入（第一次）
CHECKS_INPUT="$DIR/checks-input.json"
cat > "$CHECKS_INPUT" <<EOF
{
  "contentHash": "$HASH",
  "checks": [
    {"constraintType": "plot", "constraintScope": "world", "constraintName": "时间线一致性", "severity": "hard", "passed": true, "description": "时间线一致"},
    {"constraintType": "style", "constraintScope": "chapter:chapter-001", "constraintName": "战斗描写", "severity": "warning", "passed": false, "description": "战斗描写过长", "location": "第3段"}
  ]
}
EOF
"$NOVEL" constraint check --chapter chapter-001 --input "$CHECKS_INPUT" --path "$DIR"

# 5. REPLACE 场景：同复合主键写第二次（仅 description 变化），行数应保持 2
cat > "$CHECKS_INPUT" <<EOF
{
  "contentHash": "$HASH",
  "checks": [
    {"constraintType": "plot", "constraintScope": "world", "constraintName": "时间线一致性", "severity": "hard", "passed": true, "description": "时间线一致（复查）"},
    {"constraintType": "style", "constraintScope": "chapter:chapter-001", "constraintName": "战斗描写", "severity": "warning", "passed": false, "description": "战斗描写过长（复查）", "location": "第3段"}
  ]
}
EOF
"$NOVEL" constraint check --chapter chapter-001 --input "$CHECKS_INPUT" --path "$DIR"

# 6. 改 chapter-001 正文（v2），使已写检查结果变 stale
CONTENT_V2="$DIR/chapter-001-content-v2.md"
cat > "$CONTENT_V2" <<'EOF'
第一章正文内容已修改，使既有检查结果绑定 hash 失配（stale）。
EOF
"$NOVEL" chapter update chapter-001 --content "$CONTENT_V2" --path "$DIR"

echo "✓ 规则检查世界已生成于 $DIR/.novel/（chapter-001 有 2 条 stale 检查结果，chapter-002 无结果）"
