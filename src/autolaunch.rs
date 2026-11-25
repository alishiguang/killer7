//! 开机自启动，对应 Swift 版的 `AutoLaunchManager.swift`。
//!
//! Swift 版优先使用 `SMAppService`（macOS 13+），并保留 LaunchAgent 传统实现。
//! Rust 版没有直接的 `SMAppService` 绑定，这里统一走 LaunchAgent 方案：在
//! `~/Library/LaunchAgents` 写入 plist 并通过 `launchctl` 装载。

use std::path::PathBuf;

/// LaunchAgent 的 Label / plist 文件名。
const LABEL: &str = "com.igigi.chuaker7";

/// plist 文件路径。
pub fn plist_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    home.join("Library")
        .join("LaunchAgents")
        .join(format!("{}.plist", LABEL))
}

/// 当前是否处于自启动状态。
pub fn is_enabled() -> bool {
    plist_path().exists()
}

/// 启用开机自启动。
pub fn enable() -> anyhow::Result<()> {
    let path = plist_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let executable = std::env::current_exe()?;
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{program}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
        label = LABEL,
        program = escape_xml(&executable.to_string_lossy())
    );

    std::fs::write(&path, plist)?;

    // 重新装载，让改动立即生效；失败不影响下次登录自启。
    let _ = std::process::Command::new("launchctl")
        .args(["unload", "-w", &path.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::process::Command::new("launchctl")
        .args(["load", "-w", &path.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    Ok(())
}

/// 关闭开机自启动。
pub fn disable() -> anyhow::Result<()> {
    let path = plist_path();
    if path.exists() {
        let _ = std::process::Command::new("launchctl")
            .args(["unload", "-w", &path.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// 转义 XML 特殊字符。
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
