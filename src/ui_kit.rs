//! 界面设计基元。
//!
//! 视觉语言参考 cindy.cn：
//!
//! * 近黑（ink）底色 + 奶油色文字 + 单一强调色；
//! * 发丝级描边（1px 低透明度）而不是投影；
//! * 全大写等宽小标题（kicker）配 `// 01` 这样的编号；
//! * 直角偏小圆角的卡片，配渐隐的红色辉光。
//!
//! 这里把配色放进 `egui::Context` 的内存里，任意控件都能取到，避免层层传参。

use egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, Layout, RichText, Stroke, TextStyle, Ui, Vec2,
};

use crate::theme::Palette;

/// 配色在 `egui::Context` 内存中的键。
fn palette_key() -> egui::Id {
    egui::Id::new("killer7.palette")
}

/// 写入当前配色。
pub fn set_palette(ctx: &egui::Context, palette: Palette) {
    ctx.memory_mut(|memory| memory.data.insert_temp(palette_key(), palette));
}

/// 读取当前配色。
pub fn palette(ctx: &egui::Context) -> Palette {
    ctx.memory(|memory| {
        memory
            .data
            .get_temp::<Palette>(palette_key())
            .unwrap_or_default()
    })
}

/// 在字符之间插入细空格，得到参考站那种宽字距的全大写排版。
pub fn tracked(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for (index, ch) in text.chars().enumerate() {
        if index > 0 {
            out.push('\u{2009}');
        }
        out.push(ch);
    }
    out
}

/// 等宽小标题：全大写 + 宽字距 + 次要色。
pub fn kicker(ui: &mut Ui, text: &str) {
    let palette = palette(ui.ctx());
    ui.label(
        RichText::new(tracked(&text.to_uppercase()))
            .font(FontId::new(10.0, FontFamily::Monospace))
            .color(palette.text_muted),
    );
}

/// 编号小标题，例如 `// 01 · TOOLS`。
pub fn numbered_kicker(ui: &mut Ui, index: &str, text: &str) {
    let palette = palette(ui.ctx());
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("// {}", index))
                .font(FontId::new(10.0, FontFamily::Monospace))
                .color(palette.accent),
        );
        ui.label(
            RichText::new(tracked(&text.to_uppercase()))
                .font(FontId::new(10.0, FontFamily::Monospace))
                .color(palette.text_muted),
        );
    });
}

/// 展示级标题（大号重体）。
pub fn display_title(ui: &mut Ui, text: &str) {
    let palette = palette(ui.ctx());
    ui.label(
        RichText::new(text.to_uppercase())
            .font(display_font(30.0))
            .color(palette.text),
    );
}

/// 展示级字体；没有内置 Anton 时回退到系统粗体。
pub fn display_font(size: f32) -> FontId {
    #[cfg(has_display_font)]
    {
        FontId::new(size, FontFamily::Name("display".into()))
    }
    #[cfg(not(has_display_font))]
    {
        FontId::new(size, FontFamily::Proportional)
    }
}

/// 卡片：surface 底色 + 1px 发丝描边 + 小圆角。
pub fn card(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    let palette = palette(ui.ctx());
    let frame = egui::Frame {
        fill: palette.surface,
        stroke: Stroke::new(1.0, palette.border),
        corner_radius: CornerRadius::same(6),
        inner_margin: egui::Margin::same(14),
        ..Default::default()
    };
    frame.show(ui, content);
}

/// 发丝分隔线。
pub fn hairline(ui: &mut Ui) {
    let palette = palette(ui.ctx());
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, palette.border);
}

