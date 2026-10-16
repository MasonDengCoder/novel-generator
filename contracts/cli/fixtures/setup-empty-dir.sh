#!/usr/bin/env bash
# Fixture: 空目录（不初始化世界）
#
# 用途：world_not_initialized 场景——目标目录无 .novel/，所有需世界的命令应返回
# 退出码 3 + world_not_initialized。此 fixture 不执行任何 novel 命令，
# 仅确保目标目录存在（tempdir 已保证），故为 no-op。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi
echo "✓ 空目录 fixture 就绪（$DIR 无 .novel/）"
