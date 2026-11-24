//! 全局快捷键，对应 Swift 版的 `GlobalShortcutManager.swift`。
//!
//! Swift 版用 `NSEvent.addGlobalMonitorForEvents` 监听 ⌘+L；Rust 版使用
//! `global-hotkey` 注册系统级热键，功能一致但无需辅助功能权限。

use std::sync::mpsc::Sender;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

use crate::events::AppEvent;

/// 全局快捷键管理器。
pub struct ShortcutManager {
    /// hotkey 管理器。
    manager: GlobalHotKeyManager,
    /// 已注册的热键。
    hotkey: HotKey,
}

impl ShortcutManager {
    /// 注册 ⌘+L 全局快捷键。
    ///
    /// 注意：macOS 上要求在主线程创建管理器，否则注册会失败。
    pub fn new() -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyL);
        manager.register(hotkey).ok()?;
        Some(Self { manager, hotkey })
    }

    /// 热键 id，用于在事件循环中匹配。
    pub fn id(&self) -> u32 {
        self.hotkey.id()
    }

    /// 注销热键。
    pub fn unregister(&self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}

impl Drop for ShortcutManager {
    fn drop(&mut self) {
        self.unregister();
    }
}

/// 启动热键事件监听线程。
pub fn spawn_event_loop(tx: Sender<AppEvent>, hotkey_id: u32) {
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if event.id == hotkey_id {
                let _ = tx.send(AppEvent::Hotkey);
            }
        }
    });
}
