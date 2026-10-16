-- V002__settings_and_constraints.sql
-- s01-project-bootstrap：项目配置与约束定义表。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 CLI Contract 位于 contracts/cli/commands.md 与 json-schemas/。

-- 项目配置表。键值对存储，支持 UPSERT 语义。
CREATE TABLE IF NOT EXISTS settings (
    name       TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 创作约束规则表。复合主键 (constraint_type, scope, name)。
CREATE TABLE IF NOT EXISTS constraints (
    constraint_type TEXT NOT NULL,
    scope           TEXT NOT NULL,
    name            TEXT NOT NULL,
    content         TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK(severity IN ('soft', 'hard')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (constraint_type, scope, name)
);

-- constraints 表索引：支持按 scope 前缀查询。
CREATE INDEX IF NOT EXISTS idx_constraints_scope ON constraints(scope);

-- constraints 表索引：支持按 severity 过滤。
CREATE INDEX IF NOT EXISTS idx_constraints_severity ON constraints(severity);
