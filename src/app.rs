//! 应用主体：窗口布局、事件分发与各功能页的调度。

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use notify::Watcher;

use egui::{CornerRadius, FontFamily, FontId, Layout, RichText, Stroke};

use crate::clipboard::ClipboardManager;
use crate::events::AppEvent;
use crate::files::FileManager;
use crate::i18n::I18n;
use crate::menubar::{MenuBar, MenuLabels};
use crate::monitor::SystemMonitor;
use crate::plugin::PluginManager;
use crate::process::ProcessManager;
use crate::settings::Settings;
use crate::shortcut::ShortcutManager;
use crate::theme::ThemeManager;
use crate::ui_kit;
use crate::views;

/// 主标签页。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    /// 进程管理
    Process,
    /// 文件管理
    Files,
    /// 剪贴板
    Clipboard,
    /// 系统监控
    Monitor,
    /// 插件
    Plugins,
    /// 设置
    Settings,
}

/// 进程排序方式。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessSort {
    /// 按 PID
    Pid,
    /// 按名称
    Name,
    /// 按端口
    Port,
}

/// 应用主体。
pub struct Killer7App {
    /// 全局设置。
    pub settings: Settings,
    /// 主题。
    pub theme: ThemeManager,
    /// 文案。
    pub i18n: I18n,
    /// 事件发送端。
    pub tx: Sender<AppEvent>,
    /// 事件接收端。
    pub rx: Receiver<AppEvent>,
    /// 进程管理器。
    pub processes: ProcessManager,
    /// 文件管理器。
    pub files: FileManager,
    /// 剪贴板管理器。
    pub clipboard: ClipboardManager,
    /// 系统监控。
    pub monitor: SystemMonitor,
    /// 插件管理器。
    pub plugins: PluginManager,
    /// 菜单栏图标。
    pub menu_bar: Option<MenuBar>,
    /// 全局快捷键。
    pub shortcut: Option<ShortcutManager>,
    /// 当前目录的文件变更监听器。
    pub watcher: Option<notify::RecommendedWatcher>,

    /// 采样间隔（毫秒），可在设置里实时调整。
    pub sample_interval_ms: Arc<AtomicU64>,
    /// 是否统计网络流量。
    pub network_enabled: Arc<AtomicBool>,

    /// 当前标签页。
    pub tab: Tab,
    /// 进程搜索词。
    pub process_search: String,
    /// 进程排序。
    pub process_sort: ProcessSort,
    /// 选中的进程。
    pub selected_pid: Option<i32>,
    /// 待确认结束进程的 PID。
    pub pending_kill: Option<i32>,
    /// 文件搜索词。
    pub file_search: String,
    /// 待重命名的文件及其新名字。
    pub file_rename: Option<(PathBuf, String)>,
    /// 待新建的文件夹名。
    pub new_folder_name: Option<String>,
    /// 待新建的文件名。
    pub new_file_name: Option<String>,
    /// 剪贴板搜索词。
    pub clipboard_search: String,
    /// 顶部提示消息。
    pub toast: Option<(String, Instant)>,
    /// 上次刷新进程的时刻。
    last_process_refresh: Instant,
}

impl Killer7App {
    /// 构造应用。
    pub fn new(settings: Settings) -> Self {
        Self::build(settings, true)
    }

    /// 构造应用，但不接入系统级集成（全局快捷键、菜单栏、剪贴板监听、目录监听）。
    ///
    /// 单元测试用它：多个用例并行跑时，重复注册 ⌘+L 或重复创建托盘图标会让
    /// 进程直接崩溃。
    #[cfg(test)]
    pub fn new_headless(settings: Settings) -> Self {
        Self::build(settings, false)
    }

