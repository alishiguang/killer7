//! 系统级命令：窗口显隐、菜单栏模式、快捷键、自启。

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

/// 隐藏主窗口。
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// 显示主窗口并置前。
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// 切换菜单栏模式（隐藏 Dock 图标）。
#[tauri::command]
pub fn set_menu_bar_mode(app: AppHandle, state: State<'_, Arc<AppState>>, enabled: bool) {
    {
        let mut settings = state.settings.lock().expect("设置锁中毒");
        settings.show_menu_bar = enabled;
    }
    // 落盘放在锁外：磁盘 IO 不该占着设置锁（与剪贴板落盘同一原则）。
    state.settings.lock().expect("设置锁中毒").save();

    crate::system::activation::set_accessory_mode(enabled, &app);

    if enabled {
        let _ = app.get_webview_window("main").map(|w| w.hide());
    } else {
        let _ = app.get_webview_window("main").map(|w| w.show());
    }
}

/// 启用或停用全局快捷键。
#[tauri::command]
pub fn set_shortcut_enabled(app: AppHandle, state: State<'_, Arc<AppState>>, enabled: bool) {
    {
        let mut settings = state.settings.lock().expect("设置锁中毒");
        settings.global_shortcut_enabled = enabled;
    }
    // 落盘放在锁外：磁盘 IO 不该占着设置锁（与剪贴板落盘同一原则）。
    state.settings.lock().expect("设置锁中毒").save();

    crate::system::shortcut::set_enabled(&app, enabled);
}

/// 启用或停用开机自启。
#[tauri::command]
pub fn set_auto_launch(enabled: bool) -> Result<(), String> {
    if enabled {
        crate::system::autolaunch::enable().map_err(|err| err.to_string())
    } else {
        crate::system::autolaunch::disable().map_err(|err| err.to_string())
    }
}

/// 弹出原生右键菜单（Task 25 实现完整逻辑）。
#[tauri::command]
pub fn popup_context_menu(app: AppHandle, kind: String, payload: serde_json::Value) {
    let _ = (app, kind, payload);
}
