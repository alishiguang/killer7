//! 剪贴板命令。

use std::sync::Arc;

use tauri::State;

use crate::commands::dto::HistoryRow;
use crate::core::clipboard;
use crate::state::AppState;

/// 读取当前剪贴板文本。
#[tauri::command]
pub fn clipboard_current() -> String {
    clipboard::read_current().unwrap_or_default()
}

/// 读取剪贴板历史。
#[tauri::command]
pub fn clipboard_history(state: State<'_, Arc<AppState>>) -> Vec<HistoryRow> {
    let manager = state.clipboard.lock().expect("剪贴板锁中毒");
    manager.history.iter().map(HistoryRow::from).collect()
}

/// 删除一条历史（按稳定 id）。
///
/// 只在锁内改内存，快照出来之后**在锁外落盘** —— 磁盘 IO 不该占着剪贴板锁。
#[tauri::command]
pub fn remove_history_item(state: State<'_, Arc<AppState>>, id: u64) {
    let snapshot = {
        let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
        if manager.delete_history_item(id) {
            Some(manager.history.clone())
        } else {
            None
        }
    };
    if let Some(history) = snapshot {
        clipboard::persist_history(&history);
    }
}

/// 清空历史。
///
/// 只在锁内改内存，快照出来之后**在锁外落盘** —— 磁盘 IO 不该占着剪贴板锁。
#[tauri::command]
pub fn clear_history(state: State<'_, Arc<AppState>>) {
    let snapshot = {
        let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
        manager.clear_history();
        manager.history.clone()
    };
    clipboard::persist_history(&snapshot);
}
