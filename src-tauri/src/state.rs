//! 全局共享状态。
//!
//! 所有 command 与后台线程共享同一个 `AppState`（通过 `tauri::State`），
//! 内部用 `Mutex` 保护可变部分，采样间隔与网络开关用原子变量以便热改。
//!
//! `AppState` 本身由 `Arc` 托管（`Arc<AppState>`），**不再外套一层 `Mutex`**：
//! 每个可变字段各自带 `Mutex`，套一层只会把所有子系统串行化——
//! 进程采集要跑 `ps`（几十毫秒），期间不该把设置、剪贴板命令一起卡住。

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

use crate::core::clipboard::ClipboardManager;
use crate::core::files::FileManager;
use crate::core::i18n::I18n;
use crate::core::monitor::SystemMonitor;
use crate::core::plugin::PluginManager;
use crate::core::process::ProcessManager;
use crate::core::settings::Settings;

/// 应用共享状态。
pub struct AppState {
    /// 全局设置。
    pub settings: Mutex<Settings>,
    /// 文案表。
    pub i18n: Mutex<I18n>,
    /// 进程管理器。
    pub processes: Mutex<ProcessManager>,
    /// 文件管理器。
    pub files: Mutex<FileManager>,
    /// 剪贴板管理器。
    pub clipboard: Mutex<ClipboardManager>,
    /// 系统监控。
    pub monitor: Mutex<SystemMonitor>,
    /// 插件管理器。
    pub plugins: Mutex<PluginManager>,
    /// 采样间隔（毫秒），可在设置里实时调整。
    pub sample_interval_ms: Arc<AtomicU64>,
    /// 是否统计网络流量。
    pub network_enabled: Arc<AtomicBool>,
    /// 当前目录的文件变更监听器。
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

impl AppState {
    /// 用磁盘上的设置构造状态。
    pub fn new(settings: Settings) -> Self {
        let sample_interval_ms = Arc::new(AtomicU64::new(
            (settings.auto_refresh_interval * 1000.0) as u64,
        ));
        let network_enabled = Arc::new(AtomicBool::new(settings.enable_network_monitor));

        let locale = settings.locale;
        let mut clipboard = ClipboardManager::default();
        clipboard.load_history();

        let mut plugins = PluginManager::default();
        plugins.load_plugins();

        // 启动时同步采一次进程列表：窗口一打开就有数据，不必等后台循环的第一轮。
        // 实测 `ps` 约 30ms、`lsof` 约 55ms，这个代价可以接受；
        // 不这么做的话 `list_processes` 会拿到空列表，进程页要空好几秒。
        let mut processes = ProcessManager::default();
        processes.load_blocking(settings.process_list_limit);

        Self {
            i18n: Mutex::new(I18n::new(locale)),
            processes: Mutex::new(processes),
            files: Mutex::new(FileManager::default()),
            clipboard: Mutex::new(clipboard),
            monitor: Mutex::new(SystemMonitor::default()),
            plugins: Mutex::new(plugins),
            sample_interval_ms,
            network_enabled,
            watcher: Mutex::new(None),
            settings: Mutex::new(settings),
        }
    }
}
