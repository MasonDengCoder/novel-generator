-- V001__initial.sql
-- 初始 schema：项目级元数据表 + schema_migrations 记录表。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 schemaVersion 由 PRAGMA user_version 承载（单一数据源），由 migration runner 设置。

-- 已应用 migration 版本记录表。runner 据此跳过已应用版本。
CREATE TABLE IF NOT EXISTS schema_migrations (
    version    INTEGER PRIMARY KEY,
    filename   TEXT    NOT NULL,
    applied_at TEXT    NOT NULL
);

-- 项目级元数据表。本切片仅存一行（单世界单行）。
CREATE TABLE IF NOT EXISTS project_meta (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    project_name TEXT    NOT NULL,
    db_path      TEXT    NOT NULL,
    created_at   TEXT    NOT NULL,
    schema_version TEXT  NOT NULL
);
