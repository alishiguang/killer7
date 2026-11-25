//! 文件管理页，对应 Swift 版的 `FileManagerContentView.swift`。

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use egui::{Align, FontFamily, FontId, Layout, RichText, Stroke};

use crate::app::Chuaker7App;
use crate::files::FileItem;
use crate::ui_kit;

/// 渲染文件管理页。
pub fn render(ui: &mut egui::Ui, app: &mut Chuaker7App) {
    let palette = ui_kit::palette(ui.ctx());

    let title = app.i18n.t("file.title");
    let search_hint = app.i18n.t("file.search.placeholder");
    let name_label = app.i18n.t("file.column.name");
    let size_label = app.i18n.t("file.column.size");
    let modified_label = app.i18n.t("file.column.modified");
    let new_folder = app.i18n.t("file.newFolder");
    let new_file = app.i18n.t("file.newFile");
    let empty_hint = app.i18n.t("file.newFolder.hint");

    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::display_title(ui, &title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(18.0);

            if ui_kit::ghost_button(ui, &new_file).clicked() {
                app.new_file_name = Some(String::new());
            }
            ui.add_space(6.0);
            if ui_kit::primary_button(ui, &new_folder).clicked() {
                app.new_folder_name = Some(String::new());
            }
        });
    });

    ui.add_space(6.0);
    ui_kit::hairline(ui);

    // 路径栏。
    ui.horizontal(|ui| {
        ui.add_space(18.0);

        let up_label = "‹";
        let can_go_up = app.files.current_path.parent() != Some(app.files.current_path.as_path());
        let up = ui.add(
            egui::Button::new(
                RichText::new(up_label)
                    .font(FontId::new(15.0, FontFamily::Monospace))
                    .color(if can_go_up {
                        palette.text
                    } else {
                        palette.text_muted
                    }),
            )
            .fill(palette.surface)
            .stroke(Stroke::new(1.0, palette.border))
            .corner_radius(egui::CornerRadius::same(4)),
        );
        if up.clicked() && can_go_up {
            app.go_up_files();
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new(app.files.current_path.to_string_lossy().to_string())
                .font(FontId::new(12.0, FontFamily::Monospace))
                .color(palette.text_muted),
        );
    });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(18.0);
        ui_kit::search_field(ui, &mut app.file_search, &search_hint);
    });
    ui.add_space(12.0);

    let filtered = filtered_files(app);

    let mut intent_open: Option<PathBuf> = None;
    let mut intent_reveal: Option<PathBuf> = None;
    let mut intent_rename: Option<PathBuf> = None;
    let mut intent_delete: Option<PathBuf> = None;
    let mut intent_copy: Option<String> = None;

    if filtered.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.label(
                RichText::new("◇")
                    .font(FontId::new(30.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(ui_kit::tracked(&empty_hint.to_uppercase()))
                    .font(FontId::new(10.0, FontFamily::Monospace))
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
            .scroll_bar_visibility(
                egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
            )
            // 固定宽度列：表头与表体才能对齐（`Column::auto` 的宽度在表体阶段才确定，
            // 表头会与表体错位）。
            .column(egui_extras::Column::remainder().at_least(180.0))
            .column(egui_extras::Column::exact(90.0))
            .column(egui_extras::Column::exact(150.0));

        let table = table.header(22.0, |mut header| {
            header.col(|ui| {
                ui.label(
                    RichText::new(ui_kit::tracked(&name_label.to_uppercase()))
                        .font(FontId::new(9.0, FontFamily::Monospace))
                        .color(palette.text_muted),
                );
            });
            header.col(|ui| {
                ui.label(
                    RichText::new(ui_kit::tracked(&size_label.to_uppercase()))
                        .font(FontId::new(9.0, FontFamily::Monospace))
                        .color(palette.text_muted),
                );
            });
            header.col(|ui| {
                ui.label(
                    RichText::new(ui_kit::tracked(&modified_label.to_uppercase()))
                        .font(FontId::new(9.0, FontFamily::Monospace))
                        .color(palette.text_muted),
                );
            });
        });

        let reveal_label = app.i18n.t("file.reveal");
        let copy_path_label = app.i18n.t("file.copyPath");
        let rename_label = app.i18n.t("file.rename");
        let delete_label = app.i18n.t("file.delete");
        let open_label = app.i18n.t("file.open");
        let enter_label = app.i18n.t("file.enterFolder");

        table.body(|body| {
            // 用 `body.rows` 而不是逐行 `body.row`：后者不会为不可见行预留高度，
            // 滚动区会误判为「没有溢出」，列表因此无法滚动。
            body.rows(26.0, filtered.len(), |mut row| {
                let item = &filtered[row.index()];
                let mut row_response: Option<egui::Response> = None;

                row.col(|ui| {
                    let icon = if item.is_dir { "▶" } else { "·" };
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(icon)
                                .font(FontId::new(12.0, FontFamily::Monospace))
                                .color(if item.is_dir {
                                    palette.accent
                                } else {
                                    palette.text_muted
                                }),
                        );
                        let response = ui
                            .add(
                                egui::Label::new(ui_kit::mono_cell(&item.name, palette.text))
                                    .truncate(),
                            )
                            .on_hover_text(item.path.to_string_lossy().to_string());
                        row_response = Some(response);
                    });
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(ui_kit::mono_cell(
                            &item.size_string(),
                            palette.text_muted,
                        ))
                        .truncate(),
                    );
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(ui_kit::mono_cell(&format_time(item), palette.text_muted))
                            .truncate(),
                    );
                });

                let Some(response) = row_response else {
                    return;
                };

                // 双击进入 / 打开。
                if response.double_clicked() {
                    if item.is_dir {
                        intent_open = Some(item.path.clone());
                    } else {
                        crate::platform::open_path(&item.path);
                    }
                }

                response.context_menu(|ui| {
                    if item.is_dir {
                        if ui
                            .button(
                                RichText::new(enter_label.clone())
                                    .font(FontId::new(12.0, FontFamily::Monospace)),
                            )
                            .clicked()
                        {
                            ui.close();
                            intent_open = Some(item.path.clone());
                        }
                    } else if ui
                        .button(
                            RichText::new(open_label.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        )
                        .clicked()
                    {
                        ui.close();
                        crate::platform::open_path(&item.path);
                    }

                    ui.add_space(4.0);
                    ui.separator();

                    if ui
                        .button(
                            RichText::new(reveal_label.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_reveal = Some(item.path.clone());
                    }
                    if ui
                        .button(
                            RichText::new(copy_path_label.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_copy = Some(item.path.to_string_lossy().to_string());
                    }

                    ui.add_space(4.0);
                    ui.separator();

                    if ui
                        .button(
                            RichText::new(rename_label.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace)),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_rename = Some(item.path.clone());
                    }
                    if ui
                        .button(
                            RichText::new(delete_label.clone())
                                .font(FontId::new(12.0, FontFamily::Monospace))
                                .color(palette.danger),
                        )
                        .clicked()
                    {
                        ui.close();
                        intent_delete = Some(item.path.clone());
                    }
                });
            });
        });
    }

    // 落地操作。
    if let Some(path) = intent_open {
        app.navigate_files(path);
    }
    if let Some(path) = intent_reveal {
        crate::platform::reveal_in_finder(&path);
    }
    if let Some(path) = intent_copy {
        crate::platform::copy_to_clipboard(&path);
    }
    if let Some(path) = intent_rename {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        app.file_rename = Some((path, name));
    }
    if let Some(path) = intent_delete {
        if app.files.delete_file(&path) {
            let message = app.i18n.t("file.deleted").to_string();
            app.notify_toast(message);
        } else {
            let message = app.i18n.t("file.failed").to_string();
            app.notify_toast(message);
        }
    }

    render_new_folder_dialog(ui, app);
    render_new_file_dialog(ui, app);
    render_rename_dialog(ui, app);
}

