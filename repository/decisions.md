# 决策理由累积

> 本文件沉淀已归档 OpenSpec change 的「设计决策理由」——为什么这么定。
> 与 `repository/dev-progress.md`（进度/状态）分离，本文件只记决策理由，不追进度时间线。
> **契约取值不在此复制**：退出码/错误码/schemaVersion/view-scope 的实际取值以 `contracts/cli/` 为 SSOT，本文件只给文件指针。
> 提取规则见 `.claude/skills/openspec-extract-facts/references/decisions-template.md`。

---

## 退出码契约治理（align-exit-codes-ssot）

以下五条决策来自 `align-exit-codes-ssot`，把散落在三处文档、互相冲突的退出码语义收口为单一事实来源。

### 退出码 SSOT 选址为 exit-codes.md，不新建文件

- **源 change**：`openspec/changes/archive/2026-06-25-align-exit-codes-ssot`（cli-contract，2026-06-25）
- **决定**：把 `contracts/cli/exit-codes.md` 升格为退出码语义唯一权威，不新建 `exit-code-policy.md` 之类文件
- **理由**：该文件已是契约文件、已与 `errors/mod.rs` 一致、已有 golden 与契约测试背书，选它为零迁移成本且符合 CLAUDE.md §9"稳定契约固化在 contracts/cli"；新建文件只会多一个可能漂移的落点
- **契约指针**：详见 `contracts/cli/exit-codes.md`（SSOT，生效码/预留码取值以该文件为准）

### exit-codes.md 采用 §A 生效 / §B 预留双层结构，非一次锁定全部码

- **源 change**：`openspec/changes/archive/2026-06-25-align-exit-codes-ssot`（cli-contract，2026-06-25）
- **决定**：`exit-codes.md` 分两节——§A 标注"有实现+测试背书，Spec 可依赖"的生效契约码；§B 登记"对应切片实现并通过测试前不生效，Spec 不得依赖"的预留码（语义意图 + 归属切片方向）
- **理由**：调和"规则要完整覆盖将来会用到的码"与 `config.yaml` apply 规则"不锁未验证语义"——预留码在文件里登记意图以保规则完整，但区分"已固化"与"待固化"避免把推断语义当稳定契约
- **契约指针**：详见 `contracts/cli/exit-codes.md` §A / §B（生效码与预留码的具体取值、升格路径以该文件为准）

### 文档改引用不重抄退出码表，错误对象示例对齐实现层

- **源 change**：`openspec/changes/archive/2026-06-25-align-exit-codes-ssot`（cli-contract，2026-06-25）
- **决定**：`docs/CLI架构设计.md` §11.3 与 `docs/CLI模块对齐清单.md` §12.1 删除自抄退出码表，替换为引用 SSOT + 一行人读索引；§11.2 错误对象示例字段对齐 `errors/mod.rs` 的 `ErrorObject`（移除 `blocking`，`suggestions`→`suggestion`）
- **理由**：SSOT 纪律是"唯一处定义，其余只引用"——只要文档还自抄表就会再次漂移（本次三轨冲突就是自抄后果）；示例字段与实现层不符会误导 Spec 消费方以为有 `blocking` 字段
- **契约指针**：详见 `contracts/cli/exit-codes.md`（退出码语义）与 `contracts/cli/error-codes.md`（错误对象字段定义）

### CLI模块对齐清单.md §9.4 锁码 reserved 化

- **源 change**：`openspec/changes/archive/2026-06-25-align-exit-codes-ssot`（cli-contract，2026-06-25）
- **决定**：§9.4"并发锁获取失败映射到退出码 8"改为"映射到预留码 8（见 `contracts/cli/exit-codes.md` §B），对应切片实现前不生效"；§12.2 的 `details` 字段对齐清单状态保持未完成 `[ ]` 不动
- **理由**：8 是预留码，清单直接写"映射到 8"会让人误以为已生效；改为 reserved 引用与双层结构一致。`details` 是实现层未完成项，对齐清单 `[ ]` 如实反映进度，文档治理不碰实现进度标记
- **契约指针**：详见 `contracts/cli/exit-codes.md` §B（预留码 8 的语义意图与升格条件以该文件为准）

