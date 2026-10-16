-- V006__verdicts.sql
-- s04-state-machine-and-verdicts：主线落地裁决表，支持 hash 绑定与 UPSERT 覆盖语义。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 CLI Contract 位于 contracts/cli/commands.md、verdicts.md 与 json-schemas/verdict.schema.json。

-- 裁决表。裁决绑定章节正文的 content_hash，作为状态推进门禁的输入之一。
CREATE TABLE IF NOT EXISTS verdicts (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id    TEXT NOT NULL,
    verdict_type  TEXT NOT NULL,            -- 裁决类型，本切片仅 'mainplot-landing'，预留扩展
    node_id       TEXT NOT NULL,            -- 主线节点 ID
    achieved      INTEGER NOT NULL CHECK(achieved IN (0, 1)),
    content_hash  TEXT NOT NULL,            -- 绑定写入时的正文 hash
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(chapter_id, verdict_type, node_id),  -- UPSERT 覆盖语义
    FOREIGN KEY (chapter_id) REFERENCES chapters(id)
);

-- 索引：按 chapter_id 查询该章节的所有裁决（门禁查询主入口）。
CREATE INDEX IF NOT EXISTS idx_verdicts_chapter ON verdicts(chapter_id);

-- 索引：按 content_hash 快速检测是否存在绑定当前正文版本的裁决。
CREATE INDEX IF NOT EXISTS idx_verdicts_hash ON verdicts(content_hash);
