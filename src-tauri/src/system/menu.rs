//! 原生应用菜单栏。
//!
//! 使用 Tauri 的 `Menu::set_as_app_menu()` 构建真正的 macOS 菜单栏：
//! 关于、设置、隐藏、退出走 `PredefinedMenuItem`（系统原生行为），
//! 视图切换走带 `Cmd+数字` 加速器的自定义项。
//!
//! 菜单文案走 `core::i18n`：`nav.*` / `menubar.*` 都有现成的 key。
//! 「编辑 / 视图 / 窗口」三个下拉的标题**没有**对应 key，而 `locales/*.strings`
//! 约定「不新增 key」，因此这三个标题保持硬编码中文。

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter};

use crate::core::i18n::{I18n, Locale};
use crate::events::EV_MENU_ACTION;

/// 构建并安装应用菜单栏。
pub fn build(app: &AppHandle, locale: Locale) -> tauri::Result<()> {
    let i18n = I18n::new(locale);

    let app_menu = Submenu::with_items(
        app,
        i18n.t("menubar.title"),
        true,
        &[
            &PredefinedMenuItem::about(app, Some("chuaker7"), None)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "app:settings",
                i18n.t("nav.settings"),
                true,
                Some("CmdOrCtrl+,"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            // 这三项**不能**传 None：muda 0.20.0 在 text=None 时用的是它自己写死的
            // 英文表（`muda-0.20.0/src/items/predefined.rs:305-308,434-487`），而 macOS
            // 后端用 `initWithTitle:` 显式设标题，AppKit 不再本地化 —— 实测本机
            // AppleLocale=zh_CN，菜单仍然显示英文的 Hide / Hide Others / Show All。
            // `locales/*.strings` 与 Swift 版共用、约定不新增 key，而这三项没有对应
            // key，所以只能写死中文（应用以简体中文为主）；英文版会在这三处不一致，
            // 属已知取舍。退出项有 `menubar.quit` key，走 i18n。
            &PredefinedMenuItem::hide(app, Some("隐藏 chuaker7"))?,
            &PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?,
            &PredefinedMenuItem::show_all(app, Some("显示全部"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some(&i18n.t("menubar.quit")))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "编辑",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    // 「设置」的 Cmd+, 加速键只留给应用菜单里的那一项；视图菜单里再挂一次
    // 会让同一个快捷键指向两个菜单项，macOS 只能二选一。
    let view_menu = Submenu::with_items(
        app,
        "视图",
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
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        "窗口",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
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
