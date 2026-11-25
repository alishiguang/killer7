//! 原生应用菜单栏。
//!
//! 使用 Tauri 的 `Menu::set_as_app_menu()` 构建真正的 macOS 菜单栏：
//! 关于、设置、隐藏、退出走 `PredefinedMenuItem`（系统原生行为），
//! 视图切换走带 `Cmd+数字` 加速器的自定义项。
//!
//! 菜单文案全部走 `core::i18n`。muda 0.20 在 `text=None` 时用的是它自己写死的
//! 英文表（`muda-0.20.0/src/items/predefined.rs:305-308,434-487`），而 macOS 后端
//! 用 `initWithTitle:` 显式设标题，AppKit 不再本地化 —— 实测本机 AppleLocale=zh_CN，
//! 菜单仍然显示英文的 Hide / Hide Others / Show All / Undo / Cut / Minimize。
//! 因此**每一个**预定义项都必须显式传文案，`locales/*.strings` 里为此备了
//! `menubar.*` / `edit.*` / `window.*` / `view.fullscreen` 等 key。
//!
//! 「关于 / 隐藏 / 退出」三项按 macOS 惯例带应用名后缀，在 Rust 侧拼成
//! `<动词> chuaker7`；应用名本身取自 `menubar.title`（= "chuaker7"）。

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter};

use crate::core::i18n::{I18n, Locale};
use crate::events::EV_MENU_ACTION;

/// 构建并安装应用菜单栏。
pub fn build(app: &AppHandle, locale: Locale) -> tauri::Result<()> {
    let i18n = I18n::new(locale);

    // 应用名（"chuaker7"），用于拼「关于 / 隐藏 / 退出」三项。
    let app_name = i18n.t("menubar.title");
    let about_text = format!("{} {}", i18n.t("settings.about"), app_name);
    let hide_text = format!("{} {}", i18n.t("menubar.hide"), app_name);
    let quit_text = format!("{} {}", i18n.t("menubar.quitApp"), app_name);

    let app_menu = Submenu::with_items(
        app,
        i18n.t("menubar.title"),
        true,
        &[
            &PredefinedMenuItem::about(app, Some(about_text.as_str()), None)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "app:settings",
                i18n.t("nav.settings"),
                true,
                Some("CmdOrCtrl+,"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some(hide_text.as_str()))?,
            &PredefinedMenuItem::hide_others(app, Some(&i18n.t("menubar.hideOthers")))?,
            &PredefinedMenuItem::show_all(app, Some(&i18n.t("menubar.showAll")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some(quit_text.as_str()))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        i18n.t("menubar.edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, Some(&i18n.t("edit.undo")))?,
            &PredefinedMenuItem::redo(app, Some(&i18n.t("edit.redo")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, Some(&i18n.t("edit.cut")))?,
            &PredefinedMenuItem::copy(app, Some(&i18n.t("edit.copy")))?,
            &PredefinedMenuItem::paste(app, Some(&i18n.t("edit.paste")))?,
            &PredefinedMenuItem::select_all(app, Some(&i18n.t("edit.selectAll")))?,
        ],
    )?;

    // 「设置」的 Cmd+, 加速键只留给应用菜单里的那一项；视图菜单里再挂一次
    // 会让同一个快捷键指向两个菜单项，macOS 只能二选一。
    let view_menu = Submenu::with_items(
        app,
        i18n.t("menubar.view"),
        true,
        &[
            &MenuItem::with_id(
                app,
                "nav:processes",
                i18n.t("nav.process"),
                true,
                Some("CmdOrCtrl+1"),
            )?,
            &MenuItem::with_id(
                app,
                "nav:files",
                i18n.t("nav.file"),
                true,
                Some("CmdOrCtrl+2"),
            )?,
            &MenuItem::with_id(
                app,
                "nav:clipboard",
                i18n.t("nav.clipboard"),
                true,
                Some("CmdOrCtrl+3"),
            )?,
            &MenuItem::with_id(
                app,
                "nav:monitor",
                i18n.t("nav.monitor"),
                true,
                Some("CmdOrCtrl+4"),
            )?,
            &MenuItem::with_id(
                app,
                "nav:plugins",
                i18n.t("nav.plugins"),
                true,
                Some("CmdOrCtrl+5"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "nav:settings",
                i18n.t("nav.settings"),
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::fullscreen(app, Some(&i18n.t("view.fullscreen")))?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        i18n.t("menubar.window"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some(&i18n.t("window.minimize")))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, Some(&i18n.t("window.close")))?,
        ],
    )?;

    let menu = Menu::with_items(app, &[&app_menu, &edit_menu, &view_menu, &window_menu])?;

    menu.set_as_app_menu()?;
    Ok(())
}

/// 处理菜单项点击：把自定义动作转成前端事件。
pub fn on_menu_event(app: &AppHandle, id: &str) {
    if id.starts_with("nav:") || id == "app:settings" {
        let _ = app.emit(EV_MENU_ACTION, id);
    }
}
