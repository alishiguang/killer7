//! Ki6 —— macOS 系统工具箱（Tauri v2 + React 重写版）。
//!
//! 业务逻辑在 `core/`，Tauri 命令门面在 `commands/`，原生集成在 `system/`。

pub mod commands;
pub mod core;
pub mod events;
pub mod state;
pub mod system;

use std::sync::Arc;

use tauri::Manager;

use state::AppState;

/// Tauri Builder 装配入口。
pub fn run() {
    let settings = crate::core::settings::Settings::load();
    let app_state = AppState::new(settings);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(app_state))
        .invoke_handler(commands::handler())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<Arc<AppState>>().inner().clone();

            // 菜单文案跟着设置里的语言走。
            let locale = state.settings.lock().expect("设置锁中毒").locale;
            crate::system::menu::build(&handle, locale)?;
            crate::system::tray::build(&handle, locale)?;

            // 全局快捷键 ⌘L：**默认开启**（core/settings.rs 的默认值是 true），
            // 设置项可热更新。注册失败必须留下痕迹，否则「显示已开启但毫无反应」
            // 会变成无法排查的静默失效。
            {
                let shortcut_enabled = state
                    .settings
                    .lock()
                    .expect("设置锁中毒")
                    .global_shortcut_enabled;
                if shortcut_enabled && let Err(err) = crate::system::shortcut::register(&handle) {
                    eprintln!("[shortcut] 注册 ⌘L 失败: {err}");
                }
            }

            // Dock / 菜单栏模式必须在启动时按已保存的设置复现一次。
            // 只在 `update_settings` / `set_menu_bar_mode` 里应用是不够的：
            // 那两条路径只覆盖「本次运行中切换」的情况，用户开启菜单栏模式后
            // 重启应用会重新变回常规 Dock 模式，设置形同虚设。
            // 这里的顺序与 `commands::system::set_menu_bar_mode` 保持一致。
            {
                let show_menu_bar = state.settings.lock().expect("设置锁中毒").show_menu_bar;
                if show_menu_bar {
                    crate::system::activation::set_accessory_mode(true, &handle);
                    let _ = app.get_webview_window("main").map(|w| w.hide());
                }
            }

            crate::system::tasks::spawn_process_loop(handle.clone(), state.clone());
            crate::system::tasks::spawn_clipboard_loop(handle.clone(), state.clone());
            crate::system::tasks::spawn_monitor_loop(
                handle.clone(),
                state.clone(),
                state.sample_interval_ms.clone(),
                state.network_enabled.clone(),
            );
            crate::system::tasks::spawn_watcher(handle.clone(), state);

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            crate::system::menu::on_menu_event(app, id);
            // 右键菜单的 id 统一以 `ctx.` 开头，与应用菜单的 `nav:` / `app:` 区分开。
            if id.starts_with("ctx.") {
                crate::system::menu::on_context_menu_event(app, id);
            }
        })
        .on_tray_icon_event(|app, event| {
            crate::system::tray::on_tray_event(app, &event);
        })
        .run(tauri::generate_context!())
        .expect("error while running Ki6");
}
