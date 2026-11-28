//! 开机自启动。
//!
//! 各平台用系统自己的「登录时启动」机制：
//!
//! - macOS：LaunchAgent —— 在 `~/Library/LaunchAgents` 写 plist，
//!   **不调用 `launchctl load`**（load 会当场把作业跑起来，等于再启一个实例）；
//! - Windows：注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` ——
//!   直接用系统自带的 `reg.exe` 读写，不引第三方注册表 crate。

pub use backend::{disable, enable, is_enabled};

/// macOS 实现：LaunchAgent plist。
#[cfg(target_os = "macos")]
mod backend {
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

        // **只写 plist，绝不 `launchctl load`。**
        //
        // `launchctl load -w` 会把作业注册进 launchd，而 `RunAtLoad=true` 的语义是
        // 「注册即启动」—— 实测（Task 14 评审）load 完不到 2 秒 ProgramArguments
        // 就被跑起来了。我们的 ProgramArguments 指向应用自身，于是用户在设置里
        // 勾一下「开机自启」，眼前就会多出一个 Ki6 进程：两个托盘图标、两套菜单栏项、
        // 两份后台采样线程，还可能并发写同一个 settings.json。
        //
        // 「开机自启」只需要「下次登录时 launchd 读到 ~/Library/LaunchAgents 里的
        // plist 并拉起它」—— 登录项本来就在这个目录里，写文件就够了。
        // 卸载交给 `disable()` 与登录时的 launchd 接管。
        Ok(())
    }

    /// 关闭开机自启动。
    pub fn disable() -> anyhow::Result<()> {
        let path = plist_path();
        if path.exists() {
            // 作业可能已被 launchd 在登录时自动装载，卸载掉；
            // 未装载时 `unload` 本来就会失败，忽略即可（删掉 plist 就够了）。
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn escapes_xml_specials() {
            assert_eq!(
                escape_xml("a&b<c>d\"e'f"),
                "a&amp;b&lt;c&gt;d&quot;e&apos;f"
            );
        }

        #[test]
        fn plist_path_lives_in_launch_agents() {
            let path = plist_path();
            assert!(path.to_string_lossy().contains("LaunchAgents"));
            assert!(path.to_string_lossy().ends_with("com.igigi.chuaker7.plist"));
        }
    }
}

/// Windows 实现：注册表 Run 键。
#[cfg(target_os = "windows")]
mod backend {
    use std::path::Path;
    use std::process::{Command, Stdio};

    /// 当前用户的登录时启动项。
    const RUN_KEY: &str = r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run";

    /// 值名（注册表里显示的条目名）。
    const VALUE_NAME: &str = "Ki6";

    /// 当前是否处于自启动状态。
    pub fn is_enabled() -> bool {
        Command::new("reg")
            .args(["query", RUN_KEY, "/v", VALUE_NAME])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// 启用开机自启动。
    pub fn enable() -> anyhow::Result<()> {
        let executable = std::env::current_exe()?;
        let status = Command::new("reg")
            .args([
                "add",
                RUN_KEY,
                "/v",
                VALUE_NAME,
                "/t",
                "REG_SZ",
                "/d",
                &run_value(&executable),
                "/f",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            anyhow::bail!("写入注册表 Run 键失败");
        }
        Ok(())
    }

    /// 关闭开机自启动。
    pub fn disable() -> anyhow::Result<()> {
        // 值不存在时 `reg delete` 返回非零 —— 那正是「本来就关着」，不算失败。
        let _ = Command::new("reg")
            .args(["delete", RUN_KEY, "/v", VALUE_NAME, "/f"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        Ok(())
    }

    /// 组装 Run 键的取值。
    ///
    /// 路径带引号是必须的：注册表里的启动项由 Windows 直接当命令行执行，
    /// 路径里有空格（默认的 `C:\Program Files\...`）时不加引号只会执行到第一个空格。
    fn run_value(executable: &Path) -> String {
        format!("\"{}\"", executable.to_string_lossy())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn run_value_is_quoted() {
            assert_eq!(
                run_value(Path::new(r"C:\Program Files\Ki6\Ki6.exe")),
                r#""C:\Program Files\Ki6\Ki6.exe""#
            );
        }
    }
}

#[cfg(test)]
mod tests {
    /// 「开机自启」只能写 plist，**不能**把作业注册进 launchd。
    ///
    /// `launchctl load -w` 配上 `RunAtLoad=true` 的语义是「注册即启动」，
    /// 用户在前端勾一下开关，眼前就会多出一个应用实例（Task 14 评审 C-1）。
    /// 单元测试没法直接观察「没有调用 launchctl」，所以用源码级断言守住这条不变量。
    #[test]
    fn enable_never_loads_the_agent_into_launchd() {
        let source = include_str!("autolaunch.rs");
        assert!(
            !source.contains("\"load\""),
            "enable() 不得调用 `launchctl load`：注册即启动，会再拉起一个应用实例"
        );
        assert!(
            !source.contains("\"bootstrap\""),
            "也不得用 `launchctl bootstrap` 代替 load —— 同样是注册即启动"
        );
    }
}
