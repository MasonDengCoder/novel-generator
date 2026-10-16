// domain/render_views.rs
// setting-constraint-render 切片：受控 view/scope 名枚举。
//
// 受控名引用 contracts/cli/views-scopes.md §A 冻结枚举：
// - SettingView（组 1，7 个）：intro|four-poles|world|style|summary|full|llm
// - ConstraintView（组 2，2 个）：llm|human
// - ConstraintScope（组 3，3 个）：world|volume|chapter
//
// 枚举只做受控名校验与字符串往返，不承载组合策略（策略在 service 层）。

use std::fmt;
use std::str::FromStr;

/// `novel setting render --view` 受控名（views-scopes.md §A 组 1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingView {
    /// 世界简介、核心设定摘要。
    Intro,
    /// 四极边界与不可越界的硬约束。
    FourPoles,
    /// 世界观细节，不含长篇风格样例。
    World,
    /// 文风、叙事质感、禁用表达等规则。
    Style,
    /// 压缩摘要，context 紧张时的兜底视图。
    Summary,
    /// 完整设定视图，人读审查、导出或离线核对。
    Full,
    /// 面向当前任务的预算内组合视图，Spec 默认写前入口。
    Llm,
}

impl SettingView {
    /// 全部受控名（用于错误提示与校验）。
    pub const ALL: [&'static str; 7] = [
        "intro",
        "four-poles",
        "world",
        "style",
        "summary",
        "full",
        "llm",
    ];
}

impl fmt::Display for SettingView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingView::Intro => write!(f, "intro"),
            SettingView::FourPoles => write!(f, "four-poles"),
            SettingView::World => write!(f, "world"),
            SettingView::Style => write!(f, "style"),
            SettingView::Summary => write!(f, "summary"),
            SettingView::Full => write!(f, "full"),
            SettingView::Llm => write!(f, "llm"),
        }
    }
}

impl FromStr for SettingView {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "intro" => Ok(SettingView::Intro),
            "four-poles" => Ok(SettingView::FourPoles),
            "world" => Ok(SettingView::World),
            "style" => Ok(SettingView::Style),
            "summary" => Ok(SettingView::Summary),
            "full" => Ok(SettingView::Full),
            "llm" => Ok(SettingView::Llm),
            _ => Err(format!(
                "Invalid setting view: '{}'. Must be one of: {}.",
                s,
                SettingView::ALL.join("|")
            )),
        }
    }
}

/// `novel constraint render --view` 受控名（views-scopes.md §A 组 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintView {
    /// 面向 LLM 的规则视图，机器可消费。
    Llm,
    /// 人读规则视图。
    Human,
}

impl ConstraintView {
    pub const ALL: [&'static str; 2] = ["llm", "human"];
}

impl fmt::Display for ConstraintView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintView::Llm => write!(f, "llm"),
            ConstraintView::Human => write!(f, "human"),
        }
    }
}

impl FromStr for ConstraintView {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "llm" => Ok(ConstraintView::Llm),
            "human" => Ok(ConstraintView::Human),
            _ => Err(format!(
                "Invalid constraint view: '{}'. Must be one of: {}.",
                s,
                ConstraintView::ALL.join("|")
            )),
        }
    }
}

/// `novel constraint render --scope` 受控名（views-scopes.md §A 组 3）。
///
/// 注意：受控 `--scope` 是三枚举，与既有 `constraint define`/`render --type` 形态的
/// 自由字符串 `--scope`（如 `volume:v1`）语义不同——受控形态由 `--view` 出现触发，
/// 走独立校验分支，不共享自由字符串 scope 校验路径。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintScope {
    /// 全局不变式。
    World,
    /// 当前卷相关约束（需 `--volume-id` 或 `--chapter-id` 定位卷）。
    Volume,
    /// 本章相关约束（需 `--chapter-id`）。
    Chapter,
}

impl ConstraintScope {
    pub const ALL: [&'static str; 3] = ["world", "volume", "chapter"];
}

impl fmt::Display for ConstraintScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintScope::World => write!(f, "world"),
            ConstraintScope::Volume => write!(f, "volume"),
            ConstraintScope::Chapter => write!(f, "chapter"),
        }
    }
}

impl FromStr for ConstraintScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "world" => Ok(ConstraintScope::World),
            "volume" => Ok(ConstraintScope::Volume),
            "chapter" => Ok(ConstraintScope::Chapter),
            _ => Err(format!(
                "Invalid controlled scope: '{}'. Must be one of: {}.",
                s,
                ConstraintScope::ALL.join("|")
            )),
        }
    }
}
