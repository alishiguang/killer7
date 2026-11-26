//! 全局快捷键 ⌘+L。
//!
//! 注册放在 Tauri 的 `setup()` 里（主线程），以确保 Carbon 事件能被应用事件循环派发 ——
//! `GlobalHotKeyManager::new()` 本身并不检查线程，真正的约束是事件循环必须在主线程。

use std::sync::{Mutex, Once};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{AppHandle, Runtime};

/// 热键管理器与已注册热键（全局唯一）。
///
/// 两者放同一个槽里，check-then-act 一次加锁完成，避免出现
/// 「管理器已写入、热键还没写入」的中间态。
static SLOT: Mutex<(Option<GlobalHotKeyManager>, Option<HotKey>)> = Mutex::new((None, None));
/// 监听线程的启动闸门（全局只允许拉起一次）。
static LISTENER: Once = Once::new();

/// 注册 ⌘+L 并启动监听线程。
///
/// 幂等：已经注册过就直接返回成功。macOS 的 `RegisterEventHotKey` 对重复注册返回
/// `eventHotKeyExistsErr`（`global_hotkey::Error::AlreadyRegistered`），不挡住的话，
/// 每次「启用」都会再建一个管理器、并多拉起一个监听线程。
///
/// 注意：macOS 用 `kEventHotKeyExclusive` 注册，**Ki6 运行期间会独占 ⌘L**，
/// 其他应用（浏览器聚焦地址栏、Finder 前往文件夹…）收不到这个键。
/// 这是全局热键注册方式本身的语义，不是实现缺陷。
pub fn register<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    {
        let slot = SLOT.lock().expect("快捷键锁中毒");
        if slot.1.is_some() {
            return Ok(());
        }
    }

    let manager =
        GlobalHotKeyManager::new().map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
    let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyL);
    let id = hotkey.id();
    manager
        .register(hotkey)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;

    {
        let mut slot = SLOT.lock().expect("快捷键锁中毒");
        *slot = (Some(manager), Some(hotkey));
    }

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
        // 注册失败必须留下痕迹：否则「设置里显示已开启、但 ⌘L 毫无反应」，
        // 用户与开发者都拿不到任何线索（典型触发场景：开了第二个实例，
        // 或别的应用已经持有 ⌘L）。
        if let Err(err) = register(app) {
            eprintln!("[shortcut] 注册 ⌘L 失败: {err}");
        }
    } else {
        // 先把两者从静态槽里取出来，再做注销，避免持锁调用原生 API。
        let (manager, hotkey) = {
            let mut slot = SLOT.lock().expect("快捷键锁中毒");
            (slot.0.take(), slot.1.take())
        };
        if let (Some(hotkey), Some(manager)) = (hotkey, manager)
            && let Err(err) = manager.unregister(hotkey)
        {
            eprintln!("[shortcut] 注销 ⌘L 失败: {err}");
        }
    }
}
