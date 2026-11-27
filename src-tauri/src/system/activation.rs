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

/// 应用「菜单栏模式」：切换 Dock 占用 + 主窗口显隐。
///
/// 这是菜单栏模式的**唯一入口**，启动路径（`lib.rs`）与设置热更新路径
/// （`commands::settings::update_settings`）都必须走这里。
///
/// 以前两条路径各写一半：启动时会隐藏主窗口，运行期切换却只改激活策略，
/// 于是「勾上开关窗口还留在桌面上、重启后窗口又不见了」——
/// 同一个设置项在运行中与重启后表现相反。
///
/// 语义（README「Dock / 菜单栏模式」）：
/// - `true`：不占 Dock 位置，主窗口隐藏，应用只在菜单栏常驻；
/// - `false`：恢复常规 Dock 模式，主窗口显示。
///
/// 同样**必须在主线程调用**（`MainThreadMarker::new()`）。
pub fn apply_menu_bar_mode<R: Runtime>(enabled: bool, app: &AppHandle<R>) {
    set_accessory_mode(enabled, app);

    // 激活策略切换失败时 `set_accessory_mode` 已经做过降级（隐藏窗口）；
    // 这里仍按目标状态统一处理显隐，保证「不占 Dock ⇒ 窗口不在桌面上」。
    if let Some(window) = app.get_webview_window("main") {
        if enabled {
            let _ = window.hide();
        } else {
            let _ = window.show();
        }
    }
}

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

#[cfg(test)]
mod tests {
    /// 菜单栏模式的**所有**路径必须走 `apply_menu_bar_mode`。
    ///
    /// 这条不变量曾经被破坏过：启动路径自己藏窗口，运行期热更新只切激活策略，
    /// 两者行为相反。单元测试没法拉起 `AppHandle` 去观察窗口显隐，
    /// 所以用源码级断言守住「只有一条入口」这条结构性约束
    /// （与 `autolaunch::enable` 不许调 `launchctl load` 的守门方式一致）。
    #[test]
    fn menu_bar_mode_has_a_single_entry_point() {
        let settings_cmd = include_str!("../commands/settings.rs");
        assert!(
            settings_cmd.contains("apply_menu_bar_mode"),
            "设置热更新必须走 activation::apply_menu_bar_mode"
        );
        assert!(
            !settings_cmd.contains("set_accessory_mode"),
            "设置热更新不得绕过 apply_menu_bar_mode 直接切激活策略"
        );

        let lib = include_str!("../lib.rs");
        assert!(
            lib.contains("apply_menu_bar_mode"),
            "启动路径必须走 activation::apply_menu_bar_mode"
        );
        assert!(
            !lib.contains("set_accessory_mode"),
            "启动路径不得绕过 apply_menu_bar_mode 直接切激活策略"
        );
    }
}
