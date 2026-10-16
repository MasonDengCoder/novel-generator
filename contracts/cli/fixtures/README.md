# CLI Fixture Worlds

本目录存放**可执行的 setup 脚本**，用于生成可复用的 fixture 世界，供 Spec 运行层与契约测试端到端断言消费。

## 为什么是脚本而非静态 .db

纯静态 `.novel/novel.db` 不可行：
- `schemaVersion` 与 migration 持续演进，静态 `.db` 会腐化。
- 章节正文文件必须真实存在，且 `content_hash` 必须与正文内容一致，否则 CLI 的 hash 检测会拒绝。
- fixture 脚本调用 `novel` CLI 真实命令序列构建世界，与测试内联 setup（如 `tests/integration/common.rs`）同构，只是从私有 helper 提升为共享资产。

## 脚本

| 脚本 | 生成内容 | 依赖 |
|---|---|---|
| `setup-minimal-world.sh <DIR>` | init + 1 卷 + 1 草稿章（outline） | novel |
| `setup-constraint-three-tier.sh <DIR>` | 在已初始化世界追加 info/warning/hard 三档约束 | 先 minimal，novel |
| `setup-approved-chapter.sh <DIR>` | 自建世界 + 卷 + 1 个 approved 章（含真实 sha256 正文）；独立，不要与 minimal 混用 | novel + sqlite3 + shasum |
| `setup-minimal-world-with-volume.sh <DIR>` | init + 1 个 active 卷（volume_id=1），不建章节；供 `chapter create --volume` 契约测试 | novel |
| `setup-sealed-volume.sh <DIR>` | init + 卷1 active + 卷2 sealed（SQL 直插置 sealed）；供 `chapter create --volume 2` 拒绝加章测试 | novel + sqlite3 |
| `setup-empty-dir.sh <DIR>` | 空目录（不初始化，no-op）；供 `world_not_initialized` 场景 | 无 |
| `setup-minimal-world-with-content.sh <DIR>` | init + 1 卷 + chapter-001 含真实 sha256 正文；供 `chapter context` 契约测试 | novel |
| `setup-chapter-with-modified-content.sh <DIR>` | 正文被外部修改的世界；供 hash 失配（`content_modified_externally`）场景 | novel |
| `setup-settings.sh <DIR>` | **自包含**：init + 预置 2 个已知配置项（world-style 含 UPSERT 覆盖 + pov）；供 setting 命令族生成式契约测试 | novel |
| `setup-constraints.sh <DIR>` | **自包含**：init + 卷 + 章 + 5 条多 scope（world/volume:1/chapter:chapter-001）× 多 severity 约束；供 constraint 命令族生成式契约测试 | novel |
| `setup-rule-checks.sh <DIR>` | **自包含**：init + 卷 + 两章 + chapter-001 写入 2 条检查结果（REPLACE 写两次）后改正文使其 stale；供 rule-checking 生成式契约测试 | novel + shasum |
| `setup-initialized-empty.sh <DIR>` | **自包含**：仅 init（无卷）；供 `volume list --json` 空列表断言 | novel |
| `setup-volume-unapproved-chapters.sh <DIR>` | **自包含**：init + 卷 1 + drafting/review 两章（SQL 直插，含真实 sha256）；供封卷「未 approved 章节阻断」 | novel + sqlite3 + shasum |
| `setup-volume-pending-emergence.sh <DIR>` | **自包含**：init + 卷 1 + pending 涌现（SQL 直插）；供封卷「未处置涌现阻断」 | novel + sqlite3 |
| `setup-volume-executed-emergence.sh <DIR>` | **自包含**：init + 卷 1 + executed 涌现（SQL 直插）；供封卷「涌现已处置通过」 | novel + sqlite3 |
| `setup-volume-unachieved-goal.sh <DIR>` | **自包含**：init + 卷 1 + 必达目标未达成 + 可选目标未达成（SQL 直插 mainplot_nodes achieved=0）；供封卷「未达成必达阻断」「只验必达」 | novel + sqlite3 |
| `setup-volume-achieved-goal.sh <DIR>` | **自包含**：init + 卷 1 + 必达目标已达成 + 可选目标未达成（SQL 直插 achieved=1/0）；供封卷「必达已达成通过」+ `goal get` 多目标断言 | novel + sqlite3 |
| `setup-volume-optional-goal.sh <DIR>` | **自包含**：init + 卷 1 + 可选目标未达成（SQL 直插 achieved=0）；供封卷「可选未达成不阻断」 | novel + sqlite3 |
| `setup-volume-seal-blocked.sh <DIR>` | **自包含**：init + 卷 1 + drafting 章 + pending 涌现 + 未达成必达目标（三类硬阻断并存）；供封卷「多检查失败报告全部失败项」 | novel + sqlite3 + shasum |
| `setup-volume-archived.sh <DIR>` | **自包含**：init + 卷 1 archived（SQL 直插）；供「已归档不可重复归档 / 归档卷不可设、删目标」 | novel + sqlite3 |
| `setup-chapter-drafting.sh <DIR>` | **自包含**：init + 卷 + chapter-001（outline，无正文）→ drafting；供「正文未写入时阻断进入 review」 | novel |
| `setup-chapter-ready-for-review.sh <DIR>` | **自包含**：init + 卷 + chapter-001 + 写正文 + `constraint check`（hard、passed=true、绑定当前 hash）+ `verdict set-mainplot-landing --achieved 1` + drafting；供「门禁全满足推进至 review」 | novel + shasum |
| `setup-chapter-at-review.sh <DIR>` | **自包含**：在 ready-for-review 的相同前置上推进至 review；供「推进至 approved」「合法回退至 drafting/outline」 | novel + shasum |
| `setup-chapter-hard-violation.sh <DIR>` | **自包含**：init + 卷 + chapter-001 + 写正文 + `constraint check`（hard、passed=false）+ 不写裁决 + drafting；供「hard 违规阻断」「多项门禁同时失败」 | novel + shasum |
| `setup-chapter-no-verdict.sh <DIR>` | **自包含**：init + 卷 + chapter-001 + 写正文 + `constraint check`（passed=true）+ 不写裁决 + drafting；供「裁决未达成时阻断进入 review」 | novel + shasum |
| `setup-chapter-stale.sh <DIR>` | **自包含**：init + 卷 + chapter-001 + 写正文 v1 + check/裁决绑定 hash v1 + 经 CLI 重写正文 v2（check/裁决变 stale）+ drafting；供「裁决或检查 stale 时阻断」 | novel + shasum |
| `setup-chapter-missing-content.sh <DIR>` | **自包含**：init + 卷 + chapter-001 + 写正文后删除 `.novel/content/chapters/chapter-001.md`（元数据在、正文文件缺失）；供「正文文件缺失」场景 | novel |
| `setup-chapter-review-modified.sh <DIR>` | **自包含**：门禁全满足推进至 review 后绕过 CLI 改写正文文件（外部修改）；供「正文在 review 期间被外部修改时阻断 approved」「回退时正文被外部修改仍阻断」 | novel + shasum |
| `setup-chapter-review-hard-violation.sh <DIR>` | **自包含**：门禁全满足推进至 review 后追加一条绑定当前 hash 的 hard 违规检查结果；供「回退不要求门禁」 | novel + shasum |
| `setup-chapter-review-verdict-unachieved.sh <DIR>` | **自包含**：门禁全满足推进至 review 后 `verdict set-mainplot-landing --achieved 0`；供「裁决在 review 期间被置为未达成时阻断 approved」 | novel + shasum |

