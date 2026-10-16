// domain/constraint.rs
// s01-project-bootstrap：Constraint 实体与枚举。
//
// Constraint 是创作约束规则，包含类型、作用域、名称、内容和严重等级。
// 复合主键 (constraint_type, scope, name)，支持层级作用域（world / volume:v1 / chapter:c1）。

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 约束严重等级
///
/// 三档分级，与 `contracts/cli/json-schemas/constraint-check-list.schema.json` 及
/// `json-output.md §5.4` 声明的 `info/warning/hard` 枚举一致。`hard` 是唯一
/// 阻断级（状态机门禁、P0 上下文片段），`info`/`warning` 不阻断。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConstraintSeverity {
    /// 信息级：提示性规则，违反时不阻断、仅记录
    Info,
    /// 警告级：建议性规则，违反时给出警告但不阻断
    Warning,
    /// 硬约束：强制性规则，违反时阻断流程
    Hard,
}

impl fmt::Display for ConstraintSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintSeverity::Info => write!(f, "info"),
            ConstraintSeverity::Warning => write!(f, "warning"),
            ConstraintSeverity::Hard => write!(f, "hard"),
        }
    }
}

impl FromStr for ConstraintSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(ConstraintSeverity::Info),
            "warning" => Ok(ConstraintSeverity::Warning),
            "hard" => Ok(ConstraintSeverity::Hard),
            _ => Err(format!(
                "Invalid severity: '{}'. Must be one of: info, warning, hard.",
                s
            )),
        }
    }
}

/// 约束类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConstraintType {
    /// 情节约束（如时间线一致性、因果关系）
    Plot,
    /// 角色约束（如性格一致性、行为合理性）
    Character,
    /// 世界观约束（如魔法体系、科技水平）
    World,
    /// 风格约束（如叙事视角、语言风格）
    Style,
    /// 自定义约束
    Custom,
}

impl fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstraintType::Plot => write!(f, "plot"),
            ConstraintType::Character => write!(f, "character"),
            ConstraintType::World => write!(f, "world"),
            ConstraintType::Style => write!(f, "style"),
            ConstraintType::Custom => write!(f, "custom"),
        }
    }
}

impl FromStr for ConstraintType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plot" => Ok(ConstraintType::Plot),
            "character" => Ok(ConstraintType::Character),
            "world" => Ok(ConstraintType::World),
            "style" => Ok(ConstraintType::Style),
            "custom" => Ok(ConstraintType::Custom),
            _ => Err(format!(
                "Invalid constraint type: '{}'. Must be one of: plot, character, world, style, custom.",
                s
            )),
        }
    }
}

/// 创作约束规则实体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    /// 约束类型
    #[serde(rename = "constraintType")]
    pub constraint_type: ConstraintType,
    /// 作用域（如 "world", "volume:v1", "chapter:c1"）
    pub scope: String,
    /// 约束名称
    pub name: String,
    /// 约束内容（纯文本，不做语法验证）
    pub content: String,
    /// 严重等级
    pub severity: ConstraintSeverity,
    /// 创建时间（ISO 8601 格式）
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

impl Constraint {
    /// 创建新的 Constraint 实例
    pub fn new(
        constraint_type: ConstraintType,
        scope: String,
        name: String,
        content: String,
        severity: ConstraintSeverity,
        created_at: String,
    ) -> Self {
        Self {
            constraint_type,
            scope,
            name,
            content,
            severity,
            created_at,
        }
    }
}
