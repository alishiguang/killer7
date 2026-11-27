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
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::dto::ContextTarget;
use crate::core::i18n::{I18n, Locale};
use crate::events::EV_MENU_ACTION;
use crate::events::EV_NOTICE;

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
                i18n.t("settings.title"),
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

    // 「设置」不再是视图菜单里的一项（前端把它挪到了工具栏角落的齿轮按钮），
    // 只由应用菜单的「设置…」触发，Cmd+, 加速键也只挂在那里 —— 同一个快捷键
    // 指向两个菜单项时 macOS 只能二选一。
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

/// 处理原生右键菜单项点击。
///
/// 右键菜单的目标对象在弹出时被暂存进 `AppState::pending_context`
/// （菜单事件本身只带 id，不带目标），这里取出来执行对应动作。
pub fn on_context_menu_event(app: &AppHandle, id: &str) {
    use crate::core::clipboard;
    use crate::core::platform;
    use crate::core::process::ProcessManager;

    let state = app.state::<std::sync::Arc<crate::state::AppState>>();
    let target = {
        let mut pending = state.pending_context.lock().expect("右键目标锁中毒");
        pending.take()
    };

    let Some(target) = target else {
        return;
    };

    let locale = state.i18n.lock().expect("文案锁中毒").locale();
    let i18n = I18n::new(locale);

    match (id, &target) {
        ("ctx.process.copyPath", ContextTarget::Process { full_path, .. }) => {
            platform::copy_to_clipboard(full_path);
            notify(app, &i18n, "info", &i18n.t("clipboard.copied"));
        }
        (
            "ctx.process.kill",
            ContextTarget::Process {
                pid, display_name, ..
            },
        ) => {
            if ProcessManager::kill_process(*pid) {
                notify(
                    app,
                    &i18n,
                    "info",
                    &i18n.tf("process.kill.done", &[display_name]),
                );
                // 立刻重采一次并把新列表推给前端，不要等下一个刷新周期。
                let rows = {
                    let limit = state
                        .settings
                        .lock()
                        .expect("设置锁中毒")
                        .process_list_limit;
                    let mut processes = state.processes.lock().expect("进程锁中毒");
                    // 必须用 `collect_and_store` 而不是 `collect`：
                    // `collect` 只更新端口缓存、不写 `processes`，
                    // 被杀掉的进程会继续留在缓存里 —— 底栏的进程总数
                    // 与 `list_processes` 的结果都会停留在杀之前的快照。
                    processes.collect_and_store(limit);
                    processes
                        .processes
                        .iter()
                        .map(crate::commands::dto::ProcessRow::from)
                        .collect::<Vec<_>>()
                };
                let _ = app.emit(crate::events::EV_PROCESSES, rows);
            } else {
                notify(app, &i18n, "error", &i18n.t("process.kill.failed"));
            }
        }
        ("ctx.file.reveal", ContextTarget::File { path, .. }) => {
            platform::reveal_in_finder(std::path::Path::new(path));
        }
        ("ctx.file.copyPath", ContextTarget::File { path, .. }) => {
            platform::copy_to_clipboard(path);
            notify(app, &i18n, "info", &i18n.t("clipboard.copied"));
        }
        ("ctx.file.rename", ContextTarget::File { .. }) => {
            let _ = app.emit(EV_MENU_ACTION, "file:rename");
        }
        ("ctx.file.delete", ContextTarget::File { .. }) => {
            let _ = app.emit(EV_MENU_ACTION, "file:delete");
        }
        ("ctx.clipboard.copy", ContextTarget::Clipboard { content, .. }) => {
            platform::copy_to_clipboard(content);
            notify(app, &i18n, "info", &i18n.t("clipboard.copied"));
        }
        ("ctx.clipboard.delete", ContextTarget::Clipboard { id, .. }) => {
            let changed = {
                let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
                manager.delete_history_item(*id)
            };
            // 落盘放在锁外：磁盘 IO 不该占着剪贴板锁。
            if changed {
                let result = clipboard::persist_history(|| {
                    state
                        .clipboard
                        .lock()
                        .expect("剪贴板锁中毒")
                        .history
                        .clone()
                });
                if let Err(err) = result {
                    eprintln!("[clipboard] 落盘失败: {err}");
                }
            }
            notify(app, &i18n, "info", &i18n.t("clipboard.deleted"));
        }
        _ => {}
    }
}

/// 向前端回传提示。
fn notify(app: &AppHandle, _i18n: &I18n, kind: &str, message: &str) {
    let _ = app.emit(
        EV_NOTICE,
        crate::commands::dto::Notice {
            kind: kind.to_string(),
            message: message.to_string(),
        },
    );
}
