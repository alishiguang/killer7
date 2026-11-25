//! chuaker7 —— macOS 系统工具箱（SwiftUI 原版 LuciBox 的 Rust 重写）。
//!
//! 入口负责加载设置、准备字体并启动 egui 运行时。

mod app;
mod autolaunch;
mod clipboard;
mod events;
mod files;
mod fonts;
mod i18n;
mod icon;
mod menubar;
mod monitor;
mod platform;
mod plugin;
mod process;
mod settings;
mod shortcut;
mod theme;
mod ui_kit;
mod views;

use app::Chuaker7App;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let settings = settings::Settings::load();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 720.0])
            .with_min_inner_size([880.0, 600.0])
            .with_title("chuaker7"),
        ..Default::default()
    };

    eframe::run_native(
        "chuaker7",
        options,
        Box::new(|creation| {
            fonts::install(&creation.egui_ctx);
            let mut app = Chuaker7App::new(settings);
            if let Some(tab) = start_tab() {
                app.tab = tab;
            }
            Ok(Box::new(app))
        }),
    )
}

/// 允许用 `CHUAKER7_START_TAB` 指定启动时打开的标签页。
///
/// 主要用于自动化截图和「打开时直接进入某个功能」的场景。
fn start_tab() -> Option<app::Tab> {
    let raw = std::env::var("CHUAKER7_START_TAB").ok()?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "process" | "processes" => Some(app::Tab::Process),
        "files" | "file" => Some(app::Tab::Files),
        "clipboard" => Some(app::Tab::Clipboard),
        "monitor" => Some(app::Tab::Monitor),
        "plugins" | "plugin" => Some(app::Tab::Plugins),
        "settings" => Some(app::Tab::Settings),
        _ => None,
    }
}
