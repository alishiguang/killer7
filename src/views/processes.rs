//! 进程管理页，对应 Swift 版的 `ContentView.swift`。

use egui::{Align, FontFamily, FontId, Layout, RichText, Stroke};

use crate::app::{Killer7App, ProcessSort};
use crate::process::ProcessInfo;
use crate::ui_kit;

/// 渲染进程管理页。
pub fn render(ui: &mut egui::Ui, app: &mut Killer7App) {
    let palette = ui_kit::palette(ui.ctx());

    // 先取出文案，避免在表格闭包里借用 app。
    let title = app.i18n.t("process.title");
    let search_hint = app.i18n.t("process.search.placeholder");
    let pid_label = app.i18n.t("process.pid");
    let name_label = app.i18n.t("process.name");
    let port_label = app.i18n.t("process.port");
    let empty_label = app.i18n.t("process.empty");
    let count_text = app.i18n.tf(
        "process.count",
        &[&app.processes.processes.len().to_string()],
    );

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::display_title(ui, &title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);
            ui.label(
                RichText::new(ui_kit::tracked(&count_text.to_uppercase()))
                    .font(FontId::new(10.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
        });
    });

    ui.add_space(6.0);
    ui_kit::hairline(ui);

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::search_field(ui, &mut app.process_search, &search_hint);
    });
    ui.add_space(12.0);

    let filtered = filtered_processes(app);

    // 表格里产生的操作先收集，绘制完再落到 app 上。
    let mut intent_select: Option<i32> = None;
    let mut intent_kill: Option<i32> = None;
    let mut intent_copy: Option<String> = None;
    let mut intent_sort: Option<ProcessSort> = None;

    if filtered.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label(
                RichText::new(ui_kit::tracked(&empty_label.to_uppercase()))
                    .font(FontId::new(11.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
        });
    } else {
        // 表格高度不能超过窗口剩余空间，否则底部行会被裁掉且无法滚动。
        let table_height = (ui.max_rect().bottom() - ui.cursor().top() - 22.0).max(80.0);
        let table = egui_extras::TableBuilder::new(ui)
            .striped(false)
            .cell_layout(egui::Layout::left_to_right(Align::Center))
            .max_scroll_height((table_height - 22.0).max(80.0))
            // 固定宽度列：表头与表体才能对齐（`Column::auto` 的宽度在表体阶段才确定，
            // 表头会与表体错位）。
            .column(egui_extras::Column::exact(64.0))
            .column(egui_extras::Column::remainder().at_least(160.0))
            .column(egui_extras::Column::exact(200.0));

        let table = table.header(22.0, |mut header| {
            header.col(|ui| {
                header_button(
                    ui,
                    &pid_label,
                    ProcessSort::Pid,
                    app.process_sort,
                    &mut intent_sort,
                );
            });
            header.col(|ui| {
                header_button(
                    ui,
                    &name_label,
                    ProcessSort::Name,
                    app.process_sort,
                    &mut intent_sort,
                );
            });
            header.col(|ui| {
                header_button(
                    ui,
                    &port_label,
                    ProcessSort::Port,
                    app.process_sort,
                    &mut intent_sort,
                );
            });
        });

        let view_path = app.i18n.t("process.context.viewPath");
        let copy_path = app.i18n.t("process.context.copyPath");
        let kill_label = app.i18n.t("process.context.kill");

        table.body(|body| {
            // 用 `body.rows` 而不是逐行 `body.row`：后者不会为不可见行预留高度，
            // 滚动区会误判为「没有溢出」，列表因此无法滚动。
            body.rows(26.0, filtered.len(), |mut row| {
                let process = &filtered[row.index()];
                let mut row_response: Option<egui::Response> = None;

                row.col(|ui| {
                    ui.label(ui_kit::mono_cell(
                        &process.pid.to_string(),
                        palette.text_muted,
                    ));
                });
                row.col(|ui| {
                    let response = ui
                        .add(
                            egui::Label::new(ui_kit::mono_cell(
                                &process.display_name,
                                palette.text,
                            ))
                            .truncate(),
                        )
                        .on_hover_text(&process.full_path);
                    row_response = Some(response);
                });
                row.col(|ui| {
                    if process.ports.is_empty() {
                        ui.label(ui_kit::mono_cell("-", palette.text_muted));
                    } else {
                        let text = process
                            .ports
                            .iter()
                            .map(|port| port.display())
                            .collect::<Vec<_>>()
                            .join("  ");
                        ui.add(
                            egui::Label::new(ui_kit::mono_cell(&text, palette.accent)).truncate(),
                        );
                    }
                });

                let Some(response) = row_response else {
                    return;
                };

                if response.clicked() {
                    intent_select = Some(process.pid);
                }

                // 右键菜单。
                response.context_menu(|ui| {
                    ui.label(
                        RichText::new(ui_kit::tracked(&process.display_name.to_uppercase()))
                            .font(FontId::new(11.0, FontFamily::Monospace))
                            .color(palette.text),
                    );
                    ui.add_space(4.0);

                    if ui
                        .button(
                            RichText::new(view_path.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_copy = Some(process.full_path.clone());
                    }
                    if ui
                        .button(
                            RichText::new(copy_path.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_copy = Some(process.full_path.clone());
                    }

                    ui.add_space(4.0);
                    ui.separator();

                    let text = kill_label.replace("%@", &process.display_name).replacen(
                        "%@",
                        &process.pid.to_string(),
                        1,
                    );
                    if ui
                        .button(
                            RichText::new(text)
                                .font(FontId::new(12.0, FontFamily::Monospace))
                                .color(palette.danger),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_kill = Some(process.pid);
                    }
                });
            });
        });
    }

    // 落地操作。
    if let Some(sort) = intent_sort {
        app.process_sort = sort;
    }
    if let Some(pid) = intent_select {
        app.selected_pid = Some(pid);
    }
    if let Some(path) = intent_copy {
        crate::platform::copy_to_clipboard(&path);
    }
    if let Some(pid) = intent_kill {
        app.pending_kill = Some(pid);
    }

    render_kill_dialog(ui, app);
}

/// 结束进程的确认弹窗。
fn render_kill_dialog(ui: &mut egui::Ui, app: &mut Killer7App) {
    let Some(pid) = app.pending_kill else {
        return;
    };

    let palette = ui_kit::palette(ui.ctx());
    let name = app
        .processes
        .processes
        .iter()
        .find(|process| process.pid == pid)
        .map(|process| process.display_name.clone())
        .unwrap_or_default();

    let message = app
        .i18n
        .tf("process.kill.confirm", &[&name, &pid.to_string()]);
    let dialog_title = app.i18n.t("process.kill.title");
    let cancel_label = app.i18n.t("common.cancel");
    let confirm_label = app.i18n.t("common.delete");
    let done_template = app.i18n.t("process.kill.done");
    let failed_template = app.i18n.t("process.kill.failed");

    egui::Modal::new(egui::Id::new("killer7.kill"))
        .frame(egui::Frame {
            fill: palette.background,
            stroke: Stroke::new(1.0, palette.border),
            corner_radius: egui::CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        })
        .show(ui.ctx(), |ui| {
            ui_kit::numbered_kicker(ui, "!", &dialog_title);
            ui.add_space(10.0);
            ui.label(
                RichText::new(message)
                    .font(FontId::new(13.0, FontFamily::Proportional))
                    .color(palette.text),
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui_kit::ghost_button(ui, &cancel_label).clicked() {
                    app.pending_kill = None;
                }
                if ui_kit::primary_button(ui, &confirm_label).clicked() {
                    let ok = crate::process::ProcessManager::kill_process(pid);
                    let toast = if ok {
                        done_template.replace("%@", &name)
                    } else {
                        failed_template.clone()
                    };
                    app.notify_toast(toast);
                    app.pending_kill = None;
                    app.refresh_processes();
                }
            });
        });
}

