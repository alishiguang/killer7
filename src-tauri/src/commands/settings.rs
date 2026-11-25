//! 设置与文案命令。

use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::core::i18n::{I18n, Locale};
use crate::core::settings::{Settings, SettingsPatch};
use crate::state::AppState;

/// 读取当前设置。
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    let settings = state.settings.lock().expect("设置锁中毒");
    settings.clone()
}

/// 提交设置补丁，返回合并后的完整设置。
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    patch: SettingsPatch,
) -> Settings {
    // 先取旧值：菜单栏模式与全局快捷键要靠「改了什么」判断是否重新作用到原生层。
    // 若在写入之后再取旧值，拿到的就是新值，比较永远相等，热更新就成了死代码。
    let previous = state.settings.lock().expect("设置锁中毒").clone();

    let next = {
        let mut settings = state.settings.lock().expect("设置锁中毒");
        let next = patch.apply(&settings);
        *settings = next.clone();
        next
    };

    // 落盘放在锁外：磁盘 IO 不该占着设置锁（与剪贴板落盘同一原则）。
    next.save();

    // 语言变了要换文案表。
    {
        let mut i18n = state.i18n.lock().expect("文案锁中毒");
        if i18n.locale() != next.locale {
            i18n.set_locale(next.locale);
        }
    }

    // 采样间隔与网络开关热更新。
    {
        state.sample_interval_ms.store(
            (next.auto_refresh_interval * 1000.0) as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        state.network_enabled.store(
            next.enable_network_monitor,
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    // 菜单栏模式与全局快捷键的变化需要立即作用到原生层。
    if previous.show_menu_bar != next.show_menu_bar {
        crate::system::activation::set_accessory_mode(next.show_menu_bar, &app);
    }

    if previous.global_shortcut_enabled != next.global_shortcut_enabled {
        crate::system::shortcut::set_enabled(&app, next.global_shortcut_enabled);
    }

    next
}

/// 取指定语言的整张文案表。
#[tauri::command]
pub fn get_strings(locale: String) -> HashMap<String, String> {
    let parsed = match locale.as_str() {
        "en" => Locale::En,
        _ => Locale::ZhHans,
    };
    I18n::new(parsed).table()
}