/// 新建文件夹弹窗。
fn render_new_folder_dialog(ui: &mut egui::Ui, app: &mut Chuaker7App) {
    let Some(name) = app.new_folder_name.clone() else {
        return;
    };

    let palette = ui_kit::palette(ui.ctx());
    let title = app.i18n.t("file.newFolder.title");
    let placeholder = app.i18n.t("file.newFolder.placeholder");
    let cancel_label = app.i18n.t("common.cancel");
    let create_label = app.i18n.t("common.create");
    let created_template = app.i18n.t("file.created");

    egui::Modal::new(egui::Id::new("chuaker7.new_folder"))
        .frame(egui::Frame {
            fill: palette.background,
            stroke: Stroke::new(1.0, palette.border),
            corner_radius: egui::CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        })
        .show(ui.ctx(), |ui| {
            ui_kit::numbered_kicker(ui, "01", &title);
            ui.add_space(10.0);

            let mut buffer = name;
            ui.add(
                egui::TextEdit::singleline(&mut buffer)
                    .hint_text(placeholder)
                    .desired_width(260.0),
            );

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui_kit::ghost_button(ui, &cancel_label).clicked() {
                    app.new_folder_name = None;
                }
                if ui_kit::primary_button(ui, &create_label).clicked() {
                    let created = app.files.create_folder(buffer.trim());
                    let message = if created {
                        created_template.replace("%@", buffer.trim())
                    } else {
                        app.i18n.t("file.failed").to_string()
                    };
                    app.notify_toast(message);
                    app.new_folder_name = None;
                }
            });
        });
}

