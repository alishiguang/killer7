//! Dock / 菜单栏模式切换。
//!
//! Tauri v2 没有暴露 `NSApp.activationPolicy`，这里用 `objc2-app-kit` 直调 AppKit。
//!
//! **必须在主线程调用**：`MainThreadMarker::new()` 在非主线程返回 `None`，此时直接跳过。
//! 当前的调用方都是同步的 Tauri command（Tauri v2 里同步 command 跑在主线程），
//! 但**不要**给这些 command 加 `async` 或 `#[tauri::command(async)]` ——
//! 那会把命令体切到 async runtime 线程上，菜单栏模式会静默失效。

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use tauri::{AppHandle, Manager, Runtime};

/// 设置激活策略：`true` 为仅菜单栏（不占 Dock），`false` 为常规（占 Dock）。
///
/// 切换失败时**真正执行**降级：隐藏主窗口，让应用退回「只在菜单栏可见」的效果。
pub fn set_accessory_mode<R: Runtime>(enabled: bool, app: &AppHandle<R>) {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("[activation] 不在主线程，跳过激活策略切换");
        return;
    };

    let ns_app = NSApplication::sharedApplication(mtm);
    let policy = if enabled {
        NSApplicationActivationPolicy::Accessory
    } else {
        NSApplicationActivationPolicy::Regular
    };

    if !ns_app.setActivationPolicy(policy) {
        eprintln!("[activation] 切换激活策略失败，降级为仅隐藏窗口");
        // 降级只在「切到仅菜单栏」失败时才有意义 —— 此时至少把窗口藏起来，
        // 不让它继续以普通窗口的方式占着 Dock 位置。
        if enabled && let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
}
