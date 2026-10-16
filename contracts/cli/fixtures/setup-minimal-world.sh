#!/usr/bin/env bash
# Fixture: 最小世界（init + 1 卷 + 1 草稿章节）
#
# 用途：为 Spec 端到端测试提供最小可消费世界。生成的世界 schemaVersion
# 与当前 CLI 一致（见 novel info --json）。
#
# 用法：
#   ./setup-minimal-world.sh <目标目录>
#   若目录已含 .novel/ 会失败（novel init 拒绝覆盖）。
#
# 依赖：novel CLI 在 PATH 中，或通过 cargo run。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 1. 初始化世界
"$NOVEL" init "fixture-minimal" --path "$DIR" --json

# 2. 创建一卷（volume create 不支持 --json，人读输出）
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 3. 创建一章（drafting 状态，默认 outline）
"$NOVEL" chapter create "第一章" --path "$DIR"

echo "✓ 最小世界已生成于 $DIR/.novel/"
"$NOVEL" info --json --path "$DIR" | python3 -c "import sys,json; d=json.load(sys.stdin); print('schemaVersion=', d['schemaVersion'])"
