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
            &PredefinedMenuItem::about(app, None, None)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "app:settings",
                i18n.t("nav.settings"),
                true,
                Some("CmdOrCtrl+,"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            // 预定义项传 None，交给 macOS 用系统语言渲染（「隐藏 / 隐藏其他 /
            // 显示全部 / 退出」），不要自己写死中文。
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
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
