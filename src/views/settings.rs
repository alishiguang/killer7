//! 设置页，对应 Swift 版的 `SettingsView.swift`。
//!
//! Swift 版用 `Form` + `Section` 分组，这里换成参考站的卡片式分块：小标题用等宽
//! 全大写，每组一张带发丝描边的卡片。

use egui::{FontFamily, FontId, RichText};

use crate::app::Killer7App;
use crate::theme::{Accent, ThemeMode};
use crate::ui_kit;

/// 渲染设置页。
pub fn render(ui: &mut egui::Ui, app: &mut Killer7App) {
    let palette = ui_kit::palette(ui.ctx());

    let startup_label = app.i18n.t("settings.startup");
    let auto_launch = app.i18n.t("settings.autoLaunch");
    let auto_launch_hint = app.i18n.t("settings.autoLaunch.hint");
    let shortcut_label = app.i18n.t("settings.globalShortcut");
    let shortcut_hint = app.i18n.t("settings.globalShortcut.hint");
    let appearance_label = app.i18n.t("settings.appearance");
    let menu_bar_label = app.i18n.t("settings.enableMenuBar");
    let menu_bar_hint = app.i18n.t("settings.menuBar.hint");
    let theme_mode_label = app.i18n.t("settings.theme.mode");
    let accent_label = app.i18n.t("settings.accent");
    let performance_label = app.i18n.t("settings.performance");
    let refresh_label = app.i18n.t("settings.refresh.interval");
    let limit_label = app.i18n.t("settings.processLimit");
    let no_limit = app.i18n.t("settings.noLimit");
    let network_label = app.i18n.t("settings.networkMonitor");
    let performance_hint = app.i18n.t("settings.performance.hint");
    let general_label = app.i18n.t("settings.general");
    let locale_label = app.i18n.t("settings.locale");
    let about_label = app.i18n.t("settings.about");
    let version_label = app.i18n.t("settings.version");
    let bundle_label = app.i18n.t("settings.bundleId");
    let copyright_label = app.i18n.t("settings.copyright");

    // 设置项较多，放进纵向滚动区，保证小窗口下也能看到「关于」区块。
    egui::ScrollArea::vertical()
        .id_salt("killer7.settings_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);

                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());

                    // 启动项。
                    ui_kit::numbered_kicker(ui, "01", &startup_label);
                    ui.add_space(8.0);
                    ui_kit::card(ui, |ui| {
                        // 以 LaunchAgent 是否存在为准，反映真实自启状态。
                        let mut launch =
                            app.settings.launch_at_login || crate::autolaunch::is_enabled();
                        if ui
                            .add(egui::Checkbox::new(&mut launch, &auto_launch))
                            .changed()
                        {
                            let ok = if launch {
                                crate::autolaunch::enable().is_ok()
                            } else {
                                crate::autolaunch::disable().is_ok()
                            };
                            app.settings.launch_at_login = launch && ok;
                            app.settings.save();
                        }
                        ui.label(
                            RichText::new(auto_launch_hint.clone())
                                .font(FontId::new(11.0, FontFamily::Proportional))
                                .color(palette.text_muted),
                        );

                        ui.add_space(10.0);
                        ui_kit::hairline(ui);
                        ui.add_space(10.0);

                        let mut shortcut = app.settings.global_shortcut_enabled;
                        if ui
                            .add(egui::Checkbox::new(&mut shortcut, &shortcut_label))
                            .changed()
                        {
                            app.toggle_shortcut(shortcut);
                        }
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(ui_kit::tracked("⌘ + L"))
                                    .font(FontId::new(11.0, FontFamily::Monospace))
                                    .color(palette.accent),
                            );
                            ui.label(
                                RichText::new(shortcut_hint.clone())
                                    .font(FontId::new(11.0, FontFamily::Proportional))
                                    .color(palette.text_muted),
                            );
                        });
                    });

                    ui.add_space(16.0);

                    // 外观。
                    ui_kit::numbered_kicker(ui, "02", &appearance_label);
                    ui.add_space(8.0);
                    ui_kit::card(ui, |ui| {
                        let mut menu_bar = app.settings.show_menu_bar;
                        if ui
                            .add(egui::Checkbox::new(&mut menu_bar, &menu_bar_label))
                            .changed()
                        {
                            app.toggle_menu_bar(menu_bar);
                        }
                        ui.label(
                            RichText::new(menu_bar_hint.clone())
                                .font(FontId::new(11.0, FontFamily::Proportional))
                                .color(palette.text_muted),
                        );

                        ui.add_space(10.0);
                        ui_kit::hairline(ui);
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui_kit::kicker(ui, &theme_mode_label);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    for mode in ThemeMode::ALL {
                                        let selected = app.theme.mode == mode;
                                        let label = match mode {
                                            ThemeMode::System => {
                                                app.i18n.t("settings.theme.system")
                                            }
                                            ThemeMode::Light => app.i18n.t("settings.theme.light"),
                                            ThemeMode::Dark => app.i18n.t("settings.theme.dark"),
                                        };
                                        let text = if selected {
                                            palette.on_accent
                                        } else {
                                            palette.text_muted
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(ui_kit::tracked(
                                                        &label.to_uppercase(),
                                                    ))
                                                    .font(FontId::new(9.0, FontFamily::Monospace))
                                                    .color(text),
                                                )
                                                .fill(if selected {
                                                    palette.accent
                                                } else {
                                                    palette.surface_alt
                                                })
                                                .stroke(egui::Stroke::new(
                                                    1.0,
                                                    if selected {
                                                        palette.accent
                                                    } else {
                                                        palette.border
                                                    },
                                                ))
                                                .corner_radius(egui::CornerRadius::same(4)),
                                            )
                                            .clicked()
                                        {
                                            app.theme.mode = mode;
                                            app.settings.theme_mode = mode;
                                            app.settings.save();
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(10.0);
                        ui_kit::hairline(ui);
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui_kit::kicker(ui, &accent_label);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    for accent in Accent::ALL {
                                        let selected = app.theme.accent == accent;
                                        let color = match accent {
                                            Accent::Signal => {
                                                egui::Color32::from_rgb(0xF4, 0x3D, 0x3F)
                                            }
                                            Accent::Haze => {
                                                egui::Color32::from_rgb(0xA9, 0xC6, 0xE8)
                                            }
                                            Accent::Moss => {
                                                egui::Color32::from_rgb(0x7F, 0xB0, 0x69)
                                            }
                                            Accent::Amber => {
                                                egui::Color32::from_rgb(0xE9, 0xB4, 0x4C)
                                            }
                                        };

                                        let response = ui.add(
                                            egui::Button::new(
                                                RichText::new(if selected { "●" } else { "○" })
                                                    .font(FontId::new(13.0, FontFamily::Monospace))
                                                    .color(color),
                                            )
                                            .frame(false),
                                        );
                                        if response.clicked() {
                                            app.theme.accent = accent;
                                            app.settings.accent = accent;
                                            app.settings.save();
                                        }
                                    }
                                },
                            );
                        });
                    });

                    ui.add_space(16.0);

                    // 性能。
                    ui_kit::numbered_kicker(ui, "03", &performance_label);
                    ui.add_space(8.0);
                    ui_kit::card(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui_kit::kicker(ui, &refresh_label);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    for interval in [1.0_f64, 2.0, 5.0, 10.0] {
                                        let selected =
                                            (app.settings.auto_refresh_interval - interval).abs()
                                                < 0.01;
                                        let text = if selected {
                                            palette.on_accent
                                        } else {
                                            palette.text_muted
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(ui_kit::tracked(&format!(
                                                        "{}S",
                                                        interval as i64
                                                    )))
                                                    .font(FontId::new(9.0, FontFamily::Monospace))
                                                    .color(text),
                                                )
                                                .fill(if selected {
                                                    palette.accent
                                                } else {
                                                    palette.surface_alt
                                                })
                                                .stroke(egui::Stroke::new(
                                                    1.0,
                                                    if selected {
                                                        palette.accent
                                                    } else {
                                                        palette.border
                                                    },
                                                ))
                                                .corner_radius(egui::CornerRadius::same(4)),
                                            )
                                            .clicked()
                                        {
                                            app.settings.auto_refresh_interval = interval;
                                            app.apply_settings();
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(10.0);
                        ui_kit::hairline(ui);
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui_kit::kicker(ui, &limit_label);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    for limit in [100_usize, 200, 500, 1000] {
                                        let selected = app.settings.process_list_limit == limit;
                                        let text = if selected {
                                            palette.on_accent
                                        } else {
                                            palette.text_muted
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(ui_kit::tracked(
                                                        &limit.to_string(),
                                                    ))
                                                    .font(FontId::new(9.0, FontFamily::Monospace))
                                                    .color(text),
                                                )
                                                .fill(if selected {
                                                    palette.accent
                                                } else {
                                                    palette.surface_alt
                                                })
                                                .stroke(egui::Stroke::new(
                                                    1.0,
                                                    if selected {
                                                        palette.accent
                                                    } else {
                                                        palette.border
                                                    },
                                                ))
                                                .corner_radius(egui::CornerRadius::same(4)),
                                            )
                                            .clicked()
                                        {
                                            app.settings.process_list_limit = limit;
                                            app.settings.save();
                                        }
                                    }

                                    let selected = app.settings.process_list_limit == 0;
                                    let text = if selected {
                                        palette.on_accent
                                    } else {
                                        palette.text_muted
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(ui_kit::tracked(
                                                    &no_limit.to_uppercase(),
                                                ))
                                                .font(FontId::new(9.0, FontFamily::Monospace))
                                                .color(text),
                                            )
                                            .fill(if selected {
                                                palette.accent
                                            } else {
                                                palette.surface_alt
                                            })
                                            .stroke(egui::Stroke::new(
                                                1.0,
                                                if selected {
                                                    palette.accent
                                                } else {
                                                    palette.border
                                                },
                                            ))
                                            .corner_radius(egui::CornerRadius::same(4)),
                                        )
                                        .clicked()
                                    {
                                        app.settings.process_list_limit = 0;
                                        app.settings.save();
                                    }
                                },
                            );
                        });

                        ui.add_space(10.0);
                        ui_kit::hairline(ui);
                        ui.add_space(10.0);

                        let mut network = app.settings.enable_network_monitor;
                        if ui
                            .add(egui::Checkbox::new(&mut network, &network_label))
                            .changed()
                        {
                            app.settings.enable_network_monitor = network;
                            app.apply_settings();
                        }
                        ui.label(
                            RichText::new(performance_hint.clone())
                                .font(FontId::new(11.0, FontFamily::Proportional))
                                .color(palette.text_muted),
                        );
                    });

                    ui.add_space(16.0);

                    // 通用。
                    ui_kit::numbered_kicker(ui, "04", &general_label);
                    ui.add_space(8.0);
                    ui_kit::card(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui_kit::kicker(ui, &locale_label);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    for locale in
                                        [crate::i18n::Locale::ZhHans, crate::i18n::Locale::En]
                                    {
                                        let selected = app.i18n.locale() == locale;
                                        let text = if selected {
                                            palette.on_accent
                                        } else {
                                            palette.text_muted
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(ui_kit::tracked(locale.code()))
                                                        .font(FontId::new(
                                                            9.0,
                                                            FontFamily::Monospace,
                                                        ))
                                                        .color(text),
                                                )
                                                .fill(if selected {
                                                    palette.accent
                                                } else {
                                                    palette.surface_alt
                                                })
                                                .stroke(egui::Stroke::new(
                                                    1.0,
                                                    if selected {
                                                        palette.accent
                                                    } else {
                                                        palette.border
                                                    },
                                                ))
                                                .corner_radius(egui::CornerRadius::same(4)),
                                            )
                                            .clicked()
                                        {
                                            app.i18n.set_locale(locale);
                                            app.settings.locale = locale;
                                            app.settings.save();
                                        }
                                    }
                                },
                            );
                        });
                    });

                    ui.add_space(16.0);

                    // 关于。
                    ui_kit::numbered_kicker(ui, "05", &about_label);
                    ui.add_space(8.0);
                    ui_kit::card(ui, |ui| {
                        for (label, value) in [
                            (version_label.as_str(), "1.1.0"),
                            (bundle_label.as_str(), "org.igigi.killer7"),
                            (copyright_label.as_str(), "© 2026 Killer7 · MIT"),
                        ] {
                            ui_kit::key_value(ui, 110.0, label, value);
                        }
                    });

                    ui.add_space(24.0);
                });
            });
        });
}
