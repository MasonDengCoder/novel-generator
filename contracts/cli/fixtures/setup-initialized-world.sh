#!/usr/bin/env bash
# Fixture: 纯 init 世界（无卷/无章/无业务数据）
#
# 用途：为 init 冲突与 info 读取的生成式契约测试提供「已初始化但无任何业务数据」
# 的最小世界——`novel info --json` 的 `counts.*` 全为 0（与 `info.golden.json` 语义一致），
# 再次 `novel init` 同一目录返回 resource_conflict（退出码 4）。
#
# 与相邻脚本区分：`setup-minimal-world.sh`（init + 卷 + 章）、
# `setup-minimal-world-with-volume.sh`（init + 卷）、`setup-initialized-empty.sh`
# （init only，世界名 fixture-initialized-empty，供 volume list 空列表断言）都带
# 业务数据或不同世界名；本脚本是世界名 `fixture-init` 的唯一「纯 init」fixture。
#
# 用法：
#   ./setup-initialized-world.sh <目标目录>
#   若目录已含 .novel/ 会失败（novel init 拒绝覆盖）。
#
# 依赖：novel CLI 在 PATH 中，或通过 NOVEL_BIN 指定。

set -euo pipefail

DIR="${1:?用法: $0 <目标目录>}"
NOVEL="${NOVEL_BIN:-novel}"

if [ ! -d "$DIR" ]; then
  echo "❌ 目录不存在: $DIR" >&2
  exit 2
fi

# 仅初始化世界，不创建任何卷、章或其他业务实体。
"$NOVEL" init "fixture-init" --path "$DIR" --json

echo "✓ 纯 init 世界已生成于 $DIR/.novel/（无卷/无章，counts 全零）"
