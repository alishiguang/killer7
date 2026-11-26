# chuaker7 插件系统架构设计

本文档移植自 Swift 版 `PluginSystem.swift` 顶部的架构说明，并根据 Rust 实现做了对应调整。

## 概述

chuaker7 的插件系统允许第三方开发者扩展应用功能，同时保持核心代码的稳定性和安全性。

## 架构设计

### 1. 插件类型

#### A. 原生动态库插件（推荐）

- 通过 `cdylib` 分发
- 编译时集成
- 适合复杂功能

#### B. 独立进程插件（备选）

- 主应用独立运行
- 适合独立功能模块
- 需要独立签名

#### C. 脚本插件（简易扩展）

- 运行时加载
- 适合轻量级脚本

### 2. 核心接口设计

Rust 侧用 trait 表达插件契约：

```rust
/// 插件主 trait。
pub trait Plugin: Send + Sync {
    /// 插件唯一标识符。
    fn id(&self) -> &str;

    /// 插件显示名称。
    fn name(&self) -> &str;

    /// 插件版本。
    fn version(&self) -> &str;

    /// 插件描述。
    fn description(&self) -> &str;

    /// 初始化插件。
    fn initialize(&mut self, context: &mut PluginContext) -> anyhow::Result<()>;

    /// 终止插件。
    fn terminate(&mut self);
}

/// 插件上下文。
pub struct PluginContext<'a> {
    pub plugin_manager: &'a mut PluginManager,
    pub logger: PluginLogger,
}

impl PluginContext<'_> {
    /// 注册插件标签页。
    pub fn register_tab(&mut self, tab: PluginTab) -> anyhow::Result<()> {
        self.plugin_manager.register_tab(tab);
        Ok(())
    }

    /// 注册插件命令。
    pub fn register_command(&mut self, command: PluginCommand) -> anyhow::Result<()> {
        self.plugin_manager.register_command(command);
        Ok(())
    }
}

/// 插件标签页。
pub struct PluginTab {
    pub id: String,
    pub title: String,
    pub icon: String,
}

/// 插件命令。
pub struct PluginCommand {
    pub id: String,
    pub title: String,
    pub icon: String,
}
```

### 3. 插件生命周期

1. **发现 (Discovery)**
   - 扫描 `~/Library/Application Support/chuaker7/Plugins` 目录
   - 读取插件清单 (`manifest.json`)
   - 验证插件签名

2. **加载 (Loading)**
   - 解析插件元数据
   - 创建插件实例
   - 调用 `initialize()`

3. **运行 (Running)**
   - 注册 UI 组件
   - 响应用户交互
   - 发送事件通知

4. **卸载 (Unloading)**
   - 调用 `terminate()`
   - 清理 UI 组件
   - 释放资源

### 4. 安全性设计

- **代码签名验证**：所有插件必须签名
- **沙箱隔离**：插件运行在受限环境
- **权限控制**：插件需要声明所需权限
- **安全更新**：签名验证更新包

### 5. 目录结构

```
chuaker7/
└── Plugins/                    # 插件目录
    ├── manifest.json           # 插件清单
    └── MyPlugin/
        ├── manifest.json       # 插件元数据
        ├── libmyplugin.dylib   # 插件实现
        └── Resources/          # 资源文件
```

### 6. 插件清单格式

```json
{
  "id": "com.example.myplugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Example Author",
  "description": "A sample plugin for chuaker7",
  "minimum_app_version": "1.1.0",
  "permissions": ["file_access", "network"],
  "main": "libmyplugin.dylib"
}
```

Rust 侧对应的结构体定义在 `src/plugin.rs` 的 `PluginManifest`：

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub minimum_app_version: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub main: Option<String>,
}
```

### 7. 当前实现状态

| 阶段 | 状态 |
| --- | --- |
| 清单扫描与解析 | ✅ 已实现（`PluginManager::load_plugins`） |
| 标签页 / 命令注册 | ✅ 已实现（`register_tab` / `register_command`） |
| 动态库加载与 `initialize()` | ⏳ 待实现 |

与 Swift 原版保持一致：目前只完成到「发现 + 注册」阶段，实际的插件代码加载仍是
后续工作。插件清单信息会以只读形式展示在插件页面上。

## 实现计划

- **Phase 1（v1.2）**：插件协议定义、插件管理器、基础加载机制
- **Phase 2（v1.3）**：标签页注册系统、菜单扩展、命令注册
- **Phase 3（v1.4）**：代码签名验证、权限控制系统、插件商店集成
