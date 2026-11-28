//! 全局快捷键。
//!
//! 注册放在 Tauri 的 `setup()` 里（主线程），以确保热键事件能被应用事件循环派发 ——
//! `GlobalHotKeyManager::new()` 本身并不检查线程，真正的约束是事件循环必须在主线程。
//!
//! 组合键按平台区分：macOS 用 ⌘+L；Windows 上 `Win+L` 是系统保留的「锁定屏幕」，
//! `RegisterHotKey` 会直接失败（`ERROR_HOTKEY_ALREADY_REGISTERED`），
//! 所以那边改成 Ctrl+Alt+L。

use std::sync::{Mutex, Once};

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{AppHandle, Runtime};

/// 热键管理器与已注册热键（全局唯一）。
///
/// 两者放同一个槽里，check-then-act 一次加锁完成，避免出现
/// 「管理器已写入、热键还没写入」的中间态。
///
/// `global-hotkey` 的 Windows 后端把窗口句柄（`HWND`，本质是 `*mut c_void`）
/// 塞进了 `GlobalHotKeyManager`，于是这个类型不是 `Send`；上游在 macOS 后端
/// 对同类的句柄直接写了 `unsafe impl Send / Sync`，Windows 后端只是漏了。
/// 句柄本身可以安全跨线程移动 —— 注册/注销走的是全局的 `RegisterHotKey` /
/// `UnregisterHotKey`，窗口消息仍由创建它的主线程消息循环派发，
/// 而我们始终在主线程创建与使用管理器。
struct HotKeySlot {
    manager: Option<GlobalHotKeyManager>,
    hotkey: Option<HotKey>,
}

// SAFETY: 槽位里只有 OS 句柄与热键描述，不含任何线程局部状态，
// 且句柄的注册/注销是全局操作。详见上面的说明。
unsafe impl Send for HotKeySlot {}

static SLOT: Mutex<HotKeySlot> = Mutex::new(HotKeySlot {
    manager: None,
    hotkey: None,
});
/// 监听线程的启动闸门（全局只允许拉起一次）。
static LISTENER: Once = Once::new();

/// 全局快捷键的组合键。
///
/// Windows 的 `Win+L` 是系统保留的「锁定屏幕」，注册必然失败，
/// 那边改用 Ctrl+Alt+L。
fn modifiers() -> Modifiers {
    #[cfg(target_os = "windows")]
    {
        Modifiers::CONTROL | Modifiers::ALT
    }
    #[cfg(not(target_os = "windows"))]
    {
        Modifiers::META
    }
}

/// 注册全局快捷键并启动监听线程。
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
        if slot.hotkey.is_some() {
            return Ok(());
        }
    }

    let manager =
        GlobalHotKeyManager::new().map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
    let hotkey = HotKey::new(Some(modifiers()), Code::KeyL);
    let id = hotkey.id();
    manager
        .register(hotkey)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;

    {
        let mut slot = SLOT.lock().expect("快捷键锁中毒");
        slot.manager = Some(manager);
        slot.hotkey = Some(hotkey);
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
            (slot.manager.take(), slot.hotkey.take())
        };
        if let (Some(hotkey), Some(manager)) = (hotkey, manager)
            && let Err(err) = manager.unregister(hotkey)
        {
            eprintln!("[shortcut] 注销 ⌘L 失败: {err}");
        }
    }
}
