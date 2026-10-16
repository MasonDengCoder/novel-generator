// guards 子模块：状态推进门禁（s04-state-machine-and-verdicts）与卷冻结门禁（s07）。
//
// 职责（CLAUDE.md §6、§10）：状态机转换合法性判定 + 进入 review/approved 的硬阻断门禁 +
// 归档卷跨实体冻结判定。门禁在事务内由 service 调用，收集全部失败项一次性报告（非短路）。
// 外部修改检测由 service 层独立完成（退出码 21，优先于门禁）。
//
// s07 归档冻结：卷 archived 时拒绝修改其下属实体，返回退出码 3 + volume_already_archived。
// 冻结判定集中在共享 guard，各写 service 调用同一 guard，避免散落各 repository/storage 层。

pub mod state_transition_guard;
pub mod volume_freeze_guard;