### SSOT 原则固化进 spec，不只是改文档

- **源 change**：`openspec/changes/archive/2026-06-25-align-exit-codes-ssot`（cli-contract，2026-06-25）
- **决定**：在 `openspec/specs/cli-contract/spec.md` 把"退出码类别契约"requirement 细化——明确 SSOT 落点为 `contracts/cli/exit-codes.md`、生效码语义、预留码不依赖规则、文档不得重复定义退出码表，配 Given/When/Then 场景
- **理由**：只改文档治标、spec 治本——spec 是 OpenSpec 验证依据，把 SSOT 原则写进 spec 等于给"文档不得再自抄退出码表"装了回归闸门，`openspec validate` 或 verify 阶段可对照 spec 发现偏离
- **契约指针**：详见 `openspec/specs/cli-contract/spec.md`（SSOT requirement 与场景）+ `contracts/cli/exit-codes.md`（SSOT 落点）

## V2 错误协议默认化 + V1 弃用兼容治理（contract-registry-v2-05）

以下决策来自 `contract-registry-v2-05-contract-v2-rollout-governance`，把 V2 错误协议（`ErrorCatalog` 类型化目录 + `V2ErrorEnvelope`）与注册表驱动的全量契约生成设为默认消费与维护路径，并明确 V1 的兼容、弃用与移除准则。

### 缺省错误协议切到 V2，V1 转显式选择的弃用兼容路径

- **源 change**：`openspec/changes/contract-registry-v2-05-contract-v2-rollout-governance`（cli-contract，2026-09-16）
- **决定**：CLI 的默认错误协议为 V2——`ErrorProtocolVersion::default()` 翻转为 V2，`--json` 失败输出为 `V2ErrorEnvelope`（`errorId`/`category`/`exitCode`/`message`，可选 `details`），进程退出码与信封 `exitCode` 由 `ErrorCatalog` 类别派生。V1 `ErrorObject` 转为**弃用兼容路径**：仅在显式选择 V1 时输出，逐字保持 V1 回归基线。进入门槛：`contract-registry-v2-04-full-contract-generation` 归档（全量契约生成与漂移门禁可用）后才默认化。
- **理由**：V2 目录化（`ErrorCode::category()` 唯一类别绑定 + 固定类别退出码）让 Spec 消费方按稳定类别而非 per-entity 码分支，根因消除"退出码手抄→漂移"；兼容期保留显式 V1 避免破坏既有 V1 消费者（golden 回归基线逐字锁定）。详见 `docs/契约治理-生成式契约源迁移.md` §6.5。
- **契约指针**：详见 `contracts/cli/exit-codes.md`/`error-codes.md`（V2 默认类别映射，SSOT）与 `crates/novel-cli/src/errors/catalog.rs`（`ErrorProtocolVersion`/`ErrorCatalog`，类型化权威源）。`repository/dev-progress.md` §7.2 记录兼容期/弃用/移除进入条件的当前满足度。

### V1 移除三条件齐备准则（本切片不执行 V1 删除）

- **源 change**：`openspec/changes/contract-registry-v2-05-contract-v2-rollout-governance`（cli-contract，2026-09-16）
- **决定**：V1 物理移除必须由独立未来变更执行，且须满足（1）兼容期结束：注册表全部已迁移命令消费者显式选 V2 或已完成迁移回归，仓库无未选 V2 路径的 V1 契约测试；（2）消费者迁移完成：Spec 流程/示例/契约测试不再依赖 V1 per-entity 退出码或 V1 字段；（3）独立移除变更归档前置：02 task 2.3 类目码收敛、VerifyIntegrity 目录纯派生落地，契约测试/golden 只锁 V2，生成物无漂移。三者齐备才可移除 V1。
- **理由**：杀伐期过早删除会影响仍在按 V1 输出消费的稳定契约消费者；把移除条件固化为前置断言，避免治理收口与被"V1 还在"的反向拖累。
- **契约指针**：兼容范围、弃用通知与移除准则满足度当前状态见 `repository/dev-progress.md` §7.2；迁移说明见 `docs/契约治理-生成式契约源迁移.md` §6.5。
