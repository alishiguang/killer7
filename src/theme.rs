//! 主题系统，对应 Swift 版的 `ThemeManager.swift`。
//!
//! 视觉语言参考 cindy.cn：近黑的「ink」底色 + 奶油色文字 + 单一红色强调色，
//! 配以发丝级描边、小圆角卡片和等宽小标题。Swift 版提供「跟随系统 / 浅色 / 深色」
//! 三档模式 + 可自定义调色板，这里保留同样的两级结构：先按模式选基础色板，
//! 再叠加一组强调色。

use egui::Color32;
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

    /// 强调色在深色底上的取值。
    fn dark(self) -> Color32 {
        match self {
            Accent::Signal => rgb(0xF43D3F),
            Accent::Haze => rgb(0xA9C6E8),
            Accent::Moss => rgb(0x7FB069),
            Accent::Amber => rgb(0xE9B44C),
        }
    }

    /// 强调色在浅色底上的取值。
    fn light(self) -> Color32 {
        match self {
            Accent::Signal => rgb(0xCF2B30),
            Accent::Haze => rgb(0x4A6FA5),
            Accent::Moss => rgb(0x3F6B4A),
            Accent::Amber => rgb(0x9A6B12),
        }
    }
}

/// 一套完整配色。
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// 页面底色。
    pub background: Color32,
    /// 面板 / 卡片底色。
    pub surface: Color32,
    /// 悬浮 / 高层卡片底色。
    pub surface_alt: Color32,
    /// 输入框等控件底色。
    pub control: Color32,
    /// 正文色。
    pub text: Color32,
    /// 次要文字色。
    pub text_muted: Color32,
    /// 发丝描边。
    pub border: Color32,
    /// 强调色。
    pub accent: Color32,
    /// 强调色上的文字。
    pub on_accent: Color32,
    /// 成功色。
    pub success: Color32,
    /// 警告色。
    pub warning: Color32,
    /// 危险色。
    pub danger: Color32,
}

impl Palette {
    /// 深色（ink）配色表。
    fn ink(accent: Accent) -> Self {
        Self {
            background: rgb(0x0F0E15),
            surface: rgb(0x16151E),
            surface_alt: rgb(0x201F2A),
            control: rgb(0x24222F),
            text: rgb(0xF3EDE2),
            text_muted: rgb(0x918F9E),
            border: Color32::from_rgba_unmultiplied(0xF3, 0xED, 0xE2, 0x14),
            accent: accent.dark(),
            on_accent: rgb(0x0F0E15),
            success: rgb(0x7FB069),
            warning: rgb(0xE9B44C),
            danger: rgb(0xF43D3F),
        }
    }

    /// 浅色（paper）配色表。
    fn paper(accent: Accent) -> Self {
        Self {
            background: rgb(0xFAF6EE),
            surface: rgb(0xF3EDE2),
            surface_alt: rgb(0xEAE2D2),
            control: rgb(0xFFFFFF),
            text: rgb(0x14131B),
            text_muted: rgb(0x5D5B66),
            border: Color32::from_rgba_unmultiplied(0x14, 0x13, 0x1B, 0x2E),
            accent: accent.light(),
            on_accent: rgb(0xFAF6EE),
            success: rgb(0x3F6B4A),
            warning: rgb(0x9A6B12),
            danger: rgb(0xCF2B30),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::ink(Accent::Signal)
    }
}

/// 主题管理器。
#[derive(Clone, Debug, Default)]
pub struct ThemeManager {
    /// 显示模式。
    pub mode: ThemeMode,
    /// 强调色。
    pub accent: Accent,
}

impl ThemeManager {
    /// 是否处于深色外观。
    pub fn is_dark(&self, system_dark: bool) -> bool {
        match self.mode {
            ThemeMode::System => system_dark,
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
        }
    }

    /// 当前配色。
    pub fn palette(&self, system_dark: bool) -> Palette {
        if self.is_dark(system_dark) {
            Palette::ink(self.accent)
        } else {
            Palette::paper(self.accent)
        }
    }

    /// 生成 egui 视觉风格。
    pub fn visuals(&self, system_dark: bool) -> egui::Visuals {
        let palette = self.palette(system_dark);
        let mut visuals = if self.is_dark(system_dark) {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        visuals.panel_fill = palette.background;
        visuals.window_fill = palette.background;
        visuals.extreme_bg_color = palette.control;
        visuals.faint_bg_color = palette.surface;
        visuals.override_text_color = Some(palette.text);
        visuals.hyperlink_color = palette.accent;
        visuals.selection.bg_fill = palette.accent;
        visuals.selection.stroke = egui::Stroke::new(1.0, palette.accent);

        visuals.widgets.noninteractive.bg_fill = palette.surface;
        visuals.widgets.noninteractive.weak_bg_fill = palette.surface;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, palette.border);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, palette.text_muted);

        visuals.widgets.inactive.bg_fill = palette.surface;
        visuals.widgets.inactive.weak_bg_fill = palette.surface;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, palette.border);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, palette.text);

        visuals.widgets.hovered.bg_fill = palette.surface_alt;
        visuals.widgets.hovered.weak_bg_fill = palette.surface_alt;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, palette.accent);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, palette.text);

        visuals.widgets.active.bg_fill = palette.accent;
        visuals.widgets.active.weak_bg_fill = palette.accent;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.accent);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, palette.on_accent);

        visuals.widgets.open.bg_fill = palette.surface_alt;
        visuals.widgets.open.weak_bg_fill = palette.surface_alt;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, palette.accent);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, palette.text);

        visuals.error_fg_color = palette.danger;
        visuals.warn_fg_color = palette.warning;

        visuals
    }
}

/// 由十六进制构造不透明颜色。
fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_and_light_palettes_differ() {
        let dark = ThemeManager {
            mode: ThemeMode::Dark,
            accent: Accent::Signal,
        };
        let light = ThemeManager {
            mode: ThemeMode::Light,
            accent: Accent::Signal,
        };

        let ink = dark.palette(false);
        let paper = light.palette(true);
        assert_ne!(ink.background, paper.background);
        assert_eq!(ink.text, rgb(0xF3EDE2));
        assert_eq!(paper.background, rgb(0xFAF6EE));
    }

    #[test]
    fn system_mode_follows_system() {
        let manager = ThemeManager {
            mode: ThemeMode::System,
            accent: Accent::Signal,
        };
        assert!(manager.is_dark(true));
        assert!(!manager.is_dark(false));
    }
}
