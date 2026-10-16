// repositories 层：封装章节 CRUD 操作，调用 storage 层。
//
// 职责（CLAUDE.md §6）：封装 SQL 与文件引用读取，不做业务判断。
// 调用 storage 层的 transaction、临时文件、原子替换函数。

pub mod chapter_repository;
pub mod emergence_repository;
pub mod export_repository;
pub mod mainplot_repository;
pub mod rule_check_repository;
pub mod thread_repository;
pub mod verdict_repository;
pub mod volume_repository;
