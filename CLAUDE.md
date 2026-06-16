# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目现状

**已进入实现阶段，软硬两层均已落地骨架，仍在迭代。** 现有：
- `packages/novel-lifecycle-cli/`（硬层）— Python 状态机 + SQLite，全部 CLI 模块已实现，`tests/` 下 smoke + contract-guard + layer1~5 + 承重字段测试守护。
- `packages/novel-spec/`（软层）— 11 skill / 7 agent / 5 hook / rules / templates + `install-spec.js`。
- `openspec/`（方法论规格）、`docs/`（项目级设计基线）、`plan/`（进度与契约对照）。

**事实源分层**：CLI 命令面的权威是 **`packages/novel-lifecycle-cli/docs/CLI使用手册.md`**（硬层主副本）与 **`packages/novel-spec/docs/CLI使用手册.md`**（软层只读副本）。软层调用是否对齐硬层见 `plan/软硬契约对照清单.md`。设计意图（Why）仍以 `docs/设计理念.md` 为准。**字段未冻结期，实现可能领先文档——改动 CLI 命令面后须同步更新硬层 CLI 手册、软层只读副本与对照清单。**

文档体系（三层分治）：
- 根 `docs/` — 项目级共享文档，只放跨层需求、理念、方法论与概要架构：
  - `docs/README.md` — 三层文档结构和查找指引
  - `docs/项目启动书.md` — 立项依据与里程碑（M1 需求 / M2 设计 / M3 验收）
  - `docs/需求说明书.md` — **M1 冻结的需求基线**（FR/DR/NFR/EXT 编号可追溯）
  - `docs/架构设计.md` — 项目级概要架构（三层职责 / 技术选型 / 契约隔离）
  - `docs/设计理念.md` — 各项设计的 **Why**（为什么这么做）
  - `docs/创世观对应关系.md` — 方法论内核（§五）与项目级映射
- `packages/novel-lifecycle-cli/docs/` — 硬层文档：
  - `CLI使用手册.md` — **CLI 命令面权威主副本**（所有子命令的完整语法、参数、输出格式、退出码）
  - `硬层架构.md` — CLI 分发机制、Python 模块组织、硬层目录结构
  - `详细设计.md` — 生命周期、元素入口、schema 分片、门禁细节
- `packages/novel-spec/docs/` — 软层文档：
  - `CLI使用手册.md` — 从硬层主副本同步的只读契约副本
  - `软层架构.md` — skill/agent/hook/rules 设计与运行流
  - `方法论映射.md` — 方法论到 Claude Code 原语的映射
  - `开发指南.md` — 契约隔离规则与 CLI 调用最佳实践
- `plan/TODO.md` — 文档撰写进度。关键设计抉择（DD-1~9）+ 旧项目参考点落地索引见硬层 `详细设计.md` 与附录。

**文档卫生**：文档被升格、合并或取代后，**删除过期原件**，不保留并存。删除前用 `grep` 检查引用并改向新文档，避免死链接。

## 这是什么

`novel-generator` 是运行在 Claude Code 上的**托管式小说创作工具包**。用户只做大势规划（灵感→四极→主线→旁门），后续逐章撰写全程 LLM 托管。以《创作即创世》方法论为内核。

## 核心架构：软硬分层 + 单向依赖

```
openspec/                       方法论层（最慢变，冻结为规格）
packages/novel-spec/            软层：Claude Code LLM harness（skill/agent/hook/rules）
packages/novel-lifecycle-cli/   硬层：Python 零依赖状态机 + SQLite 唯一数据源
```

依赖方向**严格单向：软层 → 硬层**。硬层对软层零感知，可脱离 LLM 独立运行与测试。

贯穿全系统的核心判断：**LLM 记忆不可靠，凡“长程必须记住、忘了就崩”的东西全部下沉成确定性 DB 数据兜底。**

## 不可违反的铁律（编码时一律遵守）

1. **软↔硬唯一接口 = `novel-cli <子命令>`**。软层任何阶段不得绕过 CLI 直接读写 DB 或设定文件；软层开发不得访问硬层内部文档，仅可依据 `packages/novel-spec/docs/CLI使用手册.md` 与根项目级文档交互。（绕过 CLI 直接写 DB 的 Bash 命令由 `guard-db-write.sh` hook 在脱轨场景下 fail-closed 钢性拦截，正常流为空操作。）
2. **正文绝不入库**。DB 只存 `prose_ref`（弱路径引用）+ `word_count` 遥测。生命周期层不生产、不读取正文本体。
3. **不可越级修正**。四层轮回（章节→事件→卷→创作周期）权限递增：章节级只能标记，越级处置遁出/改主线一律禁止。处置遁去其一须走四级阶梯：章标记 → 事件轻修 → 卷系统处置 → 周期归档。
4. **创作元素唯一入口 = `elements/element_manager.py`**。所有元素（支线/伏笔/爽点/承诺/涌现/弧线/线索绑定）读写经此入口分发，生命周期层不直接 import 元素处理器、不直接拼 SQL。
5. **设定即视图**。世界圣经（Bible）无独立存储，是 DB 的派生渲染（`bible core|active|dormant|full`）。
6. **容错降级而非阻断**。hook 等异常时静默降级（`exit 0`），绝不阻断写作。

## Schema 演进约定

- 单一 SQLite 库 `index/novel.db`，唯一 `sqlite3` 访问点 `core/db.py`。
- 5 个分片增量叠加：layer1 奠基 / layer2 编织 / layer3 写作 / layer4 封档 / layer5 横切绑定（layer5 为 LLM 而生，不属四层轮回）。
- **运行期冻结后**只允许 `CREATE TABLE IF NOT EXISTS`，永不 ALTER/DROP，补字段靠新建表，由 `test_layer1~5` 守护。
- **当前初始阶段字段未冻结**：字段按需增删，谁用到谁加；某张表跑通、结构稳定后再纳入冻结。

## 软层原语分配判据

软层原语分配判据已迁移到 `packages/novel-spec/docs/软层架构.md`，核心规则为：

> 跨阶段不变式、忘了就崩 → **Rules**（须短）；必然发生的确定性机械动作 → **Hook**；某阶段才触发的多步流程 → **Skill**；读完即弃、会撑爆主上下文的监测/校验 → **Subagent**。

派生原则：能下沉到 Hook 的确定性逻辑，绝不写进 Rules 让 LLM 自觉。

## CLI 分发机制（编码要点）

硬层 CLI 分发机制详见 `packages/novel-lifecycle-cli/docs/硬层架构.md`。编码时只需记住：完整安装后目标项目内直接 `novel-cli <cmd>` 执行（无需 `python3` 前缀、无需项目内 `scripts/` 副本）；CLI 默认对 CWD 下的 `index/novel.db` 操作；所有元素类子命令（db/emergent/arc/bible/context）统一映射到 `element_manager.py` 唯一入口。

## 安装与运行（设计目标，待实现）

```bash
# 完整安装（软层 + 全局 novel-cli），宿主机须有 python3 >= 3.8
node packages/novel-spec/install-spec.js [目标目录]
# 仅装硬层：在无 LLM 环境复制一份自包含副本到 <target>/scripts/（FR-2.2）
node packages/novel-lifecycle-cli/install-cli.js [目标目录]
```

进程通信：Node `execFileSync` → `novel-cli`（同步阻塞，JSON stdout，无 RPC 框架）。

## 约束原则

简单优先：硬层零第三方依赖（仅 Python 标准库），无构建步骤、无 linter 配置。
