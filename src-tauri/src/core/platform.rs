//! 平台相关的辅助函数。
//!
//! `open` 与 `arboard` 均为跨平台 crate，无需平台分支即可完成；
//! 只有「在文件管理器中定位」需要按平台分派。

use std::path::Path;

/// 用系统默认程序打开文件或目录。
pub fn open_path(path: &Path) {
    let _ = open::that(path);
}

/// 在文件管理器中定位（选中）文件。
#[cfg(target_os = "macos")]
pub fn reveal_in_file_manager(path: &Path) {
    let _ = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn();
}

/// 在资源管理器中定位（选中）文件。
#[cfg(target_os = "windows")]
pub fn reveal_in_file_manager(path: &Path) {
    // explorer 的 `/select,` 与路径必须连成一个参数；路径里有空格时还要自己带引号，
    // 否则 explorer 会把路径截断在第一个空格处。
    let arg = format!("/select,\"{}\"", path.display());
    let _ = std::process::Command::new("explorer").arg(arg).spawn();
}

/// 在文件管理器中定位（选中）文件。
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn reveal_in_file_manager(path: &Path) {
    // 其余 Unix 系统没有统一的「在文件管理器中选中」接口，退化成打开所在目录。
    if let Some(parent) = path.parent() {
        let _ = open::that(parent);
    }
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
