//! macOS 平台相关的辅助函数。
//!
//! 对应 Swift 版里直接调用 AppKit 的地方（`NSWorkspace`、`NSPasteboard`）。
//! `open` 与 `arboard` 均为跨平台 crate，无需 objc2 即可完成。

use std::path::Path;

/// 用系统默认程序打开文件或目录。
pub fn open_path(path: &Path) {
    let _ = open::that(path);
}

/// 在 Finder 中定位（选中）文件。
pub fn reveal_in_finder(path: &Path) {
    let _ = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn();
}

/// 复制文本到系统剪贴板。
pub fn copy_to_clipboard(text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => clipboard.set_text(text.to_owned()).is_ok(),
        Err(_) => false,
    }
}

/// 读取系统剪贴板文本。
pub fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut clipboard| clipboard.get_text().ok())
}
