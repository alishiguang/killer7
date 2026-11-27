//! 系统级命令：窗口显隐、菜单栏模式、快捷键、自启、原生右键菜单。

use std::sync::Arc;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Manager, State};

use crate::commands::dto::ContextTarget;
use crate::core::i18n::I18n;
use crate::state::AppState;

/// 隐藏主窗口。
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// 显示主窗口并置前。
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// 启用或停用开机自启。
///
/// 设置项 `launch_at_login` 与 LaunchAgent 的真实状态必须是**同一个真相源**：
/// 只改设置项会出现「显示已开启、实际没注册」，只注册 LaunchAgent 会出现
/// 「开关显示关闭，用户再点一次就把刚建好的 Agent 关掉」（Task 15 评审 I-1）。
/// 所以这里在注册成功后把设置项一起对齐并落盘。
#[tauri::command]
pub fn set_auto_launch(state: State<'_, Arc<AppState>>, enabled: bool) -> Result<(), String> {
    if enabled {
        crate::system::autolaunch::enable().map_err(|err| err.to_string())?;
    } else {
        crate::system::autolaunch::disable().map_err(|err| err.to_string())?;
    }

    let snapshot = {
        let mut settings = state.settings.lock().expect("设置锁中毒");
        settings.launch_at_login = enabled;
        settings.clone()
    };
    snapshot.save();

    Ok(())
}

/// 弹出原生右键菜单。
///
/// 菜单项点击由 `system::menu::on_context_menu_event` 统一处理，
/// 目标对象暂存在 `AppState::pending_context`。
#[tauri::command]
pub fn popup_context_menu(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    kind: String,
    payload: serde_json::Value,
) {
    let raw = serde_json::json!({ "kind": kind, "payload": payload });
    let Ok(target) = serde_json::from_value::<ContextTarget>(raw) else {
        eprintln!("[menu] 右键目标解析失败，忽略");
        return;
    };

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let locale = state.i18n.lock().expect("文案锁中毒").locale();
    let i18n = I18n::new(locale);

    let menu = match build_context_menu(&app, &target, &i18n) {
        Ok(menu) => menu,
        Err(err) => {
            eprintln!("[menu] 构建右键菜单失败: {err}");
            return;
        }
    };

    {
        *state.pending_context.lock().expect("右键目标锁中毒") = Some(target);
    }

    let _ = window.popup_menu(&menu);
}

/// 构建与目标匹配的原生菜单。
///
/// `Menu::with_items` 要求 `&[&dyn IsMenuItem<R>]` 切片，同类型项才能放进同一个数组；
/// 这里改用 `Menu::new` + `append`，可以混合 `MenuItem` 与 `PredefinedMenuItem`。
///
/// 菜单文案全部走 `core::i18n` —— muda 0.20 在 `text=None` 时会用它写死的英文表，
/// 而 macOS 后端用 `initWithTitle:` 显式设标题，**预定义项永远不会跟随系统语言**。
fn build_context_menu(
    app: &AppHandle,
    target: &ContextTarget,
    i18n: &I18n,
) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;

    match target {
        ContextTarget::Process {
            pid, display_name, ..
        } => {
            let pid_text = pid.to_string();
            let kill_label = i18n.tf("process.context.kill", &[display_name, &pid_text]);
            menu.append(&MenuItem::with_id(
                app,
                "ctx.process.copyPath",
                i18n.t("process.context.copyPath"),
                true,
                None::<&str>,
            )?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.process.kill",
                kill_label,
                true,
                None::<&str>,
            )?)?;
        }
        ContextTarget::File { .. } => {
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.reveal",
                i18n.t("file.reveal"),
                true,
                None::<&str>,
            )?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.copyPath",
                i18n.t("file.copyPath"),
                true,
                None::<&str>,
            )?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.rename",
                i18n.t("file.rename"),
                true,
                None::<&str>,
            )?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.delete",
                i18n.t("file.delete"),
                true,
                None::<&str>,
            )?)?;
        }
        ContextTarget::Clipboard { .. } => {
            menu.append(&MenuItem::with_id(
                app,
                "ctx.clipboard.copy",
                i18n.t("clipboard.copy"),
                true,
                None::<&str>,
            )?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.clipboard.delete",
                i18n.t("clipboard.delete"),
                true,
                None::<&str>,
            )?)?;
        }
    }

    Ok(menu)
}
