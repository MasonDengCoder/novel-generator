#!/usr/bin/env bash
# Fixture: 多 scope 多 severity 约束世界（自包含）
#
# 用途：为 constraint 命令族的生成式契约测试（@codegen）提供可读回的世界。
# 预置 5 条已知约束（world×3 + volume:1×1 + chapter:chapter-001×1，
# severity 覆盖 info/warning/hard 三档；含同名不同 scope 共存与空内容约束）。
#
# 已知集合（供 @codegen 的 json_array_len 断言）：
#   list 全量 = 5；--scope volume: 前缀过滤 = 1；--severity hard 过滤 = 2
#   render --view llm --scope world = 3（world 层）
#   render --view llm --scope volume --volume-id 1 = 4（world + volume:1 两层叠加）
#   render --view llm --scope chapter --chapter-id chapter-001 = 5（world + volume:1 + chapter:chapter-001 三层叠加）
#
# 自包含：在空目录上独立完成 init + 卷 + 章 + 约束预置，不依赖其他 fixture。
#
# 用法：
#   ./setup-constraints.sh <目标目录（须为空目录）>
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

# 1. 初始化世界 + 卷 + 章（chapter-001 挂卷 1，使 chapter 视图三层叠加完整）
"$NOVEL" init "fixture-constraints" --path "$DIR" --json
"$NOVEL" volume create "第一卷" --order 1 --path "$DIR"
"$NOVEL" chapter create "第一章" --volume 1 --path "$DIR"

# 2. 预置约束（5 条，severity 只用 info|warning|hard，对齐 ConstraintSeverity 枚举）
#    world 层 ×3（含任意格式内容与空内容各一）
"$NOVEL" constraint define --type plot --scope world --name "时间线一致性" --content "这是一段无结构的纯文本约束" --severity hard --path "$DIR"
"$NOVEL" constraint define --type plot --scope world --name "角色行为" --content "全局规则" --severity info --path "$DIR"
"$NOVEL" constraint define --type plot --scope world --name "空内容" --content "" --severity info --path "$DIR"
#    volume:1 层 ×1（与 world 层"角色行为"同名不同 scope，验证复合主键共存）
"$NOVEL" constraint define --type plot --scope volume:1 --name "角色行为" --content "卷级规则" --severity hard --path "$DIR"
#    chapter:chapter-001 层 ×1
"$NOVEL" constraint define --type style --scope chapter:chapter-001 --name "战斗描写" --content "战斗从简" --severity warning --path "$DIR"

echo "✓ 约束世界已生成于 $DIR/.novel/（5 条约束，3 scope 层 × 3 档 severity）"
