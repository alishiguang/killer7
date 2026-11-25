//! 应用设置与持久化。
//!
//! Swift 版用 `@AppStorage` 把设置写在 `UserDefaults` 里；Rust 版改为把同一份
//! 配置写到 `~/Library/Application Support/chuaker7/settings.json`，字段名保持一致，
//! 方便对照迁移。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Locale;
use crate::theme::{Accent, ThemeMode};

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
pub struct Settings {
    /// 是否启用菜单栏模式。
    pub show_menu_bar: bool,
    /// 自动刷新间隔（秒）。
    pub auto_refresh_interval: f64,
    /// 进程列表上限，0 表示不限制。
    pub process_list_limit: usize,
    /// 是否启用网络监控。
    pub enable_network_monitor: bool,
    /// 主题模式。
    pub theme_mode: ThemeMode,
    /// 强调色。
    pub accent: Accent,
    /// 是否开机自启动。
    pub launch_at_login: bool,
    /// 是否启用全局快捷键。
    pub global_shortcut_enabled: bool,
    /// 界面语言。
    pub locale: Locale,
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