/// 强调色主按钮。
pub fn primary_button(ui: &mut Ui, label: &str) -> egui::Response {
    let palette = palette(ui.ctx());
    let text = RichText::new(label)
        .font(FontId::new(12.0, FontFamily::Monospace))
        .color(palette.on_accent);

    ui.add(
        egui::Button::new(text)
            .fill(palette.accent)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(4))
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

/// 描边次按钮。
pub fn ghost_button(ui: &mut Ui, label: &str) -> egui::Response {
    let palette = palette(ui.ctx());
    let text = RichText::new(label)
        .font(FontId::new(12.0, FontFamily::Monospace))
        .color(palette.text);

    ui.add(
        egui::Button::new(text)
            .fill(palette.surface_alt)
            .stroke(Stroke::new(1.0, palette.border))
            .corner_radius(CornerRadius::same(4))
            .min_size(Vec2::new(0.0, 28.0)),
    )
}

/// 搜索框：放大镜 + 纯文本输入 + 清除按钮。
pub fn search_field(ui: &mut Ui, text: &mut String, hint: &str) {
    let palette = palette(ui.ctx());

    let frame = egui::Frame {
        fill: palette.control,
        stroke: Stroke::new(1.0, palette.border),
        corner_radius: CornerRadius::same(4),
        inner_margin: egui::Margin::symmetric(10, 6),
        ..Default::default()
    };

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // 放大镜用 painter 画，避免依赖字体里的特殊字符。
            let (rect, _) = ui.allocate_exact_size(Vec2::new(13.0, 13.0), egui::Sense::hover());
            let painter = ui.painter();
            let center = rect.center() - Vec2::new(1.2, 1.6);
            let radius = 4.0;
            let stroke = Stroke::new(1.2, palette.text_muted);
            painter.circle_stroke(center, radius, stroke);
            painter.line_segment(
                [
                    center + Vec2::new(radius * 0.7, radius * 0.7),
                    center + Vec2::new(radius * 1.5, radius * 1.5),
                ],
                stroke,
            );

            let response = ui.add(
                egui::TextEdit::singleline(text)
                    .hint_text(RichText::new(hint).color(palette.text_muted))
                    .frame(false)
                    .desired_width(f32::INFINITY),
            );

            if !text.is_empty() {
                let clear = ui.add(
                    egui::Button::new(
                        RichText::new("×")
                            .font(FontId::new(13.0, FontFamily::Monospace))
                            .color(palette.text_muted),
                    )
                    .frame(false),
                );
                if clear.clicked() {
                    text.clear();
                }
            }

            response
        });
    });
}

/// 固定宽度的卡片。
///
/// 与 [`card`] 的区别是宽度完全可控：`egui::Frame` 的宽度由内容推导，
/// 在需要三等分布局时容易溢出，这里直接分配精确的矩形再自行绘制背景。
pub fn fixed_card(
    ui: &mut Ui,
    width: f32,
    height: f32,
    content: impl FnOnce(&mut Ui),
) -> egui::Response {
    let palette = palette(ui.ctx());

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());

    let radius = CornerRadius::same(6);
    ui.painter().rect_filled(rect, radius, palette.surface);
    ui.painter().rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, palette.border),
        egui::StrokeKind::Inside,
    );

    let mut inner = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(14.0))
            .layout(Layout::top_down(Align::Min)),
    );
    content(&mut inner);

    response
}

/// 指标块：标签 + 大号数值 + 单位。
pub fn stat_block(ui: &mut Ui, label: &str, value: &str, unit: &str, color: Color32) {
    let palette = palette(ui.ctx());

    ui.vertical(|ui| {
        kicker(ui, label);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(value)
                    .font(FontId::new(26.0, FontFamily::Monospace))
                    .color(color),
            );
            ui.label(
                RichText::new(unit)
                    .font(FontId::new(12.0, FontFamily::Monospace))
                    .color(palette.text_muted),
            );
        });
    });
}

/// 迷你折线图。
pub fn sparkline(ui: &mut Ui, data: &[f64], max_value: f64, color: Color32) {
    let palette = palette(ui.ctx());
    let height = 64.0;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );

    let painter = ui.painter();

    // 基线。
    painter.rect_filled(
        egui::Rect::from_min_size(rect.left_top(), Vec2::new(rect.width(), 1.0)),
        0.0,
        palette.border,
    );

    if data.len() < 2 {
        return;
    }

    let peak = max_value.max(1.0);
    let points: Vec<egui::Pos2> = data
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = rect.left() + rect.width() * (index as f32 / (data.len() - 1) as f32);
            let ratio = (value / peak).clamp(0.0, 1.0) as f32;
            let y = rect.bottom() - rect.height() * ratio;
            egui::pos2(x, y)
        })
        .collect();

    // 面积。
    let mut fill = points.clone();
    fill.push(egui::pos2(rect.right(), rect.bottom()));
    fill.push(egui::pos2(rect.left(), rect.bottom()));
    painter.add(egui::Shape::convex_polygon(
        fill,
        color.gamma_multiply(0.12),
        egui::Stroke::NONE,
    ));

    // 折线。
    painter.add(egui::Shape::line(points, Stroke::new(1.6, color)));
}

