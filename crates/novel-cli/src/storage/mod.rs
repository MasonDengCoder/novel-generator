// storage 层：SQLite transaction、migration、锁与文件写入原子性。
//
// 职责（CLAUDE.md §6、§7、CLI模块对齐清单 §9）：负责正式世界状态元数据存储（SQLite，
// 不得用临时 JSON 作为正式存储）；章节正文文件引用由 SQLite 索引；migration runner
// 逐 SQL 文件在事务内执行，失败回滚；写操作临时副本 + 原子替换 + 回滚；并发锁。
//
// SQLite schema、迁移文件名、`.novel/` 内部路径均为非稳定实现细节（CLAUDE.md §9），
// 不作为跨层契约。

pub mod chapters;
pub mod connection;
pub mod constraints;
pub mod file_ops;
pub mod metadata;
pub mod migrations;
pub mod settings;
