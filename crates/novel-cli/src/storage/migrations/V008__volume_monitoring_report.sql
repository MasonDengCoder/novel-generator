-- V008__volume_monitoring_report.sql
-- s07-archive-and-export：卷末监测报告路径引用列。
--
-- 卷末监测报告写为正文文件（.novel/content/reports/volume-<id>.md，人读 Markdown），
-- SQLite volumes 表记录报告路径引用，与章节正文"文件 + 元数据索引"策略一致（CLAUDE.md §4/§7，design D8）。
--
-- 注意（CLAUDE.md §9）：SQLite schema、迁移文件名均为非稳定实现细节，不作为跨层契约。
-- 稳定的 CLI Contract 位于 contracts/cli/commands.md 与 json-schemas/*.schema.json。

ALTER TABLE volumes ADD COLUMN monitoring_report_path TEXT;