/// 进度条：细横条 + 强调色填充。
pub fn meter(ui: &mut Ui, ratio: f32, color: Color32) {
    let palette = palette(ui.ctx());
    let height = 6.0;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(3), palette.surface_alt);

    let filled = egui::Rect::from_min_size(
        rect.left_top(),
        Vec2::new(rect.width() * ratio.clamp(0.0, 1.0), height),
    );
    painter.rect_filled(filled, CornerRadius::same(3), color);
}

/// 顶部标题栏：品牌字 + 标语。
pub fn brand(ui: &mut Ui, brand: &str, tagline: &str) {
    let palette = palette(ui.ctx());

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(tracked(brand))
                .font(display_font(21.0))
                .color(palette.text),
        );
        ui.label(
            RichText::new(tracked(&tagline.to_uppercase()))
                .font(FontId::new(9.0, FontFamily::Monospace))
                .color(palette.text_muted),
        );
    });
}

/// 顶部标题栏右侧的红色辉光，呼应参考站 hero 区的径向渐变。
pub fn accent_glow(ui: &mut Ui) {
    let palette = palette(ui.ctx());
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 0.0), egui::Sense::hover());

    // 用几层递减透明度的圆叠出柔和辉光，避免依赖纹理 API。
    let center = egui::pos2(rect.right() - 120.0, rect.center().y);
    let radius = 90.0;

    for step in 0..6 {
        let ratio = 1.0 - step as f32 / 6.0;
        let alpha = 0.012 * (1.0 - ratio);
        ui.painter().add(egui::Shape::circle_filled(
            center,
            radius * ratio,
            palette.accent.gamma_multiply(alpha),
        ));
    }
}

/// 侧边栏条目：编号 + 名称，选中时左侧有强调色竖条。
pub fn nav_item(ui: &mut Ui, index: &str, label: &str, active: bool) -> egui::Response {
    let palette = palette(ui.ctx());

    let text_color = if active {
        palette.text
    } else {
        palette.text_muted
    };
    let label_text = RichText::new(tracked(&label.to_uppercase()))
        .font(FontId::new(11.0, FontFamily::Monospace))
        .color(text_color);
    let index_text = RichText::new(index)
        .font(FontId::new(10.0, FontFamily::Monospace))
        .color(if active {
            palette.accent
        } else {
            palette.text_muted
        });

    let response = ui
        .horizontal(|ui| {
            ui.add_space(2.0);
            ui.label(index_text);
            ui.add_space(8.0);
            ui.label(label_text);
        })
        .response;

    if active {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(2.0, 0.0), egui::Sense::hover());
        let _ = rect;
        let bar = egui::Rect::from_min_size(
            ui.painter().clip_rect().left_top(),
            egui::vec2(2.0, ui.spacing().interact_size.y),
        );
        ui.painter().rect_filled(bar, 0.0, palette.accent);
    }

    response
}

/// 键值行：左侧固定宽度的小标题，右侧等宽文本。
///
/// 用来替代 `egui::Grid`——`Grid` 首次出现时会请求丢弃整帧（egui 的多遍布局），
/// 导致页面打开的第一帧是空白。手动排版更可控，也更贴合这里的对齐风格。
pub fn key_value(ui: &mut Ui, label_width: f32, label: &str, value: &str) {
    let palette = palette(ui.ctx());

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(label_width, ui.spacing().interact_size.y),
            Layout::left_to_right(Align::Center),
            |ui| {
                kicker(ui, label);
            },
        );
        ui.label(
            RichText::new(value)
                .font(FontId::new(11.0, FontFamily::Monospace))
                .color(palette.text_muted),
        );
    });
}

/// 表格正文的等宽单元格样式。
pub fn mono_cell(text: &str, color: Color32) -> RichText {
    RichText::new(text)
        .font(FontId::new(12.0, FontFamily::Monospace))
        .color(color)
}

/// 全局文本样式。
pub fn apply_text_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style
        .text_styles
        .insert(TextStyle::Heading, display_font(22.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(13.0, FontFamily::Proportional));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(12.0, FontFamily::Monospace));
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(12.0, FontFamily::Monospace),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, FontFamily::Proportional),
    );

    ctx.set_style(style);
}
