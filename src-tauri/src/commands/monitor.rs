//! 系统监控命令。

use std::sync::Arc;

use tauri::State;

use crate::commands::dto::SystemInfoDto;
use crate::state::AppState;

/// 读取一次系统指标快照（含采样历史）。
#[tauri::command]
pub fn system_snapshot(state: State<'_, Arc<AppState>>) -> SystemInfoDto {
    let monitor = state.monitor.lock().expect("监控锁中毒");
    // 进程总数直接读进程缓存的长度 —— 不额外采集，也不多付一次 IPC 负载。
    let process_count = state.processes.lock().expect("进程锁中毒").processes.len();
    SystemInfoDto {
        cpu_usage: monitor.system_info.cpu_usage,
        memory_usage: monitor.system_info.memory_usage,
        disk_usage: monitor.system_info.disk_usage,
        network_upload: monitor.system_info.network_upload,
        network_download: monitor.system_info.network_download,
        cpu_history: monitor.cpu_history.clone(),
        memory_history: monitor.memory_history.clone(),
        process_count,
    }
}
