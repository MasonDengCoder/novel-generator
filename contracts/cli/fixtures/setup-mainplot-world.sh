#!/usr/bin/env bash
# Fixture: 主线节点世界（自包含）
#
# 用途：为 mainplot node 命令族的生成式契约测试（@codegen）提供可读回的世界。
# 供 list/get 的 round-trip、update 的落库效果读回与 not-found 错误对象。
#
# 已知集合（供 @codegen 的 json_array_len / json_num_field / every_nodes_has 断言）：
#   node list 1 全量 = 2 个节点
#     mp-1-001：achieved=0，targetChapters=[chapter-001]
#     mp-1-002：achieved=1，targetChapters=[chapter-001]（fixture 内经 CLI 推进）
#   两节点均带非空 targetChapters，使 every_nodes_has targetChapters 可断言。
#
# 自包含：在空目录上独立完成 init + 卷 + 章 + 主线节点预置，不依赖其他 fixture。
# 全部状态经 novel 命令推进（无 SQL 直插）。
#
# 用法：
#   ./setup-mainplot-world.sh <目标目录（须为空目录）>
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

# 1. 初始化世界 + 卷 1 + chapter-001（供 targetChapters 引用）
"$NOVEL" init "fixture-mainplot-world" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"

# 2. 预置主线节点（mp-1-001 初始 achieved=0；mp-1-002 经 CLI update 推到 achieved=1）
"$NOVEL" mainplot node create 1 --title "节点A" --description "第一卷开篇主线节点" \
  --target-chapters chapter-001 --json --path "$DIR"
"$NOVEL" mainplot node create 1 --title "节点B" --description "第二主线节点" \
  --target-chapters chapter-001 --json --path "$DIR"
"$NOVEL" mainplot node update mp-1-002 --achieved 1 --path "$DIR"

echo "✓ 主线节点世界已生成于 $DIR/.novel/（卷 1 + chapter-001 + mp-1-001 achieved=0 / mp-1-002 achieved=1）"
