//! 系统监控页，对应 Swift 版的 `SystemMonitorView.swift`。

use egui::{Align, FontFamily, FontId, Layout, RichText};

use crate::app::Killer7App;
use crate::ui_kit;

/// 渲染系统监控页。
pub fn render(ui: &mut egui::Ui, app: &mut Killer7App) {
    let palette = ui_kit::palette(ui.ctx());

    let title = app.i18n.t("monitor.title");
    let cpu_label = app.i18n.t("monitor.cpu");
    let memory_label = app.i18n.t("monitor.memory");
    let disk_label = app.i18n.t("monitor.disk");
    let network_label = app.i18n.t("monitor.network");
    let upload_label = app.i18n.t("monitor.upload");
    let download_label = app.i18n.t("monitor.download");
    let cpu_history_label = app.i18n.t("monitor.cpu.history");
    let memory_history_label = app.i18n.t("monitor.memory.history");

    let info = app.monitor.system_info;

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::display_title(ui, &title);
    });

    ui.add_space(6.0);
    ui_kit::hairline(ui);
    ui.add_space(14.0);

    // 三个百分比指标。
    ui.horizontal(|ui| {
        ui.add_space(18.0);

        // 三等分，两侧各留 18px 空白，卡片之间留 10px。
        let available = ui.available_width() - 18.0;
        let card_width = ((available - 2.0 * 10.0) / 3.0).max(100.0);

        for (index, (label, value, color)) in [
            (&cpu_label, info.cpu_usage, palette.accent),
            (&memory_label, info.memory_usage, palette.success),
            (&disk_label, info.disk_usage, palette.warning),
        ]
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                ui.add_space(10.0);
            }

            ui_kit::fixed_card(ui, card_width, 104.0, |ui| {
                ui_kit::stat_block(ui, label, &format!("{:.1}", value), "%", color);
                ui.add_space(8.0);
                ui_kit::meter(ui, (value / 100.0) as f32, color);
            });
        }
    });

    ui.add_space(12.0);

    // 网络速度。
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        let card_width = ui.available_width() - 18.0;
        ui_kit::fixed_card(ui, card_width, 96.0, |ui| {
            ui_kit::kicker(ui, &network_label);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui_kit::stat_block(
                    ui,
                    &upload_label,
                    &format!("{:.2}", info.network_upload),
                    "MB/s",
                    palette.accent,
                );
                ui.add_space(40.0);
                ui_kit::stat_block(
                    ui,
                    &download_label,
                    &format!("{:.2}", info.network_download),
                    "MB/s",
                    palette.success,
                );
            });
        });
    });

    ui.add_space(12.0);

    // 采样曲线。
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        let card_width = ui.available_width() - 18.0;
        ui_kit::fixed_card(ui, card_width, 232.0, |ui| {
            ui.horizontal(|ui| {
                ui_kit::kicker(ui, &cpu_history_label);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(ui_kit::tracked("0 — 100%"))
                            .font(FontId::new(9.0, FontFamily::Monospace))
                            .color(palette.text_muted),
                    );
                });
            });
            ui_kit::sparkline(ui, &app.monitor.cpu_history, 100.0, palette.accent);

            ui.add_space(14.0);

            ui.horizontal(|ui| {
                ui_kit::kicker(ui, &memory_history_label);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(ui_kit::tracked("0 — 100%"))
                            .font(FontId::new(9.0, FontFamily::Monospace))
                            .color(palette.text_muted),
                    );
                });
            });
            ui_kit::sparkline(ui, &app.monitor.memory_history, 100.0, palette.success);
        });
    });
}
