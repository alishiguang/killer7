//! 全局快捷键 ⌘+L。
//!
//! macOS 上 `GlobalHotKeyManager` 必须在主线程创建，否则注册会失败，
//! 因此注册放在 Tauri 的 `setup()` 里（setup 运行在主线程）。

use std::sync::{Mutex, Once};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{AppHandle, Runtime};

/// 热键管理器（全局唯一）。
static MANAGER: Mutex<Option<GlobalHotKeyManager>> = Mutex::new(None);
/// 已注册的热键。
static HOTKEY: Mutex<Option<HotKey>> = Mutex::new(None);
/// 监听线程的启动闸门（全局只允许拉起一次）。
static LISTENER: Once = Once::new();

/// 注册 ⌘+L 并启动监听线程。
///
/// 幂等：已经注册过就直接返回成功。macOS 的 `RegisterEventHotKey` 对重复注册返回
/// `eventHotKeyExistsErr`（`global_hotkey::Error::AlreadyRegistered`），不挡住的话，
/// 每次「启用」都会再建一个管理器、并多拉起一个监听线程。
pub fn register<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if HOTKEY.lock().expect("快捷键锁中毒").is_some() {
        return Ok(());
    }

    let manager =
        GlobalHotKeyManager::new().map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
    let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyL);
    let id = hotkey.id();
    manager
        .register(hotkey)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;

    *MANAGER.lock().expect("快捷键锁中毒") = Some(manager);
    *HOTKEY.lock().expect("快捷键锁中毒") = Some(hotkey);

    LISTENER.call_once(|| {
        let handle = app.clone();
        std::thread::spawn(move || {
            let receiver = GlobalHotKeyEvent::receiver();
            while let Ok(event) = receiver.recv() {
                if event.id == id && event.state == global_hotkey::HotKeyState::Pressed {
                    crate::system::tray::show_main_window(&handle);
                }
            }
        });
    });

    Ok(())
}

/// 启用或停用快捷键。
pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, enabled: bool) {
    if enabled {
        let _ = register(app);
    } else {
        // 先把两者从静态槽里取出来，再做注销，避免持锁调用原生 API。
        let hotkey = HOTKEY.lock().expect("快捷键锁中毒").take();
        let manager = MANAGER.lock().expect("快捷键锁中毒").take();
        if let (Some(hotkey), Some(manager)) = (hotkey, manager) {
            let _ = manager.unregister(hotkey);
        }
    }
}
