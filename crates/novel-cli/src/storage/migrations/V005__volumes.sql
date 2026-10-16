-- V004__volumes.sql
-- 卷管理：volumes 表与 volume_goals 表
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的契约见 contracts/cli/commands.md 与 contracts/cli/json-output.md。

-- 卷表：小说创作的中间组织层次，介于项目与章节之间
CREATE TABLE IF NOT EXISTS volumes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    volume_order INTEGER NOT NULL,  -- 卷序号，用于排序
    status TEXT NOT NULL DEFAULT 'active',  -- active/sealing/sealed/archived
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    sealed_at TEXT,     -- 封卷时间
    archived_at TEXT    -- 归档时间
);

-- 卷目标表：卷与主线节点的绑定关系
CREATE TABLE IF NOT EXISTS volume_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    volume_id INTEGER NOT NULL,
    mainplot_node_id TEXT NOT NULL,  -- 引用主线节点 ID（s06 提供）
    description TEXT,                 -- 人读描述
    is_mandatory BOOLEAN NOT NULL DEFAULT 1,  -- 是否必达
    created_at TEXT NOT NULL,
    FOREIGN KEY (volume_id) REFERENCES volumes(id) ON DELETE CASCADE
);

-- 索引：优化常见查询场景
CREATE INDEX IF NOT EXISTS idx_volumes_status ON volumes(status);
CREATE INDEX IF NOT EXISTS idx_volumes_order ON volumes(volume_order);
CREATE INDEX IF NOT EXISTS idx_volume_goals_volume ON volume_goals(volume_id);