> `contracts/cli/fixtures/assets/` 存放 `when` 命令引用的固定路径输入资产（如 `non-utf8.bin`，供「输入文件不是有效的 UTF-8」场景）；生成式契约测试以 `crates/novel-cli` 为 cwd，故 `when` 中经 `../../contracts/cli/fixtures/assets/<name>` 引用。

| `setup-initialized-world.sh <DIR>` | **自包含**：仅 init（无卷/章）；供 init 冲突与 `info`（`counts.*` 全零）生成式契约测试 | novel |
| `setup-mainplot-world.sh <DIR>` | **自包含**：init + 卷 1 + chapter-001 + 主线节点 mp-1-001（achieved=0）/ mp-1-002（achieved=1，经 CLI update），两节点均带非空 targetChapters；供 mainplot node 族生成式契约测试 | novel |
| `setup-thread-world.sh <DIR>` | **自包含**：init + 卷 1 + chapter-001/chapter-002 + 线索 th-1-001（已绑定 chapter-001，默认 type mention）/ th-1-002（未绑定）；供 thread 族生成式契约测试 | novel |
| `setup-emergence-world.sh <DIR>` | **自包含**：init + 卷 1 + chapter-001 + 涌现 em-1-1（pending 初始态）；供 emergence create/get 与 `when` 单命令驱动的生命周期推进 | novel |
| `setup-emergence-states.sh <DIR>` | **自包含**：init + 卷 1 + chapter-001 + 七条涌现经完整 CLI 生命周期预驱动到五态（pending×1 / rated×1 / decided×3 accept+defer+reject / executed×1 / archived×1）；供涌现状态机跳过阶段、后退与已归档修改断言 | novel |
| `setup-minimal-world-with-volume-and-node.sh <DIR>` | **自包含**：init + active 卷 1 + 主线节点 mp-1-001/mp-1-002；供卷目标设置场景引用真实主线节点 | novel |

> 「自包含」= 在空目录上独立完成 `init` + 前置状态，不依赖其他脚本先运行（spec-gen 的 `setup_fixture` 对单脚本单 tempdir 运行一次）；`setup-constraint-three-tier.sh` 则依赖「先 minimal」的链式用法，二者不要混用。

## 用法

```bash
export NOVEL_BIN=$(cargo run -q -p novel-cli --version >/dev/null && echo "cargo run -q -p novel-cli --") # 或 novel 在 PATH
# 简单方式：先 cargo build，用 target/debug/novel
cargo build -p novel-cli
export NOVEL_BIN=target/debug/novel

DIR=$(mktemp -d)
./contracts/cli/fixtures/setup-minimal-world.sh "$DIR"
./contracts/cli/fixtures/setup-constraint-three-tier.sh "$DIR"
./contracts/cli/fixtures/setup-approved-chapter.sh "$DIR"   # 在另一独立世界
novel info --json --path "$DIR"   # schemaVersion 应为当前版本
```

## approved 章的 SQL 直插说明

`setup-approved-chapter.sh` 用 `sqlite3` 直接插入 `status='approved'` 的章，因为正常状态机路径（review→approved）的前置条件（残余、四相日志、主线落地裁决）所属 CLI 域尚未实现，无法走 CLI 命令造出 approved 章。本脚本跟随 `tests/e2e/archive_export_e2e.rs` 的测试 fixture 模式（SQL 直插 + 真实 sha256）。这是**测试 fixture 的非权威生成方式，仅用于测试**，不得当作 CLI 的替代入口。

## 重新生成

fixture 世界是临时的（基于 `mktemp` 或测试 tempdir）。如需固定 fixture 快照，建议在 Spec 测试 setup 阶段调用本脚本而非提交静态 `.novel/`。`contracts/cli/fixtures/` 路径已被 `.claude/hooks/project-boundary-guard.py` 放行，允许包含 `.novel/`、正文、`.db` 等。
