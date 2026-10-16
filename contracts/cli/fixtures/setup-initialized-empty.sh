#!/usr/bin/env bash
# Fixture: 已初始化但无卷的世界（init only）
#
# 用途：为「无卷时返回空列表」契约测试提供 fixture——`novel volume list --json`
# 返回 `{schemaVersion, volumes: []}`，供 json_array_len 断言空数组。
#
# 用法：
#   ./setup-initialized-empty.sh <目标目录>
#
# 依赖：novel CLI 在 PATH 中，或通过 NOVEL_BIN 指定。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 仅初始化世界，不创建任何卷。
"$NOVEL" init "fixture-initialized-empty" --path "$DIR" --json

echo "✓ 已初始化空世界（无卷）于 $DIR/.novel/"
