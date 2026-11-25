//! 后台任务循环。
//!
//! 所有周期性与监听型后台任务都在这里启动。线程只做取数与派发，
//! 不直接操作界面；界面通过 Tauri 事件收到更新。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use notify::{Event, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::core::clipboard;
use crate::core::monitor;
use crate::events::{EV_CLIPBOARD, EV_FILES_CHANGED, EV_PROCESSES, EV_SYSTEM_INFO};
use crate::state::AppState;

/// 判断事件是否属于「当前目录内容变了」。
///
/// 契约里 `files-changed` 只在目录增删改时触发：新建、删除、重命名都要触发，
/// 读取与元数据变化不触发——否则一次目录遍历就能引发刷新风暴。
pub fn is_content_change(kind: &notify::EventKind) -> bool {
    matches!(
        kind,
        notify::EventKind::Create(_)
            | notify::EventKind::Remove(_)
            | notify::EventKind::Modify(notify::event::ModifyKind::Name(_))
    )
}

/// 启动进程列表定时刷新循环。
pub fn spawn_process_loop(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        loop {
            let (limit, interval_ms) = {
                let settings = state.settings.lock().expect("设置锁中毒");
                (
                    settings.process_list_limit,
                    state.sample_interval_ms.load(Ordering::Relaxed),
                )
            };

            let rows = {
                let mut processes = state.processes.lock().expect("进程锁中毒");
                let list = processes.collect(limit);
                let rows = list
                    .iter()
                    .map(crate::commands::dto::ProcessRow::from)
                    .collect::<Vec<_>>();
                // 回写缓存：`list_processes` 命令读的就是这个字段，
                // 不回写的话那条命令永远返回空列表。
                processes.processes = list;
                rows
            };

            let _ = app.emit(EV_PROCESSES, rows);

            std::thread::sleep(Duration::from_millis(interval_ms.max(500)));
        }
    });
}

/// 启动剪贴板轮询循环。
pub fn spawn_clipboard_loop(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        let mut last = String::new();
        loop {
            std::thread::sleep(Duration::from_secs(1));

            let Some(text) = clipboard::read_current() else {
                continue;
            };
            if text == last {
                continue;
            }
            last = text.clone();

            {
                let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
                manager.add_to_history(&text, "text");
            }

            let _ = app.emit(EV_CLIPBOARD, text);
        }
    });
}

/// 启动系统指标采样循环。
pub fn spawn_monitor_loop(
    app: AppHandle,
    state: Arc<AppState>,
    interval_ms: Arc<AtomicU64>,
    network_enabled: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let mut system = sysinfo::System::new_all();
        // 预热一次，让 CPU 使用率有可比较的基线。
        system.refresh_cpu_usage();

        // 网络计数器必须复用同一个实例：每次新建会让接口基线被重置，
        // 算出来的上传 / 下载速度永远是 0。
        let mut networks = sysinfo::Networks::new_with_refreshed_list();

        let mut first = true;
        loop {
            let interval = (interval_ms.load(Ordering::Relaxed) as f64 / 1000.0).max(0.5);

            if !first {
                std::thread::sleep(Duration::from_secs_f64(interval));
            }
            first = false;

            let info = monitor::sample(
                &mut system,
                &mut networks,
                interval,
                network_enabled.load(Ordering::Relaxed),
            );

            let dto = {
                let mut mon = state.monitor.lock().expect("监控锁中毒");
                mon.push_sample(info);
                crate::commands::dto::SystemInfoDto {
                    cpu_usage: mon.system_info.cpu_usage,
                    memory_usage: mon.system_info.memory_usage,
                    disk_usage: mon.system_info.disk_usage,
                    network_upload: mon.system_info.network_upload,
                    network_download: mon.system_info.network_download,
                    cpu_history: mon.cpu_history.clone(),
                    memory_history: mon.memory_history.clone(),
                }
            };

            let _ = app.emit(EV_SYSTEM_INFO, dto);
        }
    });
}

/// 启动当前目录的变更监听。
pub fn spawn_watcher(app: AppHandle, state: Arc<AppState>) {
    let path = {
        let files = state.files.lock().expect("文件锁中毒");
        files.current_path.clone()
    };

    let target = path.clone();
    let watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
        if let Ok(event) = result
            && is_content_change(&event.kind)
        {
            let _ = app.emit(EV_FILES_CHANGED, ());
        }
    });

    let Ok(mut watcher) = watcher else {
        eprintln!("[watcher] 无法创建目录监听器");
        return;
    };

    if let Err(err) = watcher.watch(&target, RecursiveMode::NonRecursive) {
        eprintln!("[watcher] 监听 {} 失败: {err}", target.display());
        return;
    }

    let mut slot = state.watcher.lock().expect("监听锁中毒");
    *slot = Some(watcher);
}
