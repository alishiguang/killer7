//! 剪贴板页，对应 Swift 版的 `ClipboardContentView.swift`。

use egui::{Align, FontFamily, FontId, Layout, RichText, Stroke};

use crate::app::Chuaker7App;
use crate::ui_kit;

/// 渲染剪贴板页。
pub fn render(ui: &mut egui::Ui, app: &mut Chuaker7App) {
    let palette = ui_kit::palette(ui.ctx());

    let title = app.i18n.t("clipboard.title");
    let current_label = app.i18n.t("clipboard.current");
    let history_label = app.i18n.t("clipboard.history");
    let search_hint = app.i18n.t("clipboard.search.placeholder");
    let clear_label = app.i18n.t("clipboard.clear");
    let clear_history_label = app.i18n.t("clipboard.clearHistory");
    let empty_label = app.i18n.t("clipboard.empty");

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::display_title(ui, &title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            if ui_kit::ghost_button(ui, &clear_history_label).clicked() {
                app.clipboard.clear_history();
            }
        });
    });

    ui.add_space(6.0);
    ui_kit::hairline(ui);
    ui.add_space(12.0);

    // 当前剪贴板内容。
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::card(ui, |ui| {
            ui.set_width(ui.available_width() - 36.0);
            ui.horizontal(|ui| {
                ui_kit::kicker(ui, &current_label);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui_kit::ghost_button(ui, &clear_label).clicked() {
                        app.clipboard.copy_to_clipboard("");
                    }
                });
            });
            ui.add_space(8.0);

            let content = if app.clipboard.current_content.is_empty() {
                empty_label.clone()
            } else {
                app.clipboard.current_content.clone()
            };

            egui::Frame {
                fill: palette.background,
                stroke: Stroke::new(1.0, palette.border),
                corner_radius: egui::CornerRadius::same(4),
                inner_margin: egui::Margin::same(8),
                ..Default::default()
            }
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(96.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(content)
                                .font(FontId::new(12.0, FontFamily::Monospace))
                                .color(if app.clipboard.current_content.is_empty() {
                                    palette.text_muted
                                } else {
                                    palette.text
                                }),
                        );
                    });
            });
        });
    });

    ui.add_space(12.0);

    // 历史搜索。
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::search_field(ui, &mut app.clipboard_search, &search_hint);
    });
    ui.add_space(10.0);

    let keyword = app.clipboard_search.trim().to_lowercase();
    let history: Vec<usize> = app
        .clipboard
        .history
        .iter()
        .enumerate()
        .filter(|(_, item)| keyword.is_empty() || item.content.to_lowercase().contains(&keyword))
        .map(|(index, _)| index)
        .collect();

    let mut intent_copy: Option<usize> = None;
    let mut intent_delete: Option<usize> = None;

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::numbered_kicker(ui, "01", &history_label);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            let count = app.clipboard.history.len();
            let text = app
                .i18n
                .tf("clipboard.history.count", &[&count.to_string()]);
            ui.label(
                RichText::new(ui_kit::tracked(&text.to_uppercase()))
                    .font(FontId::new(10.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
        });
    });
    ui.add_space(8.0);

    if history.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                RichText::new("⎘")
                    .font(FontId::new(30.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(ui_kit::tracked(&empty_label.to_uppercase()))
                    .font(FontId::new(10.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
        });
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for index in history {
                let Some(item) = app.clipboard.history.get(index) else {
                    continue;
                };

                let content = item.content.clone();
                let kind = item.kind.clone();
                let timestamp = item.timestamp;

                egui::Frame {
                    fill: palette.surface,
                    stroke: Stroke::new(1.0, palette.border),
                    corner_radius: egui::CornerRadius::same(4),
                    inner_margin: egui::Margin::symmetric(10, 8),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format_time(timestamp))
                                .font(FontId::new(10.0, FontFamily::Monospace))
                                .color(palette.text_muted),
                        );

                        ui.label(
                            RichText::new(ui_kit::tracked(&kind.to_uppercase()))
                                .font(FontId::new(9.0, FontFamily::Monospace))
                                .color(palette.accent),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let delete = ui.add(
                                egui::Button::new(
                                    RichText::new("×")
                                        .font(FontId::new(11.0, FontFamily::Monospace))
                                        .color(palette.danger),
                                )
                                .frame(false),
                            );
                            if delete.clicked() {
                                intent_delete = Some(index);
                            }

                            let copy = ui.add(
                                egui::Button::new(
                                    RichText::new("⎘")
                                        .font(FontId::new(12.0, FontFamily::Monospace))
                                        .color(palette.text_muted),
                                )
                                .frame(false),
                            );
                            if copy.clicked() {
                                intent_copy = Some(index);
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(content)
                            .font(FontId::new(12.0, FontFamily::Monospace))
                            .color(palette.text),
                    );
                });

                ui.add_space(6.0);
            }
        });
    }

    // 落地操作。
    if let Some(index) = intent_copy
        && let Some(item) = app.clipboard.history.get(index)
    {
        let content = item.content.clone();
        app.clipboard.copy_to_clipboard(&content);
        let message = app.i18n.t("clipboard.copied").to_string();
        app.notify_toast(message);
    }

    if let Some(index) = intent_delete {
        app.clipboard.delete_history_item(index);
    }
}

/// 把 UNIX 时间戳格式化成 `HH:MM:SS`（本地时区）。
fn format_time(secs: i64) -> String {
    // SAFETY: localtime_r 写入调用方提供的 tm 结构，strftime 写入调用方提供的缓冲区。
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        let time = secs as libc::time_t;
        if libc::localtime_r(&time, &mut tm).is_null() {
            return "--:--:--".to_string();
        }

        let mut buffer = [0u8; 32];
        let format = b"%H:%M:%S\0";
        let written = libc::strftime(
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
            format.as_ptr() as *const libc::c_char,
            &tm,
        );
        if written == 0 {
            return "--:--:--".to_string();
        }

        let length = buffer.iter().position(|&byte| byte == 0).unwrap_or(0);
        String::from_utf8_lossy(&buffer[..length]).to_string()
    }
}
