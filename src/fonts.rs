//! 字体装载。
//!
//! 三类字体：
//!
//! * 展示体 `Anton` —— 宽字距全大写的标题字，缺失时回退系统粗体；
//! * 等宽体 `Geist Mono` —— 数据与标签，缺失时回退系统等宽体；
//! * 中文字体 —— 从 macOS 系统字体中挑选一个可用的中文字体（PingFang /
//!   Hiragino Sans GB / STHeiti），避免把十几 MB 的字库打进仓库。
//!
//! 中文字体作为各字族的**兜底**字体加入：拉丁字符仍由前面的字体绘制，
//! 只有缺字时才落到中文字体。

use egui::{FontData, FontDefinitions, FontFamily, FontTweak};

/// 系统中文字体候选，按优先级排列，括号内是 TTC 集合内的字面索引。
const CJK_CANDIDATES: &[(&str, u32)] = &[
    ("/System/Library/Fonts/PingFang.ttc", 0),
    ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
    ("/System/Library/Fonts/STHeiti Light.ttc", 0),
    ("/System/Library/Fonts/STHeiti Medium.ttc", 0),
    ("/System/Library/Fonts/Supplemental/Songti.ttc", 0),
    ("/Library/Fonts/Arial Unicode.ttf", 0),
];

/// 安装全部字体。
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    install_display(&mut fonts);
    install_mono(&mut fonts);
    install_cjk(&mut fonts);

    ctx.set_fonts(fonts);
}

/// 安装展示体。
fn install_display(fonts: &mut FontDefinitions) {
    #[cfg(has_display_font)]
    {
        let bytes = include_bytes!(concat!("../assets/fonts/", "Anton-Regular.ttf"));
        fonts
            .font_data
            .insert("display".to_string(), FontData::from_static(bytes).into());
        fonts.families.insert(
            FontFamily::Name("display".into()),
            vec!["display".to_string()],
        );
    }
    #[cfg(not(has_display_font))]
    {
        let _ = fonts;
    }
}

/// 安装等宽体。
fn install_mono(fonts: &mut FontDefinitions) {
    #[cfg(has_mono_font)]
    {
        let bytes = include_bytes!(concat!("../assets/fonts/", "GeistMono-Variable.ttf"));
        fonts
            .font_data
            .insert("geistmono".to_string(), FontData::from_static(bytes).into());
        if let Some(list) = fonts.families.get_mut(&FontFamily::Monospace) {
            list.insert(0, "geistmono".to_string());
        }
    }
    #[cfg(not(has_mono_font))]
    {
        let _ = fonts;
    }
}

/// 安装中文字体，作为所有字族的兜底。
fn install_cjk(fonts: &mut FontDefinitions) {
    let Some((path, index)) = find_cjk_font() else {
        eprintln!("[fonts] 未找到可用的中文字体，中文可能显示为方块");
        return;
    };

    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };

    let data = FontData {
        index,
        tweak: FontTweak {
            // 中文字体的 em 普遍偏大，缩一点让中英文基线更协调。
            scale: 0.95,
            y_offset: 0.0,
            y_offset_factor: 0.0,
        },
        ..FontData::from_owned(bytes)
    };

    fonts.font_data.insert("cjk".to_string(), data.into());

    // 加到每个字族的末尾，只在缺字时才生效。
    for family in [
        FontFamily::Proportional,
        FontFamily::Monospace,
        FontFamily::Name("display".into()),
    ] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("cjk".to_string());
    }
}

/// 在候选路径与系统资源目录里寻找中文字体。
fn find_cjk_font() -> Option<(String, u32)> {
    for (path, index) in CJK_CANDIDATES {
        if std::path::Path::new(path).exists() {
            return Some((path.to_string(), *index));
        }
    }

    // 新版 macOS 把 PingFang 放在带版本号的资源目录里，路径不固定，
    // 这里做一次浅层扫描。
    let assets = std::path::Path::new("/System/Library/AssetsV2");
    if let Ok(entries) = std::fs::read_dir(assets) {
        for entry in entries.flatten() {
            let dir = entry.path();
            let Ok(inner) = std::fs::read_dir(&dir) else {
                continue;
            };
            for asset in inner.flatten() {
                let candidate = asset.path().join("AssetData/PingFang.ttc");
                if candidate.exists() {
                    return Some((candidate.to_string_lossy().to_string(), 0));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_font_is_discoverable_on_macos() {
        // 中文界面依赖系统字体，缺失时只应警告而不应 panic。
        let found = find_cjk_font();
        assert!(found.is_some(), "macOS 上应当能找到中文字体");
    }

    #[test]
    fn install_does_not_panic() {
        let ctx = egui::Context::default();
        install(&ctx);
    }
}

/// 界面中用到的非 ASCII 符号，必须在**所有**字族下都有字形。
///
/// 展示体（Anton）只含拉丁字形，所以这里的字符也必须能从中文兜底字体取到。
#[cfg_attr(not(test), allow(dead_code))]
pub const UI_SYMBOLS: &[char] = &['©', '·', '—', '‹', '▶', '◆', '◇', '○', '●', '×', '，'];

/// 只在正文 / 等宽字体下使用的符号。
///
/// 这些字符在展示体里没有字形，但使用处不会落到展示体上。
#[cfg_attr(not(test), allow(dead_code))]
pub const UI_SYMBOLS_TEXT: &[char] = &['⌘', '⎘'];

#[cfg(test)]
mod glyph_tests {
    use super::*;
    use egui::{FontFamily, FontId};

    /// 每个界面符号在等宽与展示字体下都应可渲染。
    #[test]
    fn ui_symbols_have_glyphs() {
        let ctx = egui::Context::default();
        install(&ctx);

        // 字体图集要等第一次 run 之后才会建立。
        let _ = ctx.run(egui::RawInput::default(), |_| {});

        let mut missing = Vec::new();

        ctx.fonts_mut(|fonts| {
            for family in [
                FontFamily::Proportional,
                FontFamily::Monospace,
                FontFamily::Name("display".into()),
            ] {
                let font_id = FontId::new(12.0, family.clone());
                for symbol in UI_SYMBOLS {
                    if !fonts.has_glyph(&font_id, *symbol) {
                        missing.push(format!("{:?} in {:?}", symbol, family));
                    }
                }

                // 只在正文 / 等宽字体下使用的符号。
                if !matches!(family, FontFamily::Name(_)) {
                    for symbol in UI_SYMBOLS_TEXT {
                        if !fonts.has_glyph(&font_id, *symbol) {
                            missing.push(format!("{:?} in {:?}", symbol, family));
                        }
                    }
                }
            }
        });

        assert!(missing.is_empty(), "缺少字形的符号: {}", missing.join(", "));
    }
}
