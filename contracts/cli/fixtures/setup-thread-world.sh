#!/usr/bin/env bash
# Fixture: 线索与绑定世界（自包含）
#
# 用途：为 thread 命令族的生成式契约测试（@codegen）提供可读回的世界。
# 供 list/query 的 round-trip、bind/update 的退出码断言与
# thread_name_conflict / binding_already_exists / thread_not_found 场景。
#
# 已知集合（供 @codegen 的 json_array_len 断言）：
#   thread list 1 = 2 条线索（th-1-001 复仇线、th-1-002 备用线）
#   thread query --chapter chapter-001 = 1 条绑定（th-1-001 → chapter-001，默认 type mention）
#   thread query --chapter chapter-002 = 0 条绑定（无绑定章节）
#   th-1-002 未绑定任何章节，供 bind 的默认类型 / 指定类型 / 非法类型场景。
#
# 自包含：在空目录上独立完成 init + 卷 + 两章 + 线索与绑定预置，不依赖其他 fixture。
# 全部状态经 novel 命令推进（无 SQL 直插）。
#
# 用法：
#   ./setup-thread-world.sh <目标目录（须为空目录）>
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

# 1. 初始化世界 + 卷 1 + 两章（chapter-002 供「无绑定章节」查询）
"$NOVEL" init "fixture-thread-world" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --path "$DIR"
"$NOVEL" chapter create "第二章" --path "$DIR"

# 2. 预置线索：th-1-001 已绑定（默认 type mention）；th-1-002 未绑定
"$NOVEL" thread create 1 --name "复仇线" --description "主角复仇主线" --json --path "$DIR"
"$NOVEL" thread create 1 --name "备用线" --description "未绑定的备选线索" --json --path "$DIR"
"$NOVEL" thread bind th-1-001 --chapter chapter-001 --path "$DIR"

echo "✓ 线索世界已生成于 $DIR/.novel/（卷 1 + 两章 + th-1-001 已绑定 chapter-001 / th-1-002 未绑定）"
