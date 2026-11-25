//! Dock / 菜单栏模式切换。
//!
//! Tauri v2 没有暴露 `NSApp.activationPolicy`，这里用 `objc2-app-kit` 直调 AppKit。
//! 必须在主线程调用；失败时降级为「仅隐藏窗口」，行为差异可接受。

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use tauri::{AppHandle, Runtime};

/// 设置激活策略：`true` 为仅菜单栏（不占 Dock），`false` 为常规（占 Dock）。
pub fn set_accessory_mode<R: Runtime>(enabled: bool, _app: &AppHandle<R>) {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("[activation] 不在主线程，跳过激活策略切换");
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let policy = if enabled {
        NSApplicationActivationPolicy::Accessory
    } else {
        NSApplicationActivationPolicy::Regular
    };

    if !app.setActivationPolicy(policy) {
        eprintln!("[activation] 切换激活策略失败，降级为仅隐藏窗口");
    }
}
