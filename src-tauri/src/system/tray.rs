//! 托盘图标与托盘菜单。
//!
//! 图标沿用 egui 版的「小鸟」像素画（`core::icon::bird_rgba`），
//! 以 macOS 模板图方式挂载，系统自动适配深浅色。

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::core::i18n::{I18n, Locale};
use crate::core::icon::bird_rgba;
use crate::events::{EV_TRAY_OPEN, EV_TRAY_QUIT};

/// 托盘里需要动态刷新的菜单项。
struct TrayItems<R: Runtime> {
    cpu: MenuItem<R>,
    memory: MenuItem<R>,
    disk: MenuItem<R>,
    upload: MenuItem<R>,
    download: MenuItem<R>,
    /// 本地化后的指标名称，刷新时拼在数值前面。
    cpu_label: String,
    memory_label: String,
    disk_label: String,
    upload_label: String,
    download_label: String,
}

/// 构建托盘图标与菜单。
pub fn build<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<()> {
    let (pixels, width, height) = bird_rgba(22);
    let icon = Image::new_owned(pixels, width, height);

    let i18n = I18n::new(locale);
    let cpu_label = i18n.t("monitor.cpu");
    let memory_label = i18n.t("monitor.memory");
    let disk_label = i18n.t("monitor.disk");
    let upload_label = i18n.t("monitor.upload");
    let download_label = i18n.t("monitor.download");

    let title = MenuItem::with_id(
        app,
        "tray:title",
        i18n.t("menubar.title"),
        false,
        None::<&str>,
    )?;
    let cpu = MenuItem::with_id(
        app,
        "tray:cpu",
        format!("{cpu_label}  —"),
        false,
        None::<&str>,
    )?;
    let memory = MenuItem::with_id(
        app,
        "tray:memory",
        format!("{memory_label}  —"),
        false,
        None::<&str>,
    )?;
    let disk = MenuItem::with_id(
        app,
        "tray:disk",
        format!("{disk_label}  —"),
        false,
        None::<&str>,
    )?;
    let upload = MenuItem::with_id(
        app,
        "tray:upload",
        format!("{upload_label}  —"),
        false,
        None::<&str>,
    )?;
    let download = MenuItem::with_id(
        app,
        "tray:download",
        format!("{download_label}  —"),
        false,
        None::<&str>,
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &title,
            &cpu,
            &memory,
            &disk,
            &upload,
            &download,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "tray:open", i18n.t("menubar.open"), true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            // 用**自定义**项而不是 `PredefinedMenuItem::quit`：预定义项由系统直接
            // 退出应用，走不到下面的 `on_tray_menu_event`，`tray-quit` 事件就永远
            // 发不出去（契约里写了托盘「退出」要发这个事件）。
            &MenuItem::with_id(app, "tray:quit", i18n.t("menubar.quit"), true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("chuaker7")
        .show_menu_on_left_click(false)
        .menu(&menu)
        // 托盘菜单的点击必须自己接：`Builder::on_menu_event` 是**全局**监听，
        // 会和应用菜单的监听一起触发；而这里要的是「只认托盘菜单里的项」。
        .on_menu_event(|app, event| {
            on_tray_menu_event(app, event.id().as_ref());
        })
        .build(app)?;

    // 把可更新的菜单项挂到全局状态上，供采样循环更新。
    app.manage(TrayItems {
        cpu,
        memory,
        disk,
        upload,
        download,
        cpu_label,
        memory_label,
        disk_label,
        upload_label,
        download_label,
    });

    Ok(())
}

/// 更新托盘里的实时指标。
pub fn update_metrics<R: Runtime>(app: &AppHandle<R>, info: &crate::core::monitor::SystemInfo) {
    let Some(items) = app.try_state::<TrayItems<R>>() else {
        return;
    };
    let _ = items
        .cpu
        .set_text(format!("{}  {:.1}%", items.cpu_label, info.cpu_usage));
    let _ = items
        .memory
        .set_text(format!("{}  {:.1}%", items.memory_label, info.memory_usage));
    let _ = items
        .disk
        .set_text(format!("{}  {:.1}%", items.disk_label, info.disk_usage));
    let _ = items.upload.set_text(format!(
        "{}  {:.2} MB/s",
        items.upload_label, info.network_upload
    ));
    let _ = items.download.set_text(format!(
        "{}  {:.2} MB/s",
        items.download_label, info.network_download
    ));
}

/// 处理托盘点击与菜单点击。
pub fn on_tray_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_main_window(app);
        let _ = app.emit(EV_TRAY_OPEN, ());
    }
}

/// 处理托盘菜单项点击。
pub fn on_tray_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "tray:open" => {
            show_main_window(app);
            let _ = app.emit(EV_TRAY_OPEN, ());
        }
        "tray:quit" => {
            let _ = app.emit(EV_TRAY_QUIT, ());
            app.exit(0);
        }
        _ => {}
    }
}

/// 显示并置前主窗口。
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
