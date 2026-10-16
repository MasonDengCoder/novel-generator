# Novel Generator 项目知识库

本目录保存 `novel-generator` 项目的稳定知识，供 Claude Code 与团队成员复用。

## 目录结构

- `architecture.md` - 核心架构边界与分层模型
- `terminology.md` - 术语词典与核心概念
- `cli-modules.md` - CLI 内部模块职责与调用链
- `storage-model.md` - 存储模型与一致性保障
- `contract-boundary.md` - CLI Contract 稳定边界定义
- `dev-progress.md` - 开发进度还原（已落地切片时间线、命令组实现状态、进行中契约修复、接续检查清单）
- `harness-workflow.md` - Harness / OpenSpec 使用流程（如需长期保存）
- `decisions.md` - 已归档 OpenSpec change 的设计决策理由累积（为什么这么定，只给契约指针不复制取值）
- `cli-capabilities.md` - CLI 能力导航地图（已落地 capability 的命令骨架 + 契约/进度指针）

## 使用说明

- `CLAUDE.md` 记录 Claude 在本项目如何工作（执行边界、命令、规则、危险操作、完成前验证）
- `repository/` 记录稳定项目知识（术语、架构、模块边界、集成约束、长期决策）
- `docs/` 记录详细设计文档、用户指南与架构权衡说明
- `contracts/cli/` 记录 CLI Contract 机器可验证契约

稳定项目知识的权威来源：
1. 项目代码结构（`crates/`, `contracts/`, `spec/`）
2. Cargo 配置（`Cargo.toml`, `Cargo.lock`）
3. 契约文档（`contracts/cli/`）
4. 架构文档（`docs/项目整体架构.md`, `docs/CLI架构设计.md`, `docs/Spec架构设计.md`）
