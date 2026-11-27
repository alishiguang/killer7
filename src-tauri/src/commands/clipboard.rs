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

/// 清空历史。
///
/// 只在锁内改内存，落盘在锁外 —— 磁盘 IO 不该占着剪贴板锁。
/// 落盘失败会把错误回给前端。
#[tauri::command]
pub fn clear_history(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    {
        let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
        manager.clear_history();
    }
    clipboard::persist_history(|| {
        state
            .clipboard
            .lock()
            .expect("剪贴板锁中毒")
            .history
            .clone()
    })
}