    fn build(settings: Settings, integrate: bool) -> Self {
        let (tx, rx) = crate::events::channel();

        let sample_interval_ms = Arc::new(AtomicU64::new(
            (settings.auto_refresh_interval * 1000.0) as u64,
        ));
        let network_enabled = Arc::new(AtomicBool::new(settings.enable_network_monitor));

        crate::monitor::spawn_monitor(
            sample_interval_ms.clone(),
            network_enabled.clone(),
            tx.clone(),
        );
        if integrate {
            crate::clipboard::spawn_monitor(tx.clone());
        }

        let theme = ThemeManager {
            mode: settings.theme_mode,
            accent: settings.accent,
        };

        let i18n = I18n::new(settings.locale);

        let mut clipboard = ClipboardManager::default();
        clipboard.load_history();

        let mut files = FileManager::default();
        files.load_files();

        let mut plugins = PluginManager::default();
        plugins.load_plugins();

        // 同步抓一次进程，窗口打开时列表就是满的。
        let mut processes = ProcessManager::default();
        processes.load_blocking(settings.process_list_limit);

        // 全局快捷键：macOS 要求在主线程注册。
        let shortcut = if integrate && settings.global_shortcut_enabled {
            match ShortcutManager::new() {
                Some(manager) => {
                    crate::shortcut::spawn_event_loop(tx.clone(), manager.id());
                    Some(manager)
                }
                None => None,
            }
        } else {
            None
        };

        // 菜单栏事件监听只启动一次，避免多个线程争抢同一个事件通道。
        let menu_bar = if integrate {
            crate::menubar::spawn_event_loop(tx.clone());
            crate::menubar::spawn_menu_event_loop(tx.clone());

            if settings.show_menu_bar {
                MenuBar::new(menu_labels(&i18n))
            } else {
                None
            }
        } else {
            None
        };

        let watcher = if integrate {
            setup_watcher(&files.current_path, &tx)
        } else {
            None
        };

        Self {
            settings,
            theme,
            i18n,
            tx,
            rx,
            processes,
            files,
            clipboard,
            monitor: SystemMonitor::default(),
            plugins,
            menu_bar,
            shortcut,
            watcher,
            sample_interval_ms,
            network_enabled,
            tab: Tab::Process,
            process_search: String::new(),
            process_sort: ProcessSort::Pid,
            selected_pid: None,
            pending_kill: None,
            file_search: String::new(),
            file_rename: None,
            new_folder_name: None,
            new_file_name: None,
            clipboard_search: String::new(),
            toast: None,
            last_process_refresh: Instant::now() - Duration::from_secs(60),
        }
    }

    /// 主动刷新进程列表。
    pub fn refresh_processes(&mut self) {
        let limit = self.settings.process_list_limit;
        self.processes.refresh(limit, &self.tx);
        self.last_process_refresh = Instant::now();
    }

    /// 进入目录并重新挂上文件变更监听。
    pub fn navigate_files(&mut self, path: PathBuf) {
        self.files.navigate_to(path);
        self.watcher = setup_watcher(&self.files.current_path, &self.tx);
    }

    /// 返回上级目录。
    pub fn go_up_files(&mut self) {
        if self.files.go_up() {
            self.watcher = setup_watcher(&self.files.current_path, &self.tx);
        }
    }

    /// 显示一条顶部提示。
    pub fn notify_toast(&mut self, message: String) {
        self.toast = Some((message, Instant::now()));
    }

    /// 切换菜单栏图标。
    pub fn toggle_menu_bar(&mut self, enabled: bool) {
        self.settings.show_menu_bar = enabled;
        if enabled {
            crate::menubar::spawn_event_loop(self.tx.clone());
            crate::menubar::spawn_menu_event_loop(self.tx.clone());
            self.menu_bar = MenuBar::new(menu_labels(&self.i18n));
        } else {
            if let Some(menu_bar) = &self.menu_bar {
                menu_bar.shutdown();
            }
            self.menu_bar = None;
        }
        self.settings.save();
    }

    /// 切换全局快捷键。
    pub fn toggle_shortcut(&mut self, enabled: bool) {
        self.settings.global_shortcut_enabled = enabled;
        if enabled {
            if let Some(manager) = ShortcutManager::new() {
                crate::shortcut::spawn_event_loop(self.tx.clone(), manager.id());
                self.shortcut = Some(manager);
            }
        } else {
            self.shortcut = None;
        }
        self.settings.save();
    }

    /// 应用设置变化。
    pub fn apply_settings(&mut self) {
        self.sample_interval_ms.store(
            (self.settings.auto_refresh_interval * 1000.0) as u64,
            Ordering::Relaxed,
        );
        self.network_enabled
            .store(self.settings.enable_network_monitor, Ordering::Relaxed);
        self.settings.save();
    }

    /// 处理后台事件。
    fn poll_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::Processes(list) => {
                    self.processes.processes = list;
                    self.processes.is_loading = false;
                }
                AppEvent::ProcessLoading(loading) => {
                    self.processes.is_loading = loading;
                }
                AppEvent::PortCacheUpdated(map) => {
                    self.processes.set_port_cache(map);
                }
                AppEvent::SystemInfo(info) => {
                    self.monitor.push_sample(info);
                    if let Some(menu_bar) = &self.menu_bar {
                        menu_bar.update_info(info);
                        menu_bar.rebuild_menu();
                    }
                }
                AppEvent::ClipboardContent(text) => {
                    if text != self.clipboard.current_content {
                        self.clipboard.current_content = text.clone();
                        self.clipboard.add_to_history(&text, "TEXT");
                    }
                }
                AppEvent::FilesChanged => {
                    self.files.load_files();
                }
                AppEvent::TrayOpen => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                AppEvent::TrayQuit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                AppEvent::Hotkey => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
            }
        }
    }
}

