#!/usr/bin/env bash
# Fixture: 预置已知配置项的世界（自包含）
#
# 用途：为 setting 命令族的生成式契约测试（@codegen）提供可读回的世界。
# 预置 2 个已知键：world-style（含一次 UPSERT 覆盖，最终值"奇幻史诗"）与 pov。
#
# 自包含：在空目录上独立完成 init + 预置，不依赖其他 fixture 先运行
# （spec-gen 的 setup_fixture 对单脚本单 tempdir 运行一次）。
#
# 用法：
#   ./setup-settings.sh <目标目录（须为空目录）>
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

# 1. 初始化世界
"$NOVEL" init "fixture-settings" --path "$DIR" --json

# 2. 预置已知键（world-style 先写旧值再写新值，覆盖 UPSERT 语义）
"$NOVEL" setting set world-style "旧风格" --path "$DIR"
"$NOVEL" setting set world-style "奇幻史诗" --path "$DIR"
"$NOVEL" setting set pov "第一人称" --path "$DIR"

echo "✓ 配置项世界已生成于 $DIR/.novel/（world-style=奇幻史诗, pov=第一人称）"
