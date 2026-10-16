#!/usr/bin/env bash
# Fixture: 三档 severity 约束世界（info/warning/hard 各一）
#
# 用途：验证 CLI severity 三档枚举的端到端 fixture。依赖最小世界。
#
# 用法：
#   ./setup-constraint-three-tier.sh <目标目录>
#   先运行 setup-minimal-world.sh 生成世界，再运行本脚本追加约束。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录（须已 init）>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -f "$DIR/.novel/novel.db" ]; then
  echo "❌ 世界未初始化: $DIR/.novel/novel.db 不存在。请先运行 setup-minimal-world.sh" >&2
  exit 3
fi

# 三档约束各定义一条
"$NOVEL" constraint define --type plot    --scope world --name "info约束"    --content "提示性规则" --severity info    --path "$DIR"
"$NOVEL" constraint define --type plot    --scope world --name "warning约束" --content "建议性规则" --severity warning --path "$DIR"
"$NOVEL" constraint define --type plot    --scope world --name "hard约束"   --content "强制性规则" --severity hard   --path "$DIR"

echo "✓ 三档约束已定义"
echo "--- 约束列表 ---"
"$NOVEL" constraint list --json --path "$DIR" | python3 -c "
import sys, json
d = json.load(sys.stdin)
sevs = sorted(c['severity'] for c in d['constraints'])
print('severities=', sevs)
assert sevs == ['hard', 'info', 'warning'], '三档缺失'
print('✓ 三档枚举齐全')
"
