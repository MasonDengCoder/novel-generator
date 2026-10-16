#!/usr/bin/env bash
# Fixture: 含已达成必达目标的世界（init + 卷 1 + 必达目标已达成 + 可选目标未达成）
#
# 用途：为封卷门禁「所有必达目标已达成时通过」「同一卷可设置多个目标」契约测试提供
# fixture。卷 1 含必达目标 mp-1-001（achieved=1，已达成）与可选目标 mp-1-002（achieved=0，
# 未达成但不阻断），供 `novel volume seal 1` 断言通过（退出码 0），供 `novel volume goal
# get 1 --json` 断言 goals 数组长度 2。
#
# 注意（重要）：
#   主线节点和目标均通过 novel CLI 创建，保持与生产入口一致。
#
# 用法：
#   ./setup-volume-achieved-goal.sh <目标目录>
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
"$NOVEL" init "fixture-volume-achieved-goal" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 2. 通过 CLI 创建主线节点，再推进第一个节点为已达成。
"$NOVEL" mainplot node create 1 --title "主线节点A" --path "$DIR" >/dev/null
"$NOVEL" mainplot node create 1 --title "主线节点B" --path "$DIR" >/dev/null
"$NOVEL" mainplot node update mp-1-001 --achieved 1 --path "$DIR"

# 3. 设置必达目标 + 可选目标（节点已存在）
"$NOVEL" volume goal set 1 mp-1-001 --description "完成主线A" --path "$DIR"
"$NOVEL" volume goal set 1 mp-1-002 --optional --description "可选支线" --path "$DIR"

echo "✓ 已达成必达目标 fixture 就绪：卷 1 含必达 mp-1-001（已达成）+ 可选 mp-1-002（未达成）"
