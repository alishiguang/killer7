//! 插件系统，对应 Swift 版的 `PluginSystem.swift`。
//!
//! Swift 版定义了插件协议、清单结构与 `PluginManager`，并把插件标签页 / 命令注册
//! 暴露给插件调用；实际的内容加载仍是 TODO。Rust 版保留了同样的清单格式、目录
//! 约定与注册机制，并把清单信息展示成只读的插件页。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::settings::plugins_dir;

/// 插件清单，对应 Swift 版的 `PluginManifest`。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件唯一标识。
    pub id: String,
    /// 插件名称。
    pub name: String,
    /// 插件版本。
    pub version: String,
    /// 作者。
    #[serde(default)]
    pub author: Option<String>,
    /// 描述。
    #[serde(default)]
    pub description: Option<String>,
    /// 最低兼容的应用版本。
    #[serde(default)]
    pub minimum_app_version: Option<String>,
    /// 申请的权限。
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    /// 入口文件。
    #[serde(default)]
    pub main: Option<String>,
}

/// 插件标签页。
#[derive(Clone, Debug)]
pub struct PluginTab {
    /// 标签页标识。
    pub id: String,
    /// 标签页标题。
    pub title: String,
    /// 标签页图标（emoji）。
    pub icon: String,
}

/// 插件命令。
#[derive(Clone, Debug)]
pub struct PluginCommand {
    /// 命令标识。
    pub id: String,
    /// 命令标题。
    pub title: String,
    /// 命令图标（emoji）。
    pub icon: String,
}

/// 插件管理器。
#[derive(Default)]
pub struct PluginManager {
    /// 已加载的插件清单。
    pub loaded_plugins: Vec<PluginManifest>,
    /// 已注册的插件标签页。
    pub plugin_tabs: Vec<PluginTab>,
    /// 已注册的插件命令。
    pub plugin_commands: Vec<PluginCommand>,
}

impl PluginManager {
    /// 扫描插件目录并加载所有清单。
    pub fn load_plugins(&mut self) {
        let dir = plugins_dir();
        let _ = std::fs::create_dir_all(&dir);

        self.loaded_plugins.clear();
        self.plugin_tabs.clear();
        self.plugin_commands.clear();

        let Ok(entries) = std::fs::read_dir(&dir) else {
            return;
        };

        let mut manifests: Vec<PluginManifest> = Vec::new();

        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            let Ok(text) = std::fs::read_to_string(&manifest_path) else {
                eprintln!("[plugin] 无法读取清单: {}", manifest_path.display());
                continue;
            };
            let Ok(manifest) = serde_json::from_str::<PluginManifest>(&text) else {
                eprintln!("[plugin] 清单格式错误: {}", manifest_path.display());
                continue;
            };

            manifests.push(manifest);
        }

        manifests.sort_by(|a, b| a.id.cmp(&b.id));

        for manifest in &manifests {
            self.register_tab(PluginTab {
                id: format!("plugin.{}", manifest.id),
                title: manifest.name.clone(),
                icon: "◆".to_string(),
            });
            self.register_command(PluginCommand {
                id: format!("{}.open", manifest.id),
                title: manifest.name.clone(),
                icon: "◆".to_string(),
            });
        }

        self.loaded_plugins = manifests;
    }

    /// 注册标签页，重复 id 会被忽略。
    pub fn register_tab(&mut self, tab: PluginTab) {
        if !self.plugin_tabs.iter().any(|t| t.id == tab.id) {
            self.plugin_tabs.push(tab);
        }
    }

    /// 注册命令，重复 id 会被忽略。
    pub fn register_command(&mut self, command: PluginCommand) {
        if !self.plugin_commands.iter().any(|c| c.id == command.id) {
            self.plugin_commands.push(command);
        }
    }

    /// 插件目录路径。
    pub fn plugins_dir(&self) -> PathBuf {
        plugins_dir()
    }
}
