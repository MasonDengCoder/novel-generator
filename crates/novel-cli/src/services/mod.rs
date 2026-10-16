// services 层：业务用例、状态机、权限边界、前置条件与跨实体一致性。
//
// 职责（CLAUDE.md §6、§10、CLI模块对齐清单 §5）：每个写命令有 service/use-case 承接；
// 状态机校验、权限矩阵、前置条件、跨实体一致性集中在 service/guard；service 编排
// repository 与 storage transaction；hard 阻断一律非零退出码，不静默降级。
//
// 本切片：init（创建世界）、info（查询概况）两个用例。

pub mod chapter_service;
pub mod chapter_context_service;
pub mod chapter_state_service;
pub mod constraint_service;
pub mod emergence_service;
pub mod export_service;
pub mod info;
pub mod init;
pub mod mainplot_service;
pub mod setting_service;
pub mod thread_service;
pub mod verify_service;
pub mod verdict_service;
pub mod volume_service;
