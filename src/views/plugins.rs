//! 插件页，对应 Swift 版 `PluginSystem.swift` 中的插件发现与展示。

use egui::{Align, FontFamily, FontId, Layout, RichText};

use crate::app::Killer7App;
use crate::ui_kit;

/// 渲染插件页。
pub fn render(ui: &mut egui::Ui, app: &mut Killer7App) {
    let palette = ui_kit::palette(ui.ctx());

    let title = app.i18n.t("nav.plugins");
    let empty_label = app.i18n.t("settings.plugin.empty");
    let hint_template = app.i18n.t("settings.plugin.hint");
    let dir_label = app.i18n.t("settings.plugin.dir");
    let manifest_label = app.i18n.t("plugin.manifest");
    let author_label = app.i18n.t("plugin.author");
    let version_label = app.i18n.t("plugin.version");
    let permissions_label = app.i18n.t("plugin.permissions");
    let min_version_label = app.i18n.t("plugin.minVersion");
    let tabs_label = app.i18n.t("nav.plugins");
    let commands_label = app.i18n.t("file.column.action");

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::display_title(ui, &title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            let count = app.plugins.loaded_plugins.len();
            ui.label(
                RichText::new(ui_kit::tracked(&format!("{} PLUGINS", count)))
                    .font(FontId::new(10.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
        });
    });

    ui.add_space(6.0);
    ui_kit::hairline(ui);
    ui.add_space(14.0);

    let plugins_dir = crate::settings::plugins_dir();
    let hint = hint_template.replace("%@", &plugins_dir.to_string_lossy());

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::kicker(ui, &dir_label);
        ui.add_space(10.0);
        ui.label(
            RichText::new(plugins_dir.to_string_lossy().to_string())
                .font(FontId::new(11.0, FontFamily::Monospace))
                .color(palette.text_muted),
        );
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui.label(
            RichText::new(hint)
                .font(FontId::new(11.0, FontFamily::Proportional))
                .color(palette.text_muted),
        );
    });
    ui.add_space(14.0);

    if app.plugins.loaded_plugins.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.label(
                RichText::new("◆")
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
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for plugin in &app.plugins.loaded_plugins {
            egui::Frame {
                fill: palette.surface,
                stroke: egui::Stroke::new(1.0, palette.border),
                corner_radius: egui::CornerRadius::same(6),
                inner_margin: egui::Margin::same(14),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("◆")
                            .font(FontId::new(14.0, FontFamily::Monospace))
                            .color(palette.accent),
                    );
                    ui.label(
                        RichText::new(plugin.name.clone())
                            .font(FontId::new(15.0, FontFamily::Proportional))
                            .color(palette.text),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(ui_kit::tracked(&plugin.version.to_uppercase()))
                                .font(FontId::new(10.0, FontFamily::Monospace))
                                .color(palette.text_muted),
                        );
                    });
                });

                ui.add_space(8.0);

                if let Some(description) = &plugin.description {
                    ui.label(
                        RichText::new(description.clone())
                            .font(FontId::new(12.0, FontFamily::Proportional))
                            .color(palette.text_muted),
                    );
                    ui.add_space(8.0);
                }

                ui_kit::hairline(ui);
                ui.add_space(8.0);

                // 已注册的标签页与命令，体现插件注册机制确实生效。
                if !app.plugins.plugin_tabs.is_empty() {
                    ui_kit::kicker(ui, &tabs_label);
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for tab in &app.plugins.plugin_tabs {
                            ui.label(
                                RichText::new(format!("{} {}", tab.icon, tab.title))
                                    .font(FontId::new(11.0, FontFamily::Monospace))
                                    .color(palette.accent),
                            );
                        }
                    });
                    ui.add_space(8.0);
                }

                if !app.plugins.plugin_commands.is_empty() {
                    ui_kit::kicker(ui, &commands_label);
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for command in &app.plugins.plugin_commands {
                            ui.label(
                                RichText::new(format!("{} {}", command.icon, command.title))
                                    .font(FontId::new(11.0, FontFamily::Monospace))
                                    .color(palette.text_muted),
                            );
                        }
                    });
                    ui.add_space(8.0);
                }

                ui_kit::hairline(ui);
                ui.add_space(8.0);

                // 用手动键值行而不是 `egui::Grid`：`Grid` 首次出现时会请求丢弃整帧，
                // 导致插件页打开的第一帧是空白。
                ui_kit::key_value(ui, 170.0, &manifest_label, &plugin.id);
                if let Some(author) = &plugin.author {
                    ui_kit::key_value(ui, 170.0, &author_label, author);
                }
                ui_kit::key_value(ui, 170.0, &version_label, &plugin.version);
                if let Some(min_version) = &plugin.minimum_app_version {
                    ui_kit::key_value(ui, 170.0, &min_version_label, min_version);
                }
                if let Some(permissions) = &plugin.permissions {
                    ui_kit::key_value(ui, 170.0, &permissions_label, &permissions.join(", "));
                }
            });

            ui.add_space(10.0);
        }
    });
}
