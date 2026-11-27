//! 后台任务循环。
//!
//! 所有周期性与监听型后台任务都在这里启动。线程只做取数与派发，
//! 不直接操作界面；界面通过 Tauri 事件收到更新。

use std::path::Path;
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
            let limit = state
                .settings
                .lock()
                .expect("设置锁中毒")
                .process_list_limit;

            let rows = {
                let mut processes = state.processes.lock().expect("进程锁中毒");
                // 采集与回写由 core 的单一入口完成，漏掉回写会让
                // `list_processes` 永远返回空列表。
                processes.collect_and_store(limit);
                processes
                    .processes
                    .iter()
                    .map(crate::commands::dto::ProcessRow::from)
                    .collect::<Vec<_>>()
            };

            let _ = app.emit(EV_PROCESSES, rows);

            // 间隔在**睡之前**读：设置里改完刷新频率，下一轮立刻按新周期走，
            // 不用再多等一个旧间隔。
            let interval_ms = state.sample_interval_ms.load(Ordering::Relaxed);
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

            let changed = {
                let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
                manager.add_to_history(&text, "text")
            };

            // 落盘放在锁外：磁盘 IO 不该占着剪贴板锁。
            if changed {
                let result = clipboard::persist_history(|| {
                    state
                        .clipboard
                        .lock()
                        .expect("剪贴板锁中毒")
                        .history
                        .clone()
                });
                if let Err(err) = result {
                    eprintln!("[clipboard] 落盘失败: {err}");
                }
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

        // 采样器记住「上次刷新网络计数器」的时刻：
        // 关闭网络监控期间不刷新，重新打开时增量覆盖整个停用区间，
        // 除数必须用真实间隔而不是标称采样间隔。
        let mut sampler = monitor::NetworkSampler::new();

        let mut first = true;
        loop {
            let interval = (interval_ms.load(Ordering::Relaxed) as f64 / 1000.0).max(0.5);

            if !first {
                std::thread::sleep(Duration::from_secs_f64(interval));
            }
            first = false;

            let info = sampler.sample(
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
                    // 进程总数直接读进程缓存的长度，不额外采集。
                    process_count: state.processes.lock().expect("进程锁中毒").processes.len(),
                }
            };

            crate::system::tray::update_metrics(&app, &info);

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

    watch_path(&app, &state, &path);
}

/// 把监听器挂到指定目录，成功返回 true。
///
/// 监听器**只有一个槽位**：换目录就是替换，旧的 `RecommendedWatcher` 被 drop 时
/// 会自动解除对旧目录的监听。失败时**保留旧监听器**，并把原因打到 stderr ——
/// 静默失败会让前端以为已经切到新目录，实际却还在监听旧目录。
pub fn watch_path(app: &AppHandle, state: &Arc<AppState>, path: &Path) -> bool {
    let emitter = app.clone();
    let watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
        if let Ok(event) = result
            && is_content_change(&event.kind)
        {
            let _ = emitter.emit(EV_FILES_CHANGED, ());
        }
    });

    let Ok(mut watcher) = watcher else {
        eprintln!("[watcher] 无法创建目录监听器");
        return false;
    };

    if let Err(err) = watcher.watch(path, RecursiveMode::NonRecursive) {
        eprintln!("[watcher] 监听 {} 失败: {err}", path.display());
        return false;
    }

    *state.watcher.lock().expect("监听锁中毒") = Some(watcher);
    true
}
