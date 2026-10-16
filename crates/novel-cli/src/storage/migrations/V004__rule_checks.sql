-- V004__rule_checks.sql
-- s03-rule-checking-engine：规则检查结果表，支持版本绑定与 stale 检测。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 CLI Contract 位于 contracts/cli/commands.md 与 json-schemas/。

-- 规则检查结果表。检查结果绑定到章节正文的 content_hash。
CREATE TABLE IF NOT EXISTS rule_checks (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id        TEXT NOT NULL,
    constraint_type   TEXT NOT NULL,
    constraint_scope  TEXT NOT NULL,
    constraint_name   TEXT NOT NULL,
    content_hash      TEXT NOT NULL,
    severity          TEXT NOT NULL CHECK(severity IN ('info', 'warning', 'hard')),
    passed            INTEGER NOT NULL CHECK(passed IN (0, 1)),
    description       TEXT,
    location          TEXT,
    checked_at        TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(chapter_id, constraint_type, constraint_scope, constraint_name, content_hash),
    FOREIGN KEY (chapter_id) REFERENCES chapters(id)
);

-- 索引：按 chapter_id 查询该章节的所有检查结果。
CREATE INDEX IF NOT EXISTS idx_rule_checks_chapter ON rule_checks(chapter_id);

-- 索引：按 content_hash 快速检测是否存在匹配当前正文版本的检查结果。
CREATE INDEX IF NOT EXISTS idx_rule_checks_hash ON rule_checks(content_hash);

-- 索引：按 severity 过滤，支持状态机门禁快速判断是否存在 hard 违规。
CREATE INDEX IF NOT EXISTS idx_rule_checks_severity ON rule_checks(severity);
