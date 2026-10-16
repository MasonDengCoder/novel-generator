#!/usr/bin/env bash
# Fixture: 涌现世界（自包含，pending 初始态）
#
# 用途：为 emergence 命令族的生成式契约测试（@codegen）提供生命周期起点。
# 预置 1 个 pending 涌现 em-1-1，供 create/get 的 JSON schema 断言、
# decide 跳过评分阶段的 invalid_state_transition、以及 when 单一命令驱动的
# rate / decide / execute / archive 正路径推进。
#
# 已知集合（供 @codegen 的 json_array_len 断言）：
#   emergence list 1 = 1 个涌现（em-1-1，status=pending）
#
# 自包含：在空目录上独立完成 init + 卷 + 章 + 涌现预置，不依赖其他 fixture。
# 全部状态经 novel 命令推进（无 SQL 直插）。
#
# 用法：
#   ./setup-emergence-world.sh <目标目录（须为空目录）>
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

# 1. 初始化世界 + 卷 1 + chapter-001（供 create --chapter 与 execute --target 引用）
"$NOVEL" init "fixture-emergence-world" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

# 2. 预置 1 个 pending 涌现
"$NOVEL" emergence create 1 --title "意外的盟友" --description "昔日敌人提出协助" \
  --json --path "$DIR"

echo "✓ 涌现世界已生成于 $DIR/.novel/（卷 1 + chapter-001 + em-1-1 pending）"
