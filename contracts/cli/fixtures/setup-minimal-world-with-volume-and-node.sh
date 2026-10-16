#!/usr/bin/env bash
# Fixture: 带 active 卷 + 主线节点的世界
#
# 用途：为 `volume goal set/get/remove` 等需引用真实主线节点的契约测试提供最小世界。
# 在 setup-minimal-world-with-volume（init + 1 个 active 卷，卷 ID=1）基础上，
# 额外直插主线节点 node-001 / node-002（非权威生成方式，仅测试用；缺陷修复后
# goal set 会校验节点存在）。注意：本 fixture 含节点，不可用于「空卷列节点」断言
# （那种场景应使用 setup-minimal-world-with-volume）。
#
# 用法：
#   ./setup-minimal-world-with-volume-and-node.sh <目标目录>
#
# 依赖：novel CLI 在 PATH 中，或通过 NOVEL_BIN 指定；sqlite3 可用。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 1. 初始化世界
"$NOVEL" init "fixture-volume-node" --path "$DIR" --json

# 2. 创建一个 active 卷（ID 自增为 1）
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"

# 3. 通过 CLI 创建主线节点（ID 依次为 mp-1-001 / mp-1-002）。
"$NOVEL" mainplot node create 1 --title "主线节点A" --path "$DIR" >/dev/null
"$NOVEL" mainplot node create 1 --title "主线节点B" --path "$DIR" >/dev/null

echo "✓ 带 active 卷与主线节点的世界已生成于 $DIR/.novel/（volume_id=1，含 mp-1-001/mp-1-002）"
