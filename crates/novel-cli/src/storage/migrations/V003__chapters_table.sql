-- V003__chapters_table.sql
-- s02-chapter-content-core：章节元数据表与内容完整性索引。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 CLI Contract 位于 contracts/cli/commands.md 与 json-schemas/chapter.schema.json。

-- 章节元数据表。正文存储在 .novel/content/chapters/<id>.md，SQLite 仅保存引用和 hash。
CREATE TABLE IF NOT EXISTS chapters (
    id           TEXT PRIMARY KEY NOT NULL,
    title        TEXT NOT NULL,
    content_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    word_count   INTEGER NOT NULL DEFAULT 0,
    status       TEXT NOT NULL DEFAULT 'outline',  -- 章节状态：outline/drafting/review/approved
    volume_id    INTEGER,                           -- 卷 ID（外键引用 volumes.id）
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (volume_id) REFERENCES volumes(id) ON DELETE SET NULL
);

-- 索引：按 created_at 排序，支持 list 命令的时序查询。
CREATE INDEX IF NOT EXISTS idx_chapters_created_at ON chapters(created_at);

-- 索引：按 word_count 排序，支持后续统计功能。
CREATE INDEX IF NOT EXISTS idx_chapters_word_count ON chapters(word_count);

-- 索引：按 volume_id 过滤，支持卷级章节查询与封卷检查。
CREATE INDEX IF NOT EXISTS idx_chapters_volume ON chapters(volume_id);

-- 索引：按 status 过滤，支持状态机查询与封卷检查。
CREATE INDEX IF NOT EXISTS idx_chapters_status ON chapters(status);
