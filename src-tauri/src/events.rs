//! 后台线程与界面之间的事件通道。
//!
//! Swift 版用 `DispatchQueue.main.async` 把后台结果送回主线程；
//! Rust 版用 `std::sync::mpsc` 通道，事件循环每轮轮询一次。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::core::monitor::SystemInfo;
use crate::core::process::{PortInfo, ProcessInfo};

/// 后台产生的事件。
pub enum AppEvent {
    /// 进程列表刷新完成。
    Processes(Vec<ProcessInfo>),
    /// 端口信息缓存更新。
    PortCacheUpdated(HashMap<i32, Vec<PortInfo>>),
    /// 进程列表正在加载。
    ProcessLoading(bool),
    /// 系统监控数据更新。
    SystemInfo(SystemInfo),
    /// 剪贴板当前内容变化。
    ClipboardContent(String),
    /// 当前目录内容发生变化。
    FilesChanged,
    /// 菜单栏请求打开主窗口。
    TrayOpen,
    /// 菜单栏请求退出应用。
    TrayQuit,
    /// 全局快捷键被按下。
    Hotkey,
}

/// 创建事件通道。
pub fn channel() -> (Sender<AppEvent>, Receiver<AppEvent>) {
    std::sync::mpsc::channel()
}

/// 进程列表更新事件。
pub const EV_PROCESSES: &str = "processes";
/// 系统指标更新事件。
pub const EV_SYSTEM_INFO: &str = "system-info";
/// 剪贴板内容变化事件。
pub const EV_CLIPBOARD: &str = "clipboard";
/// 当前目录内容变化事件。
pub const EV_FILES_CHANGED: &str = "files-changed";
/// 托盘请求打开主窗口。
pub const EV_TRAY_OPEN: &str = "tray-open";
/// 托盘请求退出应用。
pub const EV_TRAY_QUIT: &str = "tray-quit";
/// 界面提示事件。
pub const EV_NOTICE: &str = "notice";
