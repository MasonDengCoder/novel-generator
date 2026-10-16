#!/usr/bin/env bash
# Fixture: 含未达成可选目标的世界（init + 卷 1 + 可选目标 + 主线节点 achieved=0）
#
# 用途：为封卷门禁「可选目标未达成不阻断」契约测试提供 fixture。
# 卷 1 含可选目标（--optional，is_mandatory=false，mp-1-001 achieved=0），
# 无必达目标，供 `novel volume seal 1` 断言通过（退出码 0）。
#
# 注意（重要）：
#   主线节点和目标均通过 novel CLI 创建，保持与生产入口一致。
#
# 用法：
#   ./setup-volume-optional-goal.sh <目标目录>
#
# 依赖：novel、sqlite3。

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

# 1. 初始化世界 + 卷 1
"$NOVEL" init "fixture-volume-optional-goal" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 2. 通过 CLI 创建未达成主线节点。
"$NOVEL" mainplot node create 1 --title "主线节点A" --path "$DIR" >/dev/null

# 3. 设置可选目标（节点已存在）
"$NOVEL" volume goal set 1 mp-1-001 --optional --description "可选支线" --path "$DIR"

echo "✓ 未达成可选目标 fixture 就绪：卷 1 含可选目标 mp-1-001（achieved=0，无必达）"
