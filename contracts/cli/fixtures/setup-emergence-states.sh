#!/usr/bin/env bash
# Fixture: 涌现五态组合世界（自包含）
#
# 用途：为 emergence 状态机的生成式契约测试（@codegen）提供各状态起点。
# 因 spec-gen 的 setup_fixture 对单脚本单 tempdir 运行一次、@codegen 的 when 为单一命令，
# 「跳过阶段 / 后退 / 已归档修改」等需要特定前置状态的场景必须由本 fixture 预驱动。
#
# 已知集合（供 @codegen 的 json_array_len / --status 过滤断言）：
#   em-1-1：pending（未评分）
#   em-1-2：rated（已评分未决策）
#   em-1-3：decided + accept（可 execute 的正路径）
#   em-1-4：decided + defer（execute 应 invalid_state_transition）
#   em-1-5：decided + reject（execute 应 invalid_state_transition）
#   em-1-6：executed（可 archive 的正路径）
#   em-1-7：archived（rate/decide/execute 应 emergence_archived）
#   list 全量 = 7；--status pending=1 / rated=1 / decided=3 / executed=1 / archived=1
#
# 自包含：在空目录上独立完成 init + 卷 + 章 + 七条涌现预驱动，不依赖其他 fixture。
# 全部状态经 novel 命令逐条走完完整 CLI 生命周期（无 SQL 直插）。
#
# 用法：
#   ./setup-emergence-states.sh <目标目录（须为空目录）>
#
# 依赖：novel CLI 在 PATH 中，或通过 NOVEL_BIN 指定。

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

# 1. 初始化世界 + 卷 1 + chapter-001（供 execute --target 引用）
"$NOVEL" init "fixture-emergence-states" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

# 2. em-1-1：pending（仅创建，不推进）
"$NOVEL" emergence create 1 --title "涌现1" --description "停留 pending 的涌现" --json --path "$DIR"

# 3. em-1-2：rated（create + rate）
"$NOVEL" emergence create 1 --title "涌现2" --description "停留 rated 的涌现" --json --path "$DIR"
"$NOVEL" emergence rate em-1-2 --rating 7 --reason "中等影响" --path "$DIR"

# 4. em-1-3：decided + accept（正路径起点，可 execute）
"$NOVEL" emergence create 1 --title "涌现3" --description "决策 accept 的涌现" --json --path "$DIR"
"$NOVEL" emergence rate em-1-3 --rating 8 --reason "高影响" --path "$DIR"
"$NOVEL" emergence decide em-1-3 --decision accept --reason "符合故事弧" --path "$DIR"

# 5. em-1-4：decided + defer
"$NOVEL" emergence create 1 --title "涌现4" --description "决策 defer 的涌现" --json --path "$DIR"
"$NOVEL" emergence rate em-1-4 --rating 6 --reason "可延后" --path "$DIR"
"$NOVEL" emergence decide em-1-4 --decision defer --reason "本卷不处理" --path "$DIR"

# 6. em-1-5：decided + reject
"$NOVEL" emergence create 1 --title "涌现5" --description "决策 reject 的涌现" --json --path "$DIR"
"$NOVEL" emergence rate em-1-5 --rating 3 --reason "与既有设定冲突" --path "$DIR"
"$NOVEL" emergence decide em-1-5 --decision reject --reason "违反世界观" --path "$DIR"

# 7. em-1-6：executed（正路径起点，可 archive）
"$NOVEL" emergence create 1 --title "涌现6" --description "已执行未归档的涌现" --json --path "$DIR"
"$NOVEL" emergence rate em-1-6 --rating 9 --reason "关键情节" --path "$DIR"
"$NOVEL" emergence decide em-1-6 --decision accept --reason "采纳" --path "$DIR"
"$NOVEL" emergence execute em-1-6 --target chapter-001 --note "已并入第一章" --path "$DIR"

# 8. em-1-7：archived（完整生命周期）
"$NOVEL" emergence create 1 --title "涌现7" --description "已归档涌现" --json --path "$DIR"
"$NOVEL" emergence rate em-1-7 --rating 9 --reason "关键情节" --path "$DIR"
"$NOVEL" emergence decide em-1-7 --decision accept --reason "采纳" --path "$DIR"
"$NOVEL" emergence execute em-1-7 --target chapter-001 --note "已并入第一章" --path "$DIR"
"$NOVEL" emergence archive em-1-7 --path "$DIR"

echo "✓ 涌现五态世界已生成于 $DIR/.novel/（7 条涌现：pending×1 / rated×1 / decided×3 / executed×1 / archived×1）"
