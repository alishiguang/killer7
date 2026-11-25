//! chuaker7 —— macOS 系统工具箱（Tauri v2 + React 重写版）。
//!
//! 业务逻辑在 `core/`，Tauri 命令门面在 `commands/`，原生集成在 `system/`。

/// Tauri Builder 装配入口。
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running chuaker7");
}