/// 新建文件弹窗。
fn render_new_file_dialog(ui: &mut egui::Ui, app: &mut Chuaker7App) {
    let Some(name) = app.new_file_name.clone() else {
        return;
    };

    let palette = ui_kit::palette(ui.ctx());
    let title = app.i18n.t("file.newFile");
    let cancel_label = app.i18n.t("common.cancel");
    let create_label = app.i18n.t("common.create");
    let created_template = app.i18n.t("file.created");

    egui::Modal::new(egui::Id::new("chuaker7.new_file"))
        .frame(egui::Frame {
            fill: palette.background,
            stroke: Stroke::new(1.0, palette.border),
            corner_radius: egui::CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        })
        .show(ui.ctx(), |ui| {
            ui_kit::numbered_kicker(ui, "02", &title);
            ui.add_space(10.0);

            let mut buffer = name;
            ui.add(
                egui::TextEdit::singleline(&mut buffer)
                    .hint_text(title.clone())
                    .desired_width(260.0),
            );

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui_kit::ghost_button(ui, &cancel_label).clicked() {
                    app.new_file_name = None;
                }
                if ui_kit::primary_button(ui, &create_label).clicked() {
                    let created = app.files.create_file(buffer.trim());
                    let message = if created {
                        created_template.replace("%@", buffer.trim())
                    } else {
                        app.i18n.t("file.failed").to_string()
                    };
                    app.notify_toast(message);
                    app.new_file_name = None;
                }
            });
        });
}

/// 重命名弹窗。
fn render_rename_dialog(ui: &mut egui::Ui, app: &mut Chuaker7App) {
    let Some((path, name)) = app.file_rename.clone() else {
        return;
    };

    let palette = ui_kit::palette(ui.ctx());
    let title = app.i18n.t("file.rename.title");
    let hint = app.i18n.t("file.rename.hint");
    let cancel_label = app.i18n.t("common.cancel");
    let confirm_label = app.i18n.t("common.save");
    let renamed_template = app.i18n.t("file.renamed");

    egui::Modal::new(egui::Id::new("chuaker7.rename"))
        .frame(egui::Frame {
            fill: palette.background,
            stroke: Stroke::new(1.0, palette.border),
            corner_radius: egui::CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        })
        .show(ui.ctx(), |ui| {
            ui_kit::numbered_kicker(ui, "03", &title);
            ui.add_space(10.0);

            let mut buffer = name;
            ui.add(
                egui::TextEdit::singleline(&mut buffer)
                    .hint_text(hint)
                    .desired_width(260.0),
            );

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui_kit::ghost_button(ui, &cancel_label).clicked() {
                    app.file_rename = None;
                }
                if ui_kit::primary_button(ui, &confirm_label).clicked() {
                    let target = path.clone();
                    let ok = app.files.rename_file(&target, buffer.trim());
                    let message = if ok {
                        renamed_template.replace("%@", buffer.trim())
                    } else {
                        app.i18n.t("file.failed").to_string()
                    };
                    app.notify_toast(message);
                    app.file_rename = None;
                }
            });
        });
}

/// 按搜索词过滤。
fn filtered_files(app: &Chuaker7App) -> Vec<FileItem> {
    let keyword = app.file_search.trim().to_lowercase();
    if keyword.is_empty() {
        return app.files.files.clone();
    }
    app.files
        .files
        .iter()
        .filter(|item| item.name.to_lowercase().contains(&keyword))
        .cloned()
        .collect()
}

/// 把时间戳格式化成 `YYYY-MM-DD HH:MM`（本地时区）。
fn format_time(item: &FileItem) -> String {
    let secs = item
        .modified
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as libc::time_t)
        .unwrap_or(0);

    // SAFETY: localtime_r 写入调用方提供的 tm 结构，strftime 写入调用方提供的缓冲区。
    unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&secs, &mut tm).is_null() {
            return "-".to_string();
        }

        let mut buffer = [0u8; 64];
        let format = b"%Y-%m-%d %H:%M\0";
        let written = libc::strftime(
            buffer.as_mut_ptr() as *mut libc::c_char,
            buffer.len(),
            format.as_ptr() as *const libc::c_char,
            &tm,
        );
        if written == 0 {
            return "-".to_string();
        }

        let length = buffer.iter().position(|&byte| byte == 0).unwrap_or(0);
        String::from_utf8_lossy(&buffer[..length]).to_string()
    }
}
