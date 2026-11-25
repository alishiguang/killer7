//! 应用设置与持久化。
//!
//! Swift 版用 `@AppStorage` 把设置写在 `UserDefaults` 里；Rust 版写到
//! `~/Library/Application Support/chuaker7/settings.json`。
//!
//! 字段以 camelCase 落盘（与前端契约一致），同时通过 `alias` 接受 egui 版
//! 写下的 snake_case 旧字段，用户设置不会因为改版而丢失。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::i18n::Locale;
use crate::core::theme::{Accent, ThemeMode};

/// 应用数据目录（macOS 下为 `~/Library/Application Support/chuaker7`）。
pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("chuaker7")
}

/// 配置文件路径。
pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

/// 剪贴板历史文件路径。
pub fn clipboard_history_path() -> PathBuf {
    data_dir().join("clipboard_history.json")
}

/// 插件目录（对应 Swift 版的 `Application Support/LuciBox/Plugins`）。
pub fn plugins_dir() -> PathBuf {
    data_dir().join("Plugins")
}

/// 全局设置。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// 是否启用菜单栏模式。
    #[serde(alias = "show_menu_bar")]
    pub show_menu_bar: bool,
    /// 自动刷新间隔（秒）。
    #[serde(alias = "auto_refresh_interval")]
    pub auto_refresh_interval: f64,
    /// 进程列表上限，0 表示不限制。
    #[serde(alias = "process_list_limit")]
    pub process_list_limit: usize,
    /// 是否启用网络监控。
    #[serde(alias = "enable_network_monitor")]
    pub enable_network_monitor: bool,
    /// 主题模式。
    #[serde(alias = "theme_mode")]
    pub theme_mode: ThemeMode,
    /// 强调色。
    pub accent: Accent,
    /// 是否开机自启动。
    #[serde(alias = "launch_at_login")]
    pub launch_at_login: bool,
    /// 是否启用全局快捷键。
    #[serde(alias = "global_shortcut_enabled")]
    pub global_shortcut_enabled: bool,
    /// 界面语言。
    pub locale: Locale,
}

/// 设置补丁：全字段可选，前端只提交实际改动的字段。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    /// 菜单栏模式。
    #[serde(alias = "show_menu_bar")]
    pub show_menu_bar: Option<bool>,
    /// 自动刷新间隔（秒）。
    #[serde(alias = "auto_refresh_interval")]
    pub auto_refresh_interval: Option<f64>,
    /// 进程列表上限。
    #[serde(alias = "process_list_limit")]
    pub process_list_limit: Option<usize>,
    /// 网络监控开关。
    #[serde(alias = "enable_network_monitor")]
    pub enable_network_monitor: Option<bool>,
    /// 主题模式。
    #[serde(alias = "theme_mode")]
    pub theme_mode: Option<ThemeMode>,
    /// 强调色。
    pub accent: Option<Accent>,
    /// 开机自启。
    #[serde(alias = "launch_at_login")]
    pub launch_at_login: Option<bool>,
    /// 全局快捷键。
    #[serde(alias = "global_shortcut_enabled")]
    pub global_shortcut_enabled: Option<bool>,
    /// 界面语言。
    pub locale: Option<Locale>,
}

impl SettingsPatch {
    /// 把补丁合并进设置，返回新设置。
    pub fn apply(self, base: &Settings) -> Settings {
        let mut next = base.clone();
        if let Some(v) = self.show_menu_bar {
            next.show_menu_bar = v;
        }
        if let Some(v) = self.auto_refresh_interval {
            next.auto_refresh_interval = v;
        }
        if let Some(v) = self.process_list_limit {
            next.process_list_limit = v;
        }
        if let Some(v) = self.enable_network_monitor {
            next.enable_network_monitor = v;
        }
        if let Some(v) = self.theme_mode {
            next.theme_mode = v;
        }
        if let Some(v) = self.accent {
            next.accent = v;
        }
        if let Some(v) = self.launch_at_login {
            next.launch_at_login = v;
        }
        if let Some(v) = self.global_shortcut_enabled {
            next.global_shortcut_enabled = v;
        }
        if let Some(v) = self.locale {
            next.locale = v;
        }
        next
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_menu_bar: false,
            auto_refresh_interval: 5.0,
            process_list_limit: 500,
            enable_network_monitor: true,
            theme_mode: ThemeMode::default(),
            accent: Accent::default(),
            launch_at_login: false,
            global_shortcut_enabled: true,
            locale: Locale::detect(),
        }
    }
}

impl Settings {
    /// 从磁盘读取设置，文件缺失或损坏时返回默认值。
    pub fn load() -> Self {
        let Ok(text) = std::fs::read_to_string(settings_path()) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// 写回磁盘，失败时忽略（设置保存失败不应影响使用）。
    pub fn save(&self) {
        let dir = data_dir();
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(settings_path(), text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_only_touches_given_fields() {
        let base = Settings::default();
        let patch = SettingsPatch {
            process_list_limit: Some(100),
            theme_mode: Some(ThemeMode::Light),
            ..Default::default()
        };
        let next = patch.apply(&base);
        assert_eq!(next.process_list_limit, 100);
        assert_eq!(next.theme_mode, ThemeMode::Light);
        assert_eq!(next.auto_refresh_interval, base.auto_refresh_interval);
        assert_eq!(next.accent, base.accent);
    }

    #[test]
    fn reads_legacy_snake_case_config() {
        let legacy = r#"{
            "show_menu_bar": true,
            "auto_refresh_interval": 8.0,
            "process_list_limit": 42,
            "enable_network_monitor": false,
            "theme_mode": "light",
            "accent": "moss",
            "launch_at_login": true,
            "global_shortcut_enabled": false,
            "locale": "en"
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).expect("旧配置应可解析");
        assert!(parsed.show_menu_bar);
        assert_eq!(parsed.auto_refresh_interval, 8.0);
        assert_eq!(parsed.process_list_limit, 42);
        assert!(!parsed.enable_network_monitor);
        assert_eq!(parsed.theme_mode, ThemeMode::Light);
        assert_eq!(parsed.accent, Accent::Moss);
        assert!(parsed.launch_at_login);
        assert!(!parsed.global_shortcut_enabled);
        assert_eq!(parsed.locale, Locale::En);
    }

    #[test]
    fn writes_camel_case_config() {
        let text = serde_json::to_string(&Settings::default()).unwrap();
        assert!(text.contains("\"showMenuBar\""));
        assert!(text.contains("\"autoRefreshInterval\""));
    }
}
