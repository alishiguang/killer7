//! chuaker7 —— macOS 系统工具箱（Tauri v2 + React 重写版）。
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
        .run(tauri::generate_context!())
        .expect("error while running chuaker7");
}
