//! 设置与文案命令。

use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::core::i18n::{I18n, Locale};
use crate::core::settings::{Settings, SettingsPatch};
use crate::state::AppState;

/// 读取当前设置。
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    let settings = state.settings.lock().expect("设置锁中毒");
    settings.clone()
}

/// 提交设置补丁，返回合并后的完整设置。
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    patch: SettingsPatch,
) -> Settings {
    // 先取旧值：菜单栏模式与全局快捷键要靠「改了什么」判断是否重新作用到原生层。
    // 若在写入之后再取旧值，拿到的就是新值，比较永远相等，热更新就成了死代码。
    let previous = state.settings.lock().expect("设置锁中毒").clone();

    let mut next = patch.apply(&previous);

    // 自启开关必须落到 LaunchAgent 上，否则会出现「设置里显示已开启、实际没注册」
    // 与「Agent 建好了、开关却显示关闭」两个互不同步的真相源（Task 15 评审 I-1）。
    // 失败就回滚这一个字段，不让设置与现实脱节。
    if previous.launch_at_login != next.launch_at_login {
        let outcome = if next.launch_at_login {
            crate::system::autolaunch::enable()
        } else {
            crate::system::autolaunch::disable()
        };
        if let Err(err) = outcome {
            eprintln!("[autolaunch] 应用开机自启设置失败，回滚该字段: {err}");
            next.launch_at_login = previous.launch_at_login;
        }
    }

    // 写回全局状态：锁内只改内存。
    {
        let mut settings = state.settings.lock().expect("设置锁中毒");
        *settings = next.clone();
    }

    // 落盘放在锁外：`next` 是独立副本，写盘不占设置锁。
    next.save();

    // 语言变了要换文案表。
    if previous.locale != next.locale {
        {
            let mut i18n = state.i18n.lock().expect("文案锁中毒");
            i18n.set_locale(next.locale);
        }
        // 原生菜单栏与托盘菜单也要跟着换文案，否则界面是英文、菜单栏还是中文 ——
        // 这正是历史提交 f5fc259「消除半中半英」要消灭的现象（Task 15 评审 I-2）。
        // `menu::build` 结尾是 `set_as_app_menu()`，是替换语义，可以安全重复调用。
        if let Err(err) = crate::system::menu::build(&app, next.locale) {
            eprintln!("[menu] 重建菜单栏失败: {err}");
        }
        crate::system::tray::update_locale(&app, next.locale);
    }

    // 采样间隔与网络开关热更新。
    {
        state.sample_interval_ms.store(
            (next.auto_refresh_interval * 1000.0) as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        state.network_enabled.store(
            next.enable_network_monitor,
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    // 菜单栏模式与全局快捷键的变化需要立即作用到原生层。
    if previous.show_menu_bar != next.show_menu_bar {
        crate::system::activation::apply_menu_bar_mode(next.show_menu_bar, &app);
    }

    if previous.global_shortcut_enabled != next.global_shortcut_enabled {
        crate::system::shortcut::set_enabled(&app, next.global_shortcut_enabled);
    }

    next
}

/// 取指定语言的整张文案表。
#[tauri::command]
pub fn get_strings(locale: String) -> HashMap<String, String> {
    // 与 `Locale::detect()` 同一口径：按前缀判定。
    // 否则前端传 "en-US" 会回落到中文，而系统语言探测会判成英文（Task 15 评审 M-2）。
    let parsed = if locale.trim_start().to_ascii_lowercase().starts_with("en") {
        Locale::En
    } else {
        Locale::ZhHans
    };
    I18n::new(parsed).table()
}
