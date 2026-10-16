#!/usr/bin/env bash
# 生成物漂移检查闸（generate-drift gate）
#
# 调用 spec-gen check：在临时目录重新生成，与仓库内 contracts/cli/generated/ 比对。
# - 一致 → exit 0
# - 漂移（有人手改生成物，或源 spec 改后未重新生成）→ exit 1
# - spec-gen 自身报错（SSOT 对齐闸 / fixture 引用闸）→ exit 1
#
# 用法：./scripts/check-generated-drift.sh   （仓库根为 cwd，cargo 可用）

set -euo pipefail
cargo run -q -p spec-gen -- check "$@"
