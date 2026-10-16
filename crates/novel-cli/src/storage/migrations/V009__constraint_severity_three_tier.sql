-- V009: constraint severity 三档化
--
-- 背景（#5 契约修复）：
-- - V002 的 constraints 表 severity CHECK 限制为 ('soft', 'hard')。
-- - 契约（constraint-check-list.schema.json、json-output.md §5.4）声明
--   检查结果 severity 为 info/warning/hard，与约束定义枚举不一致。
-- - 本 migration 把 constraints 表 severity CHECK 放宽为
--   ('info', 'warning', 'hard')，并将存量 'soft' 数据迁移为 'warning'
--   （语义最接近：建议性规则、违反时不阻断）。
--
-- SQLite 不支持 ALTER COLUMN 改 CHECK，故用建新表+拷数据+drop+rename 范式。
-- rule_checks 表（V004）severity 已是 ('info','warning','hard')，无需改。

-- 1. 建新表（三档 CHECK）
CREATE TABLE constraints_new (
    constraint_type TEXT NOT NULL,
    scope           TEXT NOT NULL,
    name            TEXT NOT NULL,
    content         TEXT NOT NULL,
    severity        TEXT NOT NULL CHECK(severity IN ('info', 'warning', 'hard')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (constraint_type, scope, name)
);

-- 2. 拷数据：soft→warning，hard→hard
INSERT INTO constraints_new (constraint_type, scope, name, content, severity, created_at)
SELECT constraint_type, scope, name, content,
       CASE severity
           WHEN 'soft'  THEN 'warning'
           WHEN 'hard'  THEN 'hard'
           ELSE 'warning'
       END,
       created_at
FROM constraints;

-- 3. drop 旧表
DROP TABLE constraints;

-- 4. rename 新表
ALTER TABLE constraints_new RENAME TO constraints;

-- 5. 重建索引（保持与 V002 一致 + 新增 severity 索引）
CREATE INDEX IF NOT EXISTS idx_constraints_scope ON constraints(scope);
CREATE INDEX IF NOT EXISTS idx_constraints_severity ON constraints(severity);
