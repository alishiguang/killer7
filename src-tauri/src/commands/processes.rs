//! 进程管理命令。

use std::sync::Arc;

use tauri::State;

use crate::commands::dto::ProcessRow;
use crate::core::process::ProcessManager;
use crate::state::AppState;

/// 列出当前进程列表。
#[tauri::command]
pub fn list_processes(state: State<'_, Arc<AppState>>) -> Vec<ProcessRow> {
    let processes = state.processes.lock().expect("进程锁中毒");
    processes.processes.iter().map(ProcessRow::from).collect()
}

/// 触发一次进程列表刷新（同步采集后立即返回）。
///
/// 采集要跑 `ps` 与 `lsof`，几十毫秒起步；`async` 让它走线程池，
/// 不占用 Tauri 主线程，窗口拖拽与动画不会被卡住。
#[tauri::command(async)]
pub fn refresh_processes(state: State<'_, Arc<AppState>>) -> Vec<ProcessRow> {
    let limit = state
        .settings
        .lock()
        .expect("设置锁中毒")
        .process_list_limit;
    let mut processes = state.processes.lock().expect("进程锁中毒");
    // 采集与回写由 core 的单一入口完成，命令侧只负责取数与映射。
    processes.collect_and_store(limit);
    processes.processes.iter().map(ProcessRow::from).collect()
}

/// 结束指定进程。
#[tauri::command]
pub fn kill_process(pid: i32) -> Result<bool, String> {
    // `kill -1` 会广播到该用户的全部进程，`kill 0` 会打到进程组，
    // 两者都不能透传。
    if pid <= 0 {
        return Err(format!("非法 pid {pid}"));
    }
    if ProcessManager::kill_process(pid) {
        Ok(true)
    } else {
        Err(format!("结束进程 {pid} 失败，可能需要管理员权限"))
    }
}
