#!/usr/bin/env bash
# Fixture: 含未达成必达目标的世界（init + 卷 1 + 必达目标 + 可选目标，均未达成）
#
# 用途：为封卷门禁「存在未达成必达目标时阻断」「封卷检查只验证必达目标」契约测试
# 提供 fixture。卷 1 含必达目标 mp-1-001（achieved=0）与可选目标 mp-1-002（achieved=0），
# 供 `novel volume seal 1` 断言 chapters_not_approved（退出码 3）——只因必达目标未达成而
# 阻断，可选目标不影响封卷。
#
# 注意（重要）：
#   主线节点和目标均通过 novel CLI 创建，保持与生产入口一致。
#
# 用法：
#   ./setup-volume-unachieved-goal.sh <目标目录>
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
"$NOVEL" init "fixture-volume-unachieved-goal" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 2. 通过 CLI 创建主线节点（均保持未达成）。
"$NOVEL" mainplot node create 1 --title "主线节点A" --path "$DIR" >/dev/null
"$NOVEL" mainplot node create 1 --title "主线节点B" --path "$DIR" >/dev/null

# 3. 设置必达目标 + 可选目标（节点已存在）
"$NOVEL" volume goal set 1 mp-1-001 --description "完成主线A" --path "$DIR"
"$NOVEL" volume goal set 1 mp-1-002 --optional --description "可选支线" --path "$DIR"

echo "✓ 未达成必达目标 fixture 就绪：卷 1 含必达 mp-1-001（未达成）+ 可选 mp-1-002（未达成）"