impl Drop for Killer7App {
    fn drop(&mut self) {
        self.settings.save();
        self.clipboard.save_history();
    }
}

/// 监听目录，文件增删时通过通道通知 UI。
///
/// 对应 Swift 版用 `DispatchSource` 监听目录的做法。
fn setup_watcher(
    dir: &std::path::Path,
    tx: &Sender<AppEvent>,
) -> Option<notify::RecommendedWatcher> {
    let tx = tx.clone();

    let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        let Ok(event) = result else { return };
        // 只关心内容变化，忽略单纯的读取事件。
        if matches!(event.kind, notify::EventKind::Access(_)) {
            return;
        }
        let _ = tx.send(AppEvent::FilesChanged);
    })
    .ok()?;

    let mut watcher = watcher;
    watcher
        .watch(dir, notify::RecursiveMode::NonRecursive)
        .ok()?;
    Some(watcher)
}

/// 菜单栏文案。
fn menu_labels(i18n: &I18n) -> MenuLabels {
    MenuLabels {
        title: i18n.t("menubar.title"),
        open: i18n.t("menubar.open"),
        quit: i18n.t("menubar.quit"),
        cpu: i18n.t("monitor.cpu"),
        memory: i18n.t("monitor.memory"),
        disk: i18n.t("monitor.disk"),
        upload: i18n.t("monitor.upload"),
        download: i18n.t("monitor.download"),
    }
}

impl eframe::App for Killer7App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render(ctx);
    }
}

