-- V007__destiny_weaving.sql
-- s06-destiny-weaving：主线节点、线索、线索绑定、涌现四张表，支持命运编织能力。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 CLI Contract 位于 contracts/cli/commands.md 与 json-schemas/*.schema.json。

-- 主线节点表：卷内关键情节锚点，用于追踪主线推进
CREATE TABLE IF NOT EXISTS mainplot_nodes (
    id                TEXT PRIMARY KEY,          -- 格式：mp-<volume_id>-<seq>
    volume_id         INTEGER NOT NULL,          -- 引用 volumes(id)，INTEGER 类型
    sequence_number   INTEGER NOT NULL,
    title             TEXT NOT NULL,
    description       TEXT,
    target_chapters   TEXT,                      -- JSON array of chapter IDs，可为空
    achieved          INTEGER NOT NULL DEFAULT 0 CHECK(achieved IN (0, 1)),
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (volume_id) REFERENCES volumes(id),
    UNIQUE(volume_id, sequence_number)
);

CREATE INDEX IF NOT EXISTS idx_mainplot_nodes_volume ON mainplot_nodes(volume_id);
CREATE INDEX IF NOT EXISTS idx_mainplot_nodes_sequence ON mainplot_nodes(volume_id, sequence_number);

-- 线索表：跨章节的情节线
CREATE TABLE IF NOT EXISTS threads (
    id             TEXT PRIMARY KEY,             -- 格式：th-<volume_id>-<seq>
    volume_id      INTEGER NOT NULL,             -- 引用 volumes(id)，INTEGER 类型
    name           TEXT NOT NULL,
    description    TEXT,
    status         TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'paused', 'resolved')),
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (volume_id) REFERENCES volumes(id),
    UNIQUE(volume_id, name)
);

CREATE INDEX IF NOT EXISTS idx_threads_volume ON threads(volume_id);
CREATE INDEX IF NOT EXISTS idx_threads_status ON threads(volume_id, status);

-- 线索绑定表：线索与章节的多对多关系
CREATE TABLE IF NOT EXISTS thread_bindings (
    thread_id      TEXT NOT NULL,
    chapter_id     TEXT NOT NULL,
    binding_type   TEXT NOT NULL DEFAULT 'mention' CHECK(binding_type IN ('mention', 'develop', 'resolve')),
    note           TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (thread_id, chapter_id),
    FOREIGN KEY (thread_id) REFERENCES threads(id),
    FOREIGN KEY (chapter_id) REFERENCES chapters(id)
);

CREATE INDEX IF NOT EXISTS idx_thread_bindings_chapter ON thread_bindings(chapter_id);
CREATE INDEX IF NOT EXISTS idx_thread_bindings_thread ON thread_bindings(thread_id);

-- 涌现表：卷内出现的非规划内容机会，需评分、决策、执行并归档
CREATE TABLE IF NOT EXISTS emergences (
    id                 TEXT PRIMARY KEY,         -- 格式：em-<volume_id>-<seq>
    volume_id          INTEGER NOT NULL,         -- 引用 volumes(id)，INTEGER 类型
    chapter_id         TEXT,                     -- 发现涌现的章节（可选）
    title              TEXT NOT NULL,
    description        TEXT NOT NULL,
    discovered_at      TEXT NOT NULL DEFAULT (datetime('now')),

    -- 状态机字段
    status             TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'rated', 'decided', 'executed', 'archived')),

    -- rated 阶段字段
    rating             INTEGER CHECK(rating >= 0 AND rating <= 10),
    rating_reason      TEXT,
    rated_at           TEXT,

    -- decided 阶段字段
    decision           TEXT CHECK(decision IN ('accept', 'defer', 'reject')),
    decision_reason    TEXT,
    decided_at         TEXT,

    -- executed 阶段字段
    execution_target   TEXT,                     -- 关联的章节或产出 ID
    execution_note     TEXT,
    executed_at        TEXT,

    -- archived 阶段字段
    archived_at        TEXT,

    created_at         TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at         TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (volume_id) REFERENCES volumes(id),
    FOREIGN KEY (chapter_id) REFERENCES chapters(id)
);

CREATE INDEX IF NOT EXISTS idx_emergences_volume ON emergences(volume_id);
CREATE INDEX IF NOT EXISTS idx_emergences_status ON emergences(volume_id, status);
CREATE INDEX IF NOT EXISTS idx_emergences_chapter ON emergences(chapter_id);
CREATE INDEX IF NOT EXISTS idx_emergences_discovered ON emergences(discovered_at);
