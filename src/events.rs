//! 后台线程与 UI 之间的事件通道。
//!
//! Swift 版用 `DispatchQueue.main.async` 把后台结果送回主线程；Rust 版用
//! `std::sync::mpsc` 通道，UI 每帧轮询一次。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::monitor::SystemInfo;
use crate::process::{PortInfo, ProcessInfo};

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