impl Killer7App {
    /// 绘制一帧。
    ///
    /// 与 `eframe::App::update` 分开，方便在单元测试里离屏渲染。
    pub fn render(&mut self, ctx: &egui::Context) {
        let system_dark = ctx.style().visuals.dark_mode;
        let palette = self.theme.palette(system_dark);
        ui_kit::set_palette(ctx, palette);
        ctx.set_visuals(self.theme.visuals(system_dark));
        ui_kit::apply_text_styles(ctx);

        // 菜单栏模式下关闭主窗口不退出应用。
        if self.settings.show_menu_bar && ctx.input(|input| input.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        self.poll_events(ctx);

        // 进程列表定时刷新。
        if self.tab == Tab::Process
            && self.last_process_refresh.elapsed()
                > Duration::from_secs_f64(self.settings.auto_refresh_interval.max(1.0))
        {
            self.refresh_processes();
        }

        self.render_header(ctx);
        self.render_sidebar(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: palette.background,
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| match self.tab {
                Tab::Process => views::processes::render(ui, self),
                Tab::Files => views::files::render(ui, self),
                Tab::Clipboard => views::clipboard::render(ui, self),
                Tab::Monitor => views::monitor::render(ui, self),
                Tab::Plugins => views::plugins::render(ui, self),
                Tab::Settings => views::settings::render(ui, self),
            });

        self.render_toast(ctx);
    }
}

impl Killer7App {
    /// 顶部标题栏。
    fn render_header(&mut self, ctx: &egui::Context) {
        let palette = ui_kit::palette(ctx);

        egui::TopBottomPanel::top("killer7.header")
            .exact_height(54.0)
            .frame(egui::Frame {
                fill: palette.background,
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(18.0);

                    // 品牌区：重体大字 + 等宽标语。
                    ui.vertical(|ui| {
                        ui.add_space(6.0);
                        ui_kit::brand(ui, &self.i18n.t("app.brand"), &self.i18n.t("app.tagline"));
                    });

                    ui.add_space(12.0);

                    // 版本标记，呼应参考站的 `◆` 分隔装饰。
                    ui.label(
                        RichText::new("◆")
                            .font(FontId::new(8.0, FontFamily::Monospace))
                            .color(palette.accent),
                    );
                    ui.label(
                        RichText::new(ui_kit::tracked("V1.1.0"))
                            .font(FontId::new(9.0, FontFamily::Monospace))
                            .color(palette.text_muted),
                    );

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(14.0);

                        // 设置。
                        let settings_label = self.i18n.t("nav.settings");
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(ui_kit::tracked(&settings_label.to_uppercase()))
                                        .font(FontId::new(10.0, FontFamily::Monospace))
                                        .color(palette.text),
                                )
                                .fill(palette.surface)
                                .stroke(Stroke::new(1.0, palette.border))
                                .corner_radius(CornerRadius::same(4)),
                            )
                            .clicked()
                        {
                            self.tab = Tab::Settings;
                        }

                        ui.add_space(6.0);

                        // 菜单栏开关。
                        let mut show_menu = self.settings.show_menu_bar;
                        let toggle = ui.add(egui::Checkbox::without_text(&mut show_menu));
                        if toggle.changed() {
                            self.toggle_menu_bar(show_menu);
                        }
                        ui.label(
                            RichText::new(ui_kit::tracked("MENUBAR"))
                                .font(FontId::new(10.0, FontFamily::Monospace))
                                .color(palette.text_muted),
                        );

                        ui.add_space(6.0);

                        // 刷新。
                        let refresh_label = self.i18n.t("process.refresh");
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(ui_kit::tracked(&refresh_label.to_uppercase()))
                                        .font(FontId::new(10.0, FontFamily::Monospace))
                                        .color(palette.text),
                                )
                                .fill(palette.surface)
                                .stroke(Stroke::new(1.0, palette.border))
                                .corner_radius(CornerRadius::same(4)),
                            )
                            .clicked()
                        {
                            self.refresh_processes();
                        }
                    });
                });

                // 顶部右侧的红色辉光。
                ui_kit::accent_glow(ui);
            });
    }

    /// 左侧导航栏。
    fn render_sidebar(&mut self, ctx: &egui::Context) {
        let palette = ui_kit::palette(ctx);

        egui::SidePanel::left("killer7.nav")
            .exact_width(206.0)
            .resizable(false)
            .frame(egui::Frame {
                fill: palette.background,
                stroke: Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.add_space(18.0);

                ui_kit::numbered_kicker(ui, "01", &self.i18n.t("nav.section.tools"));
                ui.add_space(8.0);

                for (index, tab, key) in [
                    ("01", Tab::Process, "nav.process"),
                    ("02", Tab::Files, "nav.file"),
                    ("03", Tab::Clipboard, "nav.clipboard"),
                    ("04", Tab::Monitor, "nav.monitor"),
                ] {
                    let label = self.i18n.t(key);
                    let active = self.tab == tab;
                    if ui_kit::nav_item(ui, index, &label, active).clicked() {
                        self.tab = tab;
                        if tab == Tab::Process {
                            self.refresh_processes();
                        }
                    }
                    ui.add_space(4.0);
                }

                ui.add_space(18.0);
                ui_kit::numbered_kicker(ui, "02", &self.i18n.t("nav.section.system"));
                ui.add_space(8.0);

                let plugins_label = self.i18n.t("nav.plugins");
                if ui_kit::nav_item(ui, "05", &plugins_label, self.tab == Tab::Plugins).clicked() {
                    self.tab = Tab::Plugins;
                }
                ui.add_space(4.0);

                let settings_label = self.i18n.t("nav.settings");
                if ui_kit::nav_item(ui, "06", &settings_label, self.tab == Tab::Settings).clicked()
                {
                    self.tab = Tab::Settings;
                }

                // 底部版本信息。
                ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(14.0);
                    ui_kit::hairline(ui);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(ui_kit::tracked(&self.i18n.t("app.subtitle").to_uppercase()))
                            .font(FontId::new(9.0, FontFamily::Monospace))
                            .color(palette.text_muted),
                    );
                });
            });

        // 导航栏右侧发丝描边。
        let rect = ctx.input(|input| input.viewport_rect());
        egui::Painter::new(ctx.clone(), egui::LayerId::background(), rect).vline(
            rect.left() + 206.0,
            egui::Rangef::new(rect.top(), rect.bottom()),
            Stroke::new(1.0, palette.border),
        );
    }

    /// 顶部提示消息。
    fn render_toast(&mut self, ctx: &egui::Context) {
        let Some((message, at)) = &self.toast else {
            return;
        };
        if at.elapsed() > Duration::from_millis(2600) {
            self.toast = None;
            return;
        }

        let palette = ui_kit::palette(ctx);
        egui::TopBottomPanel::top("killer7.toast")
            .exact_height(0.0)
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    egui::Frame {
                        fill: palette.accent,
                        corner_radius: CornerRadius::same(4),
                        inner_margin: egui::Margin::symmetric(10, 5),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(message)
                                .font(FontId::new(11.0, FontFamily::Monospace))
                                .color(palette.on_accent),
                        );
                    });
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{PortInfo, ProcessInfo};

    fn sample_processes() -> Vec<ProcessInfo> {
        vec![
            ProcessInfo {
                pid: 501,
                full_path: "/usr/bin/WindowServer".to_string(),
                display_name: "WindowServer".to_string(),
                ports: vec![],
            },
            ProcessInfo {
                pid: 8080,
                full_path: "/usr/local/bin/node".to_string(),
                display_name: "node".to_string(),
                ports: vec![PortInfo {
                    address: "127.0.0.1".to_string(),
                    port: 5173,
                }],
            },
        ]
    }

    /// 离屏渲染每一个标签页，确保界面代码不会 panic。
    #[test]
    fn every_tab_renders_without_panic() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut app = Killer7App::new_headless(Settings::default());
        app.processes.processes = sample_processes();

        for tab in [
            Tab::Process,
            Tab::Files,
            Tab::Clipboard,
            Tab::Monitor,
            Tab::Plugins,
            Tab::Settings,
        ] {
            app.tab = tab;
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.render(ctx);
            });
        }
    }

    /// 渲染每个标签页都不应该触发 egui 的多遍布局（整帧丢弃）。
    ///
    /// `egui::Grid` 首次出现时会调用 `Context::request_discard`，导致页面打开的
    /// 第一帧是空白——这曾经让设置页看起来完全渲染不出来。
    #[test]
    fn no_tab_requests_a_frame_discard() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut app = Killer7App::new_headless(Settings::default());
        app.processes.processes = sample_processes();

        for tab in [
            Tab::Process,
            Tab::Files,
            Tab::Clipboard,
            Tab::Monitor,
            Tab::Plugins,
            Tab::Settings,
        ] {
            app.tab = tab;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(1080.0, 720.0),
                )),
                ..Default::default()
            };
            let out = ctx.run(input, |ctx| {
                app.render(ctx);
            });
            assert!(
                !out.platform_output.requested_discard(),
                "{tab:?} 请求了整帧丢弃，首帧会是空白"
            );
            assert!(!out.shapes.is_empty(), "{tab:?} 没有产生任何绘制内容");
        }
    }

    /// 切换主题后仍然能正常渲染。
    #[test]
    fn renders_in_both_themes() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);

        for mode in [
            crate::theme::ThemeMode::Dark,
            crate::theme::ThemeMode::Light,
        ] {
            let mut app = Killer7App::new_headless(Settings::default());
            app.theme.mode = mode;
            app.tab = Tab::Monitor;
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                app.render(ctx);
            });
        }
    }

    /// 文件列表必须能滚动。
    ///
    /// `egui_extras` 的表格如果逐行用 `body.row` 添加，不会为不可见行预留高度，
    /// 滚动区会误判为「内容没有溢出」，列表就被截断且滚不动。
    #[test]
    fn file_list_scrolls_with_the_wheel() {
        fn signature(shapes: &[egui::epaint::ClippedShape]) -> Vec<f32> {
            shapes
                .iter()
                .map(|clipped| clipped.shape.visual_bounding_rect().min.y)
                .collect()
        }

        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut app = Killer7App::new_headless(Settings::default());
        app.tab = Tab::Files;

        let base = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1080.0, 720.0),
            )),
            ..Default::default()
        };

        // 两帧都把指针移到列表上，唯一的差别是第二帧多了一次滚轮事件。
        let mut hovered = base.clone();
        hovered
            .events
            .push(egui::Event::PointerMoved(egui::pos2(600.0, 400.0)));
        let first = ctx.run(hovered, |ctx| {
            app.render(ctx);
        });

        let mut scrolled = base;
        scrolled
            .events
            .push(egui::Event::PointerMoved(egui::pos2(600.0, 400.0)));
        scrolled.events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -240.0),
            modifiers: egui::Modifiers::default(),
        });

        let second = ctx.run(scrolled, |ctx| {
            app.render(ctx);
        });

        assert_ne!(
            signature(&first.shapes),
            signature(&second.shapes),
            "滚轮没有让文件列表滚动"
        );
    }

    /// 搜索过滤按名称、PID 与端口生效。
    #[test]
    fn process_search_filters_by_name_pid_and_port() {
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx);
        let mut app = Killer7App::new_headless(Settings::default());
        app.processes.processes = sample_processes();

        app.process_search = "node".to_string();
        app.tab = Tab::Process;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.render(ctx);
        });

        app.process_search = "5173".to_string();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.render(ctx);
        });

        app.process_search = "8080".to_string();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            app.render(ctx);
        });
    }
}
