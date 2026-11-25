//! 主题设置。
//!
//! 视觉语言参考 cindy.cn：近黑的「ink」底色 + 奶油色文字 + 单一红色强调色。
//! 这里只保留**设置项**（模式 + 强调色），具体的颜色渲染由前端 `tokens.css`
//! 负责，Rust 侧是这些设置项的唯一权威来源。

use serde::{Deserialize, Serialize};

/// 主题模式。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// 跟随系统
    System,
    /// 浅色模式
    Light,
    /// 深色模式（默认，参考站的 ink 配色）
    #[default]
    Dark,
}

impl ThemeMode {
    /// 全部可选模式。
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark];
}

/// 强调色。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Accent {
    /// 信号红，参考站的主色。
    #[default]
    Signal,
    /// 雾蓝，参考站的次要强调色。
    Haze,
    /// 苔绿。
    Moss,
    /// 琥珀。
    Amber,
}

impl Accent {
    /// 全部强调色。
    pub const ALL: [Accent; 4] = [Accent::Signal, Accent::Haze, Accent::Moss, Accent::Amber];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark_signal() {
        assert_eq!(ThemeMode::default(), ThemeMode::Dark);
        assert_eq!(Accent::default(), Accent::Signal);
    }

    #[test]
    fn serializes_in_stable_wire_format() {
        assert_eq!(
            serde_json::to_string(&ThemeMode::Dark).unwrap(),
            "\"dark\""
        );
        assert_eq!(
            serde_json::to_string(&Accent::Signal).unwrap(),
            "\"signal\""
        );
    }
}
