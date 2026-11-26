//! 托盘图标与托盘菜单。
//!
//! 图标沿用 egui 版的「小鸟」像素画（`core::icon::bird_rgba`），
//! 以 macOS 模板图方式挂载，系统自动适配深浅色。

use std::sync::Mutex;

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::core::i18n::{I18n, Locale};
use crate::core::icon::bird_rgba;
use crate::events::{EV_TRAY_OPEN, EV_TRAY_QUIT};

/// 托盘图标的 id。
const TRAY_ID: &str = "main";

/// 托盘里需要动态刷新的菜单项。
struct TrayItems<R: Runtime> {
    cpu: MenuItem<R>,
    memory: MenuItem<R>,
    disk: MenuItem<R>,
    upload: MenuItem<R>,
    download: MenuItem<R>,
    /// 托盘标题项（应用名）。
    title: MenuItem<R>,
    /// 「打开」项。
    open: MenuItem<R>,
    /// 「退出」项。
    quit: MenuItem<R>,
    /// 本地化后的指标名称，刷新时拼在数值前面。
    labels: Mutex<TrayLabels>,
}

/// 指标项的本地化前缀。
struct TrayLabels {
    cpu: String,
    memory: String,
    disk: String,
    upload: String,
    download: String,
}

/// 构建托盘图标与菜单。
pub fn build<R: Runtime>(app: &AppHandle<R>, locale: Locale) -> tauri::Result<()> {
    let (pixels, width, height) = bird_rgba(22);
    let icon = Image::new_owned(pixels, width, height);

    let i18n = I18n::new(locale);
    // 应用名（展示名 "Ki6"），托盘标题项与 tooltip 共用。
    let app_name = i18n.t("menubar.title");
    let cpu_label = i18n.t("monitor.cpu");
    let memory_label = i18n.t("monitor.memory");
    let disk_label = i18n.t("monitor.disk");
    let upload_label = i18n.t("monitor.upload");
    let download_label = i18n.t("monitor.download");

    let title = MenuItem::with_id(app, "tray:title", app_name.clone(), false, None::<&str>)?;
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
    let open = MenuItem::with_id(app, "tray:open", i18n.t("menubar.open"), true, None::<&str>)?;
    // 用**自定义**项而不是 `PredefinedMenuItem::quit`：预定义项由系统直接
    // 退出应用，走不到下面的 `on_tray_menu_event`，`tray-quit` 事件就永远
    // 发不出去（契约里写了托盘「退出」要发这个事件）。
    let quit = MenuItem::with_id(app, "tray:quit", i18n.t("menubar.quit"), true, None::<&str>)?;

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
            &open,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .tooltip(&app_name)
        .show_menu_on_left_click(false)
        .menu(&menu)
        // Tauri 2.12.7 的 `TrayIconBuilder::on_menu_event` **同样是全局监听器**：
        // 它与 `Builder::on_menu_event` 共用 `manager.menu.global_event_listeners`，
        // 会收到应用菜单、窗口菜单与所有托盘菜单的事件（Tauri 官方文档明写
        // "This handler is not specific to this tray icon"）。真正把范围限定在
        // 托盘菜单的，靠的是下面 `on_tray_menu_event` 里按 id 的 `match` 门控。
        // **不要把未知 id 转发给 `system::menu::on_menu_event`** —— 那会让每一次
        // 应用菜单点击都被处理两次（Tauri 文档警告的重复处理）。
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
        title,
        open,
        quit,
        labels: Mutex::new(TrayLabels {
            cpu: cpu_label,
            memory: memory_label,
            disk: disk_label,
            upload: upload_label,
            download: download_label,
        }),
    });

    Ok(())
}

/// 换语言后更新托盘菜单的文案。
///
/// 托盘菜单的**结构**在两种语言下完全一致（同样的项、同样的顺序），
/// 只有文案不同，所以这里不重建菜单、只改各菜单项的文本。
///
/// 也不能走 `build()` 重来一遍：那会 `app.manage(TrayItems{..})` 第二次托管
/// 同类型状态（`manage` 对已托管类型是静默 no-op），并且 `TrayIconBuilder`
/// 不检查重复 id，会多出一个托盘图标。
pub fn update_locale<R: Runtime>(app: &AppHandle<R>, locale: Locale) {
    let Some(items) = app.try_state::<TrayItems<R>>() else {
        return;
    };

    let i18n = I18n::new(locale);
    let _ = items.title.set_text(i18n.t("menubar.title"));
    let _ = items.open.set_text(i18n.t("menubar.open"));
    let _ = items.quit.set_text(i18n.t("menubar.quit"));

    let mut labels = items
        .labels
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    labels.cpu = i18n.t("monitor.cpu");
    labels.memory = i18n.t("monitor.memory");
    labels.disk = i18n.t("monitor.disk");
    labels.upload = i18n.t("monitor.upload");
    labels.download = i18n.t("monitor.download");

    // 指标项的前缀换掉，数值留到下一次采样刷新（约 1 秒内）。
    let _ = items.cpu.set_text(format!("{}  —", labels.cpu));
    let _ = items.memory.set_text(format!("{}  —", labels.memory));
    let _ = items.disk.set_text(format!("{}  —", labels.disk));
    let _ = items.upload.set_text(format!("{}  —", labels.upload));
    let _ = items.download.set_text(format!("{}  —", labels.download));
}

/// 更新托盘里的实时指标。
pub fn update_metrics<R: Runtime>(app: &AppHandle<R>, info: &crate::core::monitor::SystemInfo) {
    let Some(items) = app.try_state::<TrayItems<R>>() else {
        return;
    };
    let labels = items
        .labels
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _ = items
        .cpu
        .set_text(format!("{}  {:.1}%", labels.cpu, info.cpu_usage));
    let _ = items
        .memory
        .set_text(format!("{}  {:.1}%", labels.memory, info.memory_usage));
    let _ = items
        .disk
        .set_text(format!("{}  {:.1}%", labels.disk, info.disk_usage));
    let _ = items.upload.set_text(format!(
        "{}  {:.2} MB/s",
        labels.upload, info.network_upload
    ));
    let _ = items.download.set_text(format!(
        "{}  {:.2} MB/s",
        labels.download, info.network_download
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