/// 表头按钮，点击切换排序。
fn header_button(
    ui: &mut egui::Ui,
    label: &str,
    sort: ProcessSort,
    current: ProcessSort,
    intent: &mut Option<ProcessSort>,
) {
    let palette = ui_kit::palette(ui.ctx());
    let active = current == sort;

    let response = ui.add(
        egui::Button::new(
            RichText::new(ui_kit::tracked(&label.to_uppercase()))
                .font(FontId::new(9.0, FontFamily::Monospace))
                .color(if active {
                    palette.accent
                } else {
                    palette.text_muted
                }),
        )
        .frame(false),
    );

    if response.clicked() {
        *intent = Some(sort);
    }
}

/// 按搜索词过滤并排序。
fn filtered_processes(app: &Killer7App) -> Vec<ProcessInfo> {
    let keyword = app.process_search.trim().to_lowercase();

    let mut list: Vec<ProcessInfo> = app
        .processes
        .processes
        .iter()
        .filter(|process| {
            if keyword.is_empty() {
                return true;
            }
            process.display_name.to_lowercase().contains(&keyword)
                || process.pid.to_string().contains(&keyword)
                || process
                    .ports
                    .iter()
                    .any(|port| port.port.to_string().contains(&keyword))
        })
        .cloned()
        .collect();

    match app.process_sort {
        ProcessSort::Pid => list.sort_by_key(|process| process.pid),
        ProcessSort::Name => list.sort_by(|a, b| {
            a.display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase())
        }),
        ProcessSort::Port => list.sort_by_key(|process| process.min_port().unwrap_or(u16::MAX)),
    }

    list
}
