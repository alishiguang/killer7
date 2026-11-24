//! 菜单栏（系统托盘）管理，对应 Swift 版的 `MenuBarManager.swift`。
//!
//! Swift 版用 `NSStatusItem` + `NSPopover` 展示快捷系统信息；Rust 版用 `tray-icon`
//! 提供菜单栏图标，左键打开主窗口、右键弹出带实时指标的菜单。

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::events::AppEvent;
use crate::monitor::SystemInfo;

/// 「打开主窗口」菜单项 id。
const MENU_OPEN: &str = "killer7.open";
/// 「退出」菜单项 id。
const MENU_QUIT: &str = "killer7.quit";

/// 菜单栏图标。
pub struct MenuBar {
    /// 底层托盘图标。
    icon: TrayIcon,
    /// 共享的实时系统指标，供菜单展示。
    info: Arc<Mutex<SystemInfo>>,
    /// 菜单文案。
    labels: MenuLabels,
}

/// 菜单栏使用的文案。
pub struct MenuLabels {
    /// 应用标题。
    pub title: String,
    /// 「打开主窗口」。
    pub open: String,
    /// 「退出」。
    pub quit: String,
    /// CPU 标签。
    pub cpu: String,
    /// 内存标签。
    pub memory: String,
    /// 磁盘标签。
    pub disk: String,
    /// 上传标签。
    pub upload: String,
    /// 下载标签。
    pub download: String,
}

impl MenuBar {
    /// 创建菜单栏图标。
    pub fn new(labels: MenuLabels) -> Option<Self> {
        let info = Arc::new(Mutex::new(SystemInfo::default()));
        let menu = build_menu(&labels, &info);

        let (pixels, width, height) = crate::icon::bird_rgba(22);
        let icon_image = tray_icon::Icon::from_rgba(pixels, width, height).ok()?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(labels.title.clone())
            .with_icon_templated(icon_image)
            .with_menu_on_left_click(false)
            .build()
            .ok()?;

        Some(Self {
            icon: tray,
            info,
            labels,
        })
    }

    /// 更新菜单栏展示的实时指标。
    pub fn update_info(&self, info: SystemInfo) {
        if let Ok(mut guard) = self.info.lock() {
            *guard = info;
        }
    }

    /// 重新构建菜单，让展示的数值保持最新。
    pub fn rebuild_menu(&self) {
        let menu = build_menu(&self.labels, &self.info);
        self.icon.set_menu(Some(Box::new(menu)));
    }

    /// 移除菜单栏图标。
    pub fn shutdown(&self) {
        self.icon.set_visible(false).ok();
    }
}

/// 构建托盘菜单。
fn build_menu(labels: &MenuLabels, info: &Arc<Mutex<SystemInfo>>) -> Menu {
    let menu = Menu::new();

    let title = MenuItem::new(labels.title.clone(), false, None);
    let _ = menu.append(&title);

    let snapshot = info.lock().map(|guard| *guard).unwrap_or_default();
    for (label, value) in [
        (labels.cpu.clone(), format!("{:.1}%", snapshot.cpu_usage)),
        (
            labels.memory.clone(),
            format!("{:.1}%", snapshot.memory_usage),
        ),
        (labels.disk.clone(), format!("{:.1}%", snapshot.disk_usage)),
        (
            labels.upload.clone(),
            format!("{:.2} MB/s", snapshot.network_upload),
        ),
        (
            labels.download.clone(),
            format!("{:.2} MB/s", snapshot.network_download),
        ),
    ] {
        let item = MenuItem::new(format!("{}  {}", label, value), false, None);
        let _ = menu.append(&item);
    }

    let _ = menu.append(&PredefinedMenuItem::separator());

    let open = MenuItem::with_id(MenuId::new(MENU_OPEN), labels.open.clone(), true, None);
    let _ = menu.append(&open);

    let quit = MenuItem::with_id(MenuId::new(MENU_QUIT), labels.quit.clone(), true, None);
    let _ = menu.append(&quit);

    menu
}

/// 启动托盘事件监听线程。
///
/// 左键点击打开主窗口，右键点击刷新菜单内容；菜单项点击通过 `MenuEvent` 分发。
pub fn spawn_event_loop(tx: Sender<AppEvent>) {
    std::thread::spawn(move || {
        let receiver = TrayIconEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = tx.send(AppEvent::TrayOpen);
            }
        }
    });
}

/// 启动菜单点击事件监听线程。
pub fn spawn_menu_event_loop(tx: Sender<AppEvent>) {
    std::thread::spawn(move || {
        let receiver = MenuEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let id = event.id().as_ref();
            if id == MENU_OPEN {
                let _ = tx.send(AppEvent::TrayOpen);
            } else if id == MENU_QUIT {
                let _ = tx.send(AppEvent::TrayQuit);
            }
        }
    });
}
