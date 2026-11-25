//! 插件命令。

use std::sync::Arc;

use tauri::State;

use crate::commands::dto::PluginRow;
use crate::core::plugin::PluginManager;
use crate::state::AppState;

/// 把插件清单映射成 DTO 行，并补上插件根目录。
fn to_rows(plugins: &PluginManager) -> Vec<PluginRow> {
    let dir = plugins.plugins_dir().to_string_lossy().to_string();
    plugins
        .loaded_plugins
        .iter()
        .map(|manifest| {
            let mut row = PluginRow::from(manifest);
            row.dir = dir.clone();
            row
        })
        .collect()
}

/// 列出已加载的插件。
#[tauri::command]
pub fn list_plugins(state: State<'_, Arc<AppState>>) -> Vec<PluginRow> {
    let plugins = state.plugins.lock().expect("插件锁中毒");
    to_rows(&plugins)
}

/// 重新扫描插件目录。
#[tauri::command]
pub fn reload_plugins(state: State<'_, Arc<AppState>>) -> Vec<PluginRow> {
    let mut plugins = state.plugins.lock().expect("插件锁中毒");
    plugins.load_plugins();
    to_rows(&plugins)
}

/// 插件目录路径。
#[tauri::command]
pub fn plugins_dir() -> String {
    PluginManager::default()
        .plugins_dir()
        .to_string_lossy()
        .to_string()
}
