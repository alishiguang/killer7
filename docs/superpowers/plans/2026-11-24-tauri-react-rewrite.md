# chuaker7 Tauri v2 + React 重写实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 chuaker7 从 egui 立即模式重写为 Tauri v2 + React 19 + TypeScript 前端架构，保留 cindy.cn 的 ink/paper 视觉语言与全部业务功能，同时用 macOS 原生能力（应用菜单栏、原生右键菜单、托盘菜单、原生对话框）替换自绘交互层，消除首帧空白与滚动掉帧。

**Architecture:** Rust 侧保留全部业务逻辑（`core/`），作为唯一权威状态源；前端只持有渲染快照，搜索与排序在本地完成。Rust 通过 Tauri `invoke`（命令）与 `emit`（事件）与前端通信；原生能力一律走 Tauri 官方 API，仅「切换 NSApp 激活策略」一处使用 `objc2-app-kit` 直调 AppKit。egui 层（`app.rs`/`ui_kit.rs`/`theme.rs` 的 egui 部分/`fonts.rs`/`views/*`）整体废弃。

**Tech Stack:** Tauri v2.12.7（Rust，WKWebView）/ React 19.3.1 / TypeScript 5.9.3 / Vite 7.1.5 / vitest 3.2.7 / Rust 侧 serde + sysinfo + notify + trash + arboard + global-hotkey + objc2-app-kit

---

## Global Constraints

以下约束适用于**每一个**任务，任何任务都不得违反：

- **平台**：仅 macOS，不做跨平台适配，不写 Windows/Linux 分支。
- **Rust 工具链**：使用本机现有 rustc 1.90（`rustc --version` 实测 1.90.0），**不得升级全局工具链**。所有 crate 版本必须能在 1.90 上编译通过（已实测的版本见下方「已验证事实」）。
- **Node 工具链**：Vite 7 要求 Node `^20.19.0 || >=22.12.0`。本机默认 node 为 v22.11.0 **不满足**，必须用 nvm 切到 v24.17.0。仓库根放 `.nvmrc`（内容 `24.17.0`）并在此基础上安装依赖。
- **依赖白名单**：前端只允许 `react` / `react-dom` / `@tauri-apps/api` / `@tauri-apps/plugin-dialog` 四个运行时依赖。**禁止**引入状态管理库、虚拟列表库、UI 组件库、CSS 框架、日期库、图表库。开发依赖仅允许 `vite` / `@vitejs/plugin-react` / `typescript` / `vitest` / `jsdom` / `@testing-library/react` / `@testing-library/jest-dom` / `@types/react` / `@types/react-dom` / `@tauri-apps/cli`。
- **Rust 依赖白名单**：`tauri` / `tauri-build` / `tauri-plugin-dialog` / `tauri-plugin-opener` / `objc2` / `objc2-app-kit` / `serde` / `serde_json` / `dirs` / `anyhow` / `arboard` / `sysinfo` / `notify` / `trash` / `open` / `global-hotkey`。**不得**引入 `eframe` / `egui` / `egui_extras`；也**不得直接**引入 `tray-icon` crate（Tauri 的 `tray-icon` feature 会自带它，重复引入会造成类型不匹配）。
- **视觉令牌（不得改动）**：ink 底 `#0F0E15`、ink 面板 `#16151E`、ink 面板二 `#201F2A`、ink 控件 `#24222F`、paper 底 `#FAF6EE`、paper 面板 `#F3EDE2`、paper 面板二 `#EAE2D2`、paper 控件 `#FFFFFF`；正文 ink `#F3EDE2` / paper `#14131B`；次要文字 ink `#918F9E` / paper `#5D5B66`；强调色 signal `#F43D3F`、haze `#A9C6E8`、moss `#7FB069`、amber `#E9B44C`（浅色模式下 signal `#CF2B30`、haze `#4A6FA5`、moss `#3F6B4A`、amber `#9A6B12`）；发丝描边 ink `rgba(243,237,226,0.08)` / paper `rgba(20,19,27,0.18)`；圆角 ≤ 5px。
- **字体**：`assets/fonts/Anton-Regular.ttf`（展示字体，宽字距）+ `assets/fonts/GeistMono-Variable.ttf`（等宽小标题），均为 SIL OFL，许可证文件 `assets/fonts/LICENSE-Anton.txt` / `LICENSE-GeistMono.txt` 必须随字体一起使用。中文走系统字体栈（PingFang SC / Hiragino Sans GB / Songti SC），**不得**为中文引入 Web 字体。
- **文案**：`locales/zh-Hans.lproj/Localizable.strings` 与 `locales/en.lproj/Localizable.strings` 是唯一文案来源，**不新增、不删除 key**，只在前端按需读取。占位符沿用 `%@` 顺序填充。
- **提交规范**：提交信息用中文，正文末尾附
  ```
  AI-Coding: true
  AI-Tool: TophantAI
  ```
  两者缺一不可，且不得重复追加。
- **删除文件**：一律使用 `"/Users/lishiguang/.minimax/bin/mavis-trash" -- "<path>"`，**不得**使用 `rm`（会被权限网关拦截）。
- **不泄露凭证**：推送 GitHub 用 `GIT_ASKPASS` 注入临时脚本，推送完立即用 `mavis-trash` 删除脚本；任何提交、日志、计划文件中不得出现 PAT 明文。
- **性能红线**：进程列表 500 行时 DOM 节点数不超过 60（虚拟列表，固定行高 26px，overscan 8）；滚动不得掉帧；首帧不得出现空白（WebView 直接渲染 HTML，不存在 egui 的整帧丢弃）。
- **构建前置条件（重要）**：`src-tauri/Cargo.toml` 的 `default = ["custom-protocol"]` 让 `tauri::generate_context!` 在编译期**无条件**校验 `frontendDist`（即 `frontend/dist`）目录存在，而该目录由前端构建产出且被 `.gitignore` 的 `dist/` 规则忽略。因此：
  - **干净检出后直接 `cargo build` / `cargo test` 必然失败**（报 `The frontendDist configuration is set to "../frontend/dist" but this path doesn't exist`），这是 Tauri v2 项目的固有形态，**不是缺陷**。
  - 所有 Rust 命令（`cargo build` / `cargo test` / `cargo clippy`）必须在**已跑过 `npm run build`** 的工作树里执行。
  - **任何 release 构建（`tauri build`）之前，必须确认 `frontend/dist/index.html` 是真实前端产物，而不是占位文件**——否则空白首页会被内嵌进二进制，直接违反「首帧不得出现空白」的性能红线。
  - 占位文件（若需要临时创建以让 Rust 侧先跑起来）必须位于被 gitignore 的路径下，不得提交。
- **分支策略**：`main` 继续演进；egui 版永久存档在 `legacy-egui` 分支（指向 `b36b8dc`），**不得删除或改写该分支**。
- **命名约定（重要）**：GitHub **仓库名是 `chuaker7/Ki6`**（远端 `https://github.com/chuaker7/Ki6.git`）；而**包名 / crate 名 / 类型名 / 数据目录 / LaunchAgent 标签 / 应用显示名（`productName`）/ 本地化 key 等一切标识符统一为 `chuaker7`**。两者**刻意不同**，任何任务都不得"顺手统一"或把 `chuaker7` 改成 `Ki6`（反之亦然）。**README 的标题与项目介绍用 `Ki6`**（Task 27 统一改写 README 时落实），但 README 正文描述"应用本体"时仍用 `chuaker7`。

---

## 已验证事实（实测结论，禁止重复推断）

以下结论均已在本机实际跑过，后续任务直接引用，不要重新试错：

| 事实 | 结论 |
| --- | --- |
| Tauri 版本编译性 | `tauri 2.12.7` + `tauri-build 2.7.6` + `tauri-plugin-dialog 2.8.4` + `tauri-plugin-opener 2.7.3` + `objc2 0.6` + `objc2-app-kit 0.3` 在 rustc 1.90 上**实测编译通过**（36.61s 全新构建）。 |
| `macOSPrivateApi` | `tauri.conf.json` 的 `app.macOSPrivateApi: true` 必须与 Cargo 的 `macos-private-api` feature **同时存在**。只改一边会报 feature / allowlist 不匹配。 |
| 图标 | `tauri::generate_context!` 要求 `src-tauri/icons/icon.png` 存在且为 **RGBA**（纯 RGB 会报 `is not RGBA`）。 |
| `beforeDevCommand` 工作目录 | 实测为**仓库根目录**（`src-tauri/tauri.conf.json` + `frontend/` 布局下，cwd = 仓库根）。因此命令写成 `npm --prefix frontend run dev`，npm 脚本自身 cwd = `frontend/`。 |
| `frontendDist` 解析 | 相对 `src-tauri/` 目录解析。仓库根有 `frontend/dist` 时应写 `"../frontend/dist"`。 |
| Vite `root` | `root: 'frontend'` 时 `build.outDir: 'dist'` 解析为 `frontend/dist`，与上面的 `frontendDist` 对应。 |
| `Menu` 原生右键菜单 | Tauri v2 **没有** `Menu::popup` 公开方法；公开入口是 `Window::popup_menu(&self, menu: &impl ContextMenu)`。`Menu` 实现了 `ContextMenu`。 |
| `Menu::set_as_app_menu` | 签名 `pub fn set_as_app_menu(&self) -> crate::Result<Option<Menu<R>>>`。 |
| `MenuItem` 构造 | `MenuItem::with_id(manager, id, text, enabled, accelerator)` / `MenuItem::new(manager, text, enabled, accelerator)`；`set_text(&self, text) -> Result<()>` 可运行时改文案。 |
| `PredefinedMenuItem` | 提供 `separator` / `copy` / `paste` / `select_all` / `minimize` / `fullscreen` / `hide` / `close_window` / `quit` / `about`，均签名 `(manager, Option<&str>) -> Result<Self>`。 |
| 托盘构造器 | `TrayIconBuilder::new()` → `.menu(&menu)` / `.icon(Image)` / `.tooltip(s)` / `.menu_on_left_click(bool)` / `.icon_as_template(bool)` → `.build(manager) -> Result<TrayIcon>`。注意方法名是 `menu` 不是 `with_menu`，是 `icon` 不是 `with_icon`。 |
| 托盘位图 | `tauri::image::Image::new_owned(rgba: Vec<u8>, width: u32, height: u32) -> Image<'static>`，可直接喂 `core::icon::bird_rgba(22)` 的输出。 |
| 激活策略 | `objc2-app-kit 0.3`：`NSApplication::sharedApplication(mtm: MainThreadMarker) -> Retained<NSApplication>`；`app.setActivationPolicy(policy) -> bool`；策略常量 `NSApplicationActivationPolicy::Regular`（0）/ `::Accessory`（1）。 |
| Node 版本 | nvm 已装 v22.23.2 / v24.14.0 / v24.17.0；默认 v22.11.0 不满足 Vite 7。 |
| 前端截图验收 | 本机 Google Chrome 可用：`"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless --screenshot=out.png --window-size=1080,720 http://localhost:1420` |
| 已知踩坑（勿重试） | `@vitejs/plugin-react` 6.x 只支持 Vite 8 → 用 5.0.4 + Vite 7.1.5；TypeScript 7.0.2 是 Go 重写新 major → 用 5.9.3；crates.io API 不声明 MSRV，只能实测编译。 |

---

## 目录结构总览

```
chuaker7/
├── Cargo.toml                      # workspace 根（members = ["src-tauri"]）
├── Cargo.lock
├── .cargo/config.toml              # 保留（tuna 稀疏索引），对子目录同样生效
├── .nvmrc                          # 24.17.0
├── package.json                    # 唯一 package.json（前端全部依赖 + 全部脚本）
├── vite.config.ts
├── tsconfig.json
├── .gitignore                      # 增加 node_modules/、frontend/dist/
├── frontend/                       # Web 前端源码（不含 package.json）
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── test/setup.ts
│       ├── app/                    # Sidebar / TopBar / Toast / navigation
│       ├── components/             # VirtualList / DataTable / SearchField / ...
│       ├── pages/                  # Processes / Files / Clipboard / Monitor / Plugins / Settings
│       ├── lib/                    # bridge / store / types / format / i18n / native
│       └── styles/                 # tokens.css / base.css / native.css
├── src-tauri/                      # Rust 侧
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/                      # icon.png / 32x32 / 128x128 / icon.icns / icon.ico
│   └── src/
│       ├── main.rs                 # Tauri Builder 装配
│       ├── lib.rs
│       ├── state.rs                # AppState
│       ├── events.rs               # 事件名常量 + 负载类型
│       ├── commands/               # command 门面 + dto.rs
│       ├── core/                   # 业务逻辑（原样迁移）
│       │   ├── process.rs files.rs clipboard.rs monitor.rs
│       │   ├── plugin.rs i18n.rs settings.rs platform.rs icon.rs theme.rs
│       └── system/                 # 原生集成
│           ├── menu.rs tray.rs shortcut.rs autolaunch.rs activation.rs
├── locales/                        # 文案表（不变）
├── assets/fonts/                   # Anton + Geist Mono（不变）
├── scripts/make-icon.mjs           # 依赖无关的 PNG 生成器
└── docs/
```

### 相对设计文档的两处偏离（已评估，均为降低风险）

1. **`frontend/` 不设 `package.json`**：Tauri v2 的 `beforeDevCommand` 工作目录实测为**仓库根**（见「已验证事实」）。若把 `package.json` 放在 `frontend/` 下，就得用 `npm --prefix frontend run dev`，会额外引入 npm 的 prefix 解析与祖先 `node_modules/.bin` 查找规则。改为**仓库根单一 `package.json` 承载全部依赖与全部脚本**，`vite.config.ts` 用 `root: 'frontend'` 把源码目录固定住，`build.outDir: 'dist'` 解析为 `frontend/dist`。这样 `beforeDevCommand` 直接是 `npm run dev`，路径确定、依赖唯一。
2. **`Settings` 直接加 camelCase 序列化**（而非另建 DTO）：`#[serde(rename_all = "camelCase")]` + 每个字段加 `#[serde(alias = "snake_case 名")]`。这样**同一个结构体**既能被前端消费，又能读回 egui 版写下的 snake_case 旧配置文件，用户设置不丢。

---

## 契约附录（实现时以此为准）

### Command 清单（Rust `#[tauri::command]`，全部位于 `src-tauri/src/commands/`）

| 分组 | 命令 | 参数 | 返回 |
| --- | --- | --- | --- |
| 进程 | `list_processes` | — | `Vec<ProcessRow>` |
| | `refresh_processes` | — | `()` |
| | `kill_process` | `pid: i32` | `Result<bool, String>` |
| 文件 | `list_dir` | `path: String` | `Vec<FileRow>` |
| | `create_folder` | `dir: String, name: String` | `Result<(), String>` |
| | `create_file` | `dir: String, name: String` | `Result<(), String>` |
| | `rename_path` | `path: String, new_name: String` | `Result<(), String>` |
| | `delete_to_trash` | `path: String` | `Result<(), String>` |
| | `reveal_in_finder` | `path: String` | `()` |
| | `parent_dir` | `path: String` | `Option<String>` |
| | `copy_text` | `text: String` | `()` |
| | `home_dir` | — | `String` |
| | `watch_dir` | `path: String` | `()` |
| 剪贴板 | `clipboard_current` | — | `String` |
| | `clipboard_history` | — | `Vec<HistoryRow>` |
| | `remove_history_item` | `id: u64` | `()` |
| | `clear_history` | — | `()` |
| 监控 | `system_snapshot` | — | `SystemInfoDto` |
| 插件 | `list_plugins` | — | `Vec<PluginRow>` |
| | `reload_plugins` | — | `Vec<PluginRow>` |
| | `plugins_dir` | — | `String` |
| 设置 | `get_settings` | — | `Settings` |
| | `update_settings` | `patch: SettingsPatch` | `Settings` |
| | `get_strings` | `locale: String` | `HashMap<String, String>` |
| 系统 | `set_menu_bar_mode` | `enabled: bool` | `()` |
| | `set_shortcut_enabled` | `enabled: bool` | `()` |
| | `set_auto_launch` | `enabled: bool` | `Result<(), String>` |
| | `hide_main_window` | — | `()` |
| | `show_main_window` | — | `()` |
| | `popup_context_menu` | `kind: String, payload: serde_json::Value` | `()` |

> `popup_context_menu` 分两步落地：**P2 阶段**（Task 11）先注册一个空实现（`popup_context_menu` 直接返回，不弹菜单），保证 command 名与参数签名从一开始就固定下来，前端可以并行开发；**P3 收尾**（Task 25）再补上完整菜单逻辑。Task 25 的验收标准包含「右键能弹出原生菜单」，因此空实现不会被遗留到最终交付。

### Event 清单（Rust `app.emit(name, payload)`）

| 事件名 | 负载 | 触发时机 |
| --- | --- | --- |
| `processes` | `Vec<ProcessRow>` | 进程列表刷新完成 |
| `system-info` | `SystemInfoDto` | 采样周期到达 |
| `clipboard` | `String` | 剪贴板内容变化 |
| `files-changed` | `()` | 当前目录增删 |
| `tray-open` | `()` | 点击托盘图标 / 托盘菜单「打开」 |
| `tray-quit` | `()` | 托盘菜单「退出」 |
| `notice` | `Notice { kind: String, message: String }` | Rust 侧完成用户可见动作后回传提示（`kind` 取 `"info"` / `"error"`） |
| `menu-action` | `String` | 原生菜单栏或右键菜单触发的动作（`nav:processes` / `app:settings` / `file:rename` / `file:delete`） |

> `notice` 与 `menu-action` 是相对设计文档 §6.2 的**增补**：`notice` 用于把 Rust 侧执行结果回传给前端提示；`menu-action` 用于把原生菜单栏与右键菜单的点击转成前端可执行的动作（切换标签页、弹重命名/删除确认框）。

### DTO 字段名（Rust `camelCase` → TS `camelCase`）

```ts
type ProcessRow = { pid: number; displayName: string; fullPath: string; ports: PortRow[] };
type PortRow    = { address: string; port: number };
type FileRow    = { path: string; name: string; isDir: boolean; size: number; sizeText: string; modified: number };
type HistoryRow = { id: number; content: string; timestamp: number; kind: string };
type SystemInfoDto = {
  cpuUsage: number; memoryUsage: number; diskUsage: number;
  networkUpload: number; networkDownload: number;
  cpuHistory: number[]; memoryHistory: number[];
};
type PluginRow = {
  id: string; name: string; version: string; author: string | null;
  description: string | null; minimumAppVersion: string | null;
  permissions: string[]; main: string | null; dir: string;
};
type Settings = {
  showMenuBar: boolean; autoRefreshInterval: number; processListLimit: number;
  enableNetworkMonitor: boolean; themeMode: 'system' | 'light' | 'dark';
  accent: 'signal' | 'haze' | 'moss' | 'amber';
  launchAtLogin: boolean; globalShortcutEnabled: boolean;
  locale: 'zh-Hans' | 'en';
};
type SettingsPatch = Partial<Settings>;
type Notice = { kind: 'info' | 'error'; message: string };
```

---

## P0 脚手架

### Task 1: 仓库转为 workspace，落地 Tauri 空壳

**Files:**
- Modify: `Cargo.toml`（根，改为虚拟 workspace）
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `scripts/make-icon.mjs`
- Create: `src-tauri/icons/*`（由 `tauri icon` 生成）
- Delete: `src/`（egui 层全部）、`build.rs`（根）
- Keep: `locales/`、`assets/`、`docs/`、`.cargo/`、`.gitignore`、`LICENSE`、`README.md`

**Interfaces:**
- Consumes: 无（首个任务）
- Produces: 可 `cargo build` 的 workspace；`src-tauri/` 目录结构；`scripts/make-icon.mjs` 生成 1024×1024 RGBA 源图标。

- [ ] **Step 1: 用 mavis-trash 归档 egui 源码，保留可回退能力**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
"/Users/lishiguang/.minimax/bin/mavis-trash" -- "src" "build.rs"
```

> egui 版已永久存档在 `legacy-egui` 分支（`b36b8dc`），此处删除不会丢失任何代码。

- [ ] **Step 2: 根 Cargo.toml 改为虚拟 workspace**

```toml
[workspace]
members = ["src-tauri"]
resolver = "2"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
opt-level = 1
```

- [ ] **Step 3: 创建 `src-tauri/Cargo.toml`**

```toml
[package]
name = "chuaker7"
version = "1.2.0"
edition = "2024"
rust-version = "1.88"
description = "chuaker7 — macOS 系统工具箱（Tauri v2 + React 重写版）"
authors = ["chuaker7 <chuaker7@users.noreply.github.com>"]
license = "MIT"
repository = "https://github.com/chuaker7/Ki6"
keywords = ["macos", "toolbox", "tauri", "system-monitor"]
categories = ["gui", "os::macos-apis"]

[lib]
name = "chuaker7_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.7", features = [] }

[dependencies]
tauri = { version = "2.12", features = ["tray-icon", "macos-private-api"] }
tauri-plugin-dialog = "2.8"
tauri-plugin-opener = "2.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "7.0"
anyhow = "1.0"
arboard = "3.7"
sysinfo = "0.38"
notify = "8.2"
trash = "5.4"
open = "5.4"
global-hotkey = "0.10"
objc2 = "0.6"
objc2-app-kit = "0.3"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 4: 创建 `src-tauri/build.rs` 与 `src-tauri/src/{main.rs,lib.rs}`**

`src-tauri/build.rs`：

```rust
fn main() {
    tauri_build::build()
}
```

`src-tauri/src/lib.rs`：

```rust
//! chuaker7 —— macOS 系统工具箱（Tauri v2 + React 重写版）。
//!
//! 业务逻辑在 `core/`，Tauri 命令门面在 `commands/`，原生集成在 `system/`。

pub mod commands;
pub mod core;
pub mod events;
pub mod state;
pub mod system;

/// Tauri Builder 装配入口。
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running chuaker7");
}
```

`src-tauri/src/main.rs`：

```rust
// 发布构建下隐藏终端窗口（macOS 无此效果，但保持 Windows/Linux 兼容写法一致）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chuaker7_lib::run();
}
```

> `lib.rs` 里先声明的 `commands` / `core` / `events` / `state` / `system` 模块在 Task 4 之前尚未存在，因此 **Step 4 暂时只保留 `run()`**，模块声明在 Task 4 补齐。执行 Step 4 时请写成：

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running chuaker7");
}
```

- [ ] **Step 5: 创建 `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "chuaker7",
  "version": "1.2.0",
  "identifier": "com.igigi.chuaker7",
  "build": {
    "frontendDist": "../frontend/dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "macOSPrivateApi": true,
    "windows": [
      {
        "label": "main",
        "title": "chuaker7",
        "width": 1080,
        "height": 720,
        "minWidth": 880,
        "minHeight": 600,
        "center": true,
        "titleBarStyle": "Overlay",
        "hiddenTitle": true,
        "trafficLightPosition": { "x": 18, "y": 18 },
        "backgroundColor": "#0F0E15"
      }
    ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "Utility",
    "shortDescription": "macOS 系统工具箱",
    "longDescription": "chuaker7 —— 进程管理、文件管理、剪贴板、系统监控、插件与设置。",
    "macOS": { "minimumSystemVersion": "12.0" }
  },
  "plugins": {}
}
```

- [ ] **Step 6: 创建 `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "chuaker7 默认权限",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-start-dragging",
    "dialog:default"
  ]
}
```

- [ ] **Step 7: 写 `scripts/make-icon.mjs`（依赖无关的 PNG 生成器）**

```javascript
// 生成 chuaker7 应用图标源图（1024×1024 RGBA PNG）。
// 设计语言取自 cindy.cn：近黑 ink 底 + 单一信号红强调 + 奶油色小菱形。
// 纯 Node 实现，不依赖任何三方库；PNG 由 node:zlib 手写编码。
import { deflateSync } from 'node:zlib';
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname } from 'node:path';

const SIZE = 1024;
const SS = 3; // 超采样倍数，用于抗锯齿

const INK = [0x0f, 0x0e, 0x15];
const SIGNAL = [0xf4, 0x3d, 0x3f];
const PAPER = [0xf3, 0xed, 0xe2];

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) {
    c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const body = Buffer.concat([Buffer.from(type, 'latin1'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([len, body, crc]);
}

function inRoundedRect(x, y, left, top, right, bottom, radius) {
  if (x < left || x > right || y < top || y > bottom) return false;
  const cx = Math.min(Math.max(x, left + radius), right - radius);
  const cy = Math.min(Math.max(y, top + radius), bottom - radius);
  const dx = x - cx;
  const dy = y - cy;
  return dx * dx + dy * dy <= radius * radius;
}

function inDiamond(x, y, cx, cy, half) {
  return Math.abs(x - cx) + Math.abs(y - cy) <= half;
}

function blend(dst, offset, color, alpha) {
  dst[offset] = Math.round(dst[offset] * (1 - alpha) + color[0] * alpha);
  dst[offset + 1] = Math.round(dst[offset + 1] * (1 - alpha) + color[1] * alpha);
  dst[offset + 2] = Math.round(dst[offset + 2] * (1 - alpha) + color[2] * alpha);
  dst[offset + 3] = Math.max(dst[offset + 3], Math.round(255 * alpha));
}

const rgba = Buffer.alloc(SIZE * SIZE * 4);
const step = 1 / SS;
for (let py = 0; py < SIZE; py += 1) {
  for (let px = 0; px < SIZE; px += 1) {
    let bgHits = 0;
    let bigHits = 0;
    let smallHits = 0;
    for (let sy = 0; sy < SS; sy += 1) {
      for (let sx = 0; sx < SS; sx += 1) {
        const x = px + (sx + 0.5) * step;
        const y = py + (sy + 0.5) * step;
        if (inRoundedRect(x, y, 96, 96, SIZE - 96, SIZE - 96, 220)) bgHits += 1;
        if (inDiamond(x, y, SIZE / 2, SIZE / 2, 300)) bigHits += 1;
        if (inDiamond(x, y, SIZE / 2, SIZE / 2, 130)) smallHits += 1;
      }
    }
    const total = SS * SS;
    const offset = (py * SIZE + px) * 4;
    blend(rgba, offset, INK, bgHits / total);
    blend(rgba, offset, SIGNAL, bigHits / total);
    blend(rgba, offset, PAPER, smallHits / total);
  }
}

const stride = SIZE * 4 + 1;
const raw = Buffer.alloc(stride * SIZE);
for (let y = 0; y < SIZE; y += 1) {
  raw[y * stride] = 0; // filter type 0 (None)
  rgba.copy(raw, y * stride + 1, y * SIZE * 4, (y + 1) * SIZE * 4);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // color type RGBA
ihdr[10] = 0;
ihdr[11] = 0;
ihdr[12] = 0;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);

const out = process.argv[2] ?? 'scripts/icon-source.png';
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, png);
console.log(`wrote ${out} (${SIZE}x${SIZE}, ${png.length} bytes)`);
```

- [ ] **Step 8: 生成图标并跑通构建**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
node scripts/make-icon.mjs scripts/icon-source.png
npx --yes @tauri-apps/cli@2.12.6 icon scripts/icon-source.png
cargo build -p chuaker7
```

Expected：
- `scripts/icon-source.png` 生成成功（文件大小 > 10KB）。
- `src-tauri/icons/` 下出现 `32x32.png` / `128x128.png` / `128x128@2x.png` / `icon.png` / `icon.icns` / `icon.ico`。
- `cargo build -p chuaker7` 输出 `Finished` 且 **0 warning**。

> `npx @tauri-apps/cli` 需要联网拉包；若网络不可用，改为先在根 `package.json` 安装 `@tauri-apps/cli`（Task 2 会做），此处可临时 `npm i -D @tauri-apps/cli@2.12.6` 后用 `npx tauri icon`。

- [ ] **Step 9: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 迁移到 Tauri v2 workspace 骨架

- 根 Cargo.toml 改为虚拟 workspace（members = [src-tauri]）
- 删除 egui 层源码（存档于 legacy-egui 分支 b36b8dc）
- 新增 src-tauri 壳工程：tauri.conf.json / capabilities / 窗口配置
- 新增依赖无关的 scripts/make-icon.mjs 与全套应用图标

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 2: 前端脚手架（Vite + React + TS）与 Tauri 联通

**Files:**
- Create: `package.json`（根）
- Create: `.nvmrc`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `frontend/index.html`
- Create: `frontend/src/main.tsx`
- Create: `frontend/src/App.tsx`
- Create: `frontend/src/vite-env.d.ts`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: Task 1 的 `src-tauri/tauri.conf.json`（`devUrl: http://localhost:1420`）
- Produces: `npm run dev`（Vite，端口 1420）、`npm run build`、`npm run typecheck`、`npm run test`、`npm run tauri`；`frontend/src/App.tsx` 导出 `App` 组件。

- [ ] **Step 1: 固定 Node 版本并写 `.nvmrc`**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
printf '24.17.0\n' > .nvmrc
source ~/.nvm/nvm.sh && nvm use
node -v
```

Expected：`v24.17.0`。

- [ ] **Step 2: 写根 `package.json`**

```json
{
  "name": "chuaker7",
  "private": true,
  "version": "1.2.0",
  "type": "module",
  "description": "chuaker7 —— macOS 系统工具箱（Tauri v2 + React 重写版）",
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "preview": "vite preview",
    "typecheck": "tsc --noEmit",
    "test": "vitest run",
    "tauri": "tauri",
    "icon": "node scripts/make-icon.mjs scripts/icon-source.png && tauri icon scripts/icon-source.png"
  },
  "dependencies": {
    "@tauri-apps/api": "2.12.5",
    "@tauri-apps/plugin-dialog": "2.8.4",
    "react": "19.3.1",
    "react-dom": "19.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "2.12.6",
    "@testing-library/jest-dom": "^6.6.3",
    "@testing-library/react": "^16.3.0",
    "@types/react": "^19.3.0",
    "@types/react-dom": "^19.3.0",
    "@vitejs/plugin-react": "5.0.4",
    "jsdom": "^26.1.0",
    "typescript": "5.9.3",
    "vite": "7.1.5",
    "vitest": "3.2.7"
  }
}
```

- [ ] **Step 3: 确认 `frontend/` 只放源码，不建 package.json**

`frontend/` 目录里**只有** `index.html` 与 `src/`：

```
frontend/
├── index.html
└── src/
```

依赖与脚本统一放在仓库根 `package.json`。Vite 通过 `root: 'frontend'` 找到入口，
vitest 通过同一个 `vite.config.ts` 找到测试文件。这样 `npm run dev`（Tauri 的
`beforeDevCommand`，工作目录实测为仓库根）一定能解析到本地 `vite` 可执行文件。

- [ ] **Step 4: 写 `vite.config.ts`**

```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  root: 'frontend',
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Rust 侧改动由 cargo 自己监听，不要让 Vite 扫描 target/ 造成风暴。
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'safari15',
    sourcemap: false,
    chunkSizeWarningLimit: 900,
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
```

> `defineConfig` 从 `vitest/config` 引入，这样 `test` 字段才有类型定义，`tsc --noEmit` 才不会报错。
> `root: 'frontend'` 时 `build.outDir: 'dist'` 解析为 `frontend/dist`，与 `tauri.conf.json` 的 `"frontendDist": "../frontend/dist"` 对应（后者相对 `src-tauri/` 解析，已实测）。

- [ ] **Step 5: 写 `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "exactOptionalPropertyTypes": false,
    "types": ["vitest/globals", "@testing-library/jest-dom"]
  },
  "include": ["frontend/src", "vite.config.ts"]
}
```

- [ ] **Step 6: 写前端入口文件**

`frontend/index.html`：

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>chuaker7</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`frontend/src/vite-env.d.ts`：

```typescript
/// <reference types="vite/client" />
```

`frontend/src/main.tsx`：

```tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';

const container = document.getElementById('root');
if (!container) {
  throw new Error('找不到 #root 挂载点');
}

ReactDOM.createRoot(container).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

`frontend/src/App.tsx`：

```tsx
export function App() {
  return (
    <main style={{ padding: 24, fontFamily: 'system-ui' }}>
      <h1>chuaker7</h1>
      <p>Tauri v2 + React 脚手架已就绪。</p>
    </main>
  );
}
```

- [ ] **Step 7: 更新 `.gitignore`**

在现有内容后追加：

```gitignore
# Node / 前端
node_modules/
frontend/dist/
.vite/
```

- [ ] **Step 8: 安装依赖并验证**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm install
npm run typecheck
npm run build
ls -la frontend/dist
```

Expected：
- `npm install` 无 error；`npm ls @vitejs/plugin-react vite typescript vitest react` 全部落在预期主版本。
- `npm run typecheck` 无输出、退出码 0。
- `npm run build` 产出 `frontend/dist/index.html` 与 `frontend/dist/assets/*.js`。

- [ ] **Step 9: 端到端验证 Tauri 窗口**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run tauri dev
```

Expected：弹出名为 chuaker7 的窗口，尺寸 1080×720，窗口内显示「chuaker7 / Tauri v2 + React 脚手架已就绪。」。用 `⌘W` 关闭后继续。

- [ ] **Step 10: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 前端脚手架 —— Vite 7 + React 19 + TypeScript 5.9

- 根 package.json 统一管理依赖，frontend/ 仅放源码
- vite.config.ts 固定 root=frontend、端口 1420、vitest jsdom 环境
- .nvmrc 锁定 node 24.17.0（Vite 7 要求 >=22.12）

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 3: 设计令牌、基础样式与字体

**Files:**
- Create: `frontend/src/styles/tokens.css`
- Create: `frontend/src/styles/base.css`
- Create: `frontend/src/styles/native.css`
- Create: `frontend/src/styles/fonts.css`
- Modify: `frontend/src/main.tsx`
- Create: `frontend/src/test/setup.ts`
- Create: `frontend/src/styles/tokens.test.ts`

**Interfaces:**
- Consumes: Task 2 的 Vite/TS 配置
- Produces: CSS 变量 `--bg` / `--surface` / `--surface-alt` / `--control` / `--text` / `--text-muted` / `--border` / `--accent` / `--on-accent`；`<html data-theme="dark|light" data-accent="signal|haze|moss|amber">` 主题契约。

- [ ] **Step 1: 写 `frontend/src/styles/tokens.css`**

```css
/* cindy.cn 设计令牌 → CSS 变量。视觉语言不随架构改版而变化。 */

:root {
  /* ink（深色）色板 */
  --ink-bg: #0f0e15;
  --ink-surface: #16151e;
  --ink-surface-alt: #201f2a;
  --ink-control: #24222f;
  --ink-text: #f3ede2;
  --ink-text-muted: #918f9e;
  --ink-border: rgba(243, 237, 226, 0.08);

  /* paper（浅色）色板 */
  --paper-bg: #faf6ee;
  --paper-surface: #f3ede2;
  --paper-surface-alt: #eae2d2;
  --paper-control: #ffffff;
  --paper-text: #14131b;
  --paper-text-muted: #5d5b66;
  --paper-border: rgba(20, 19, 27, 0.18);

  /* 强调色（深色模式取值） */
  --accent-signal: #f43d3f;
  --accent-haze: #a9c6e8;
  --accent-moss: #7fb069;
  --accent-amber: #e9b44c;

  /* 语义色 */
  --danger: #f43d3f;
  --success: #7fb069;
  --warning: #e9b44c;

  /* 排版 */
  --font-display: 'Anton', 'PingFang SC', 'Hiragino Sans GB', 'Songti SC', sans-serif;
  --font-mono: 'Geist Mono', 'SF Mono', ui-monospace, monospace;
  --font-body: -apple-system, BlinkMacSystemFont, 'PingFang SC', 'Hiragino Sans GB',
    'Songti SC', 'Microsoft YaHei', sans-serif;

  /* 尺寸 */
  --row-height: 26px;
  --radius-sm: 3px;
  --radius-md: 5px;
  --sidebar-width: 196px;
  --topbar-height: 38px;
}

/* 深色（ink）为默认，浅色（paper）按需覆盖 */
:root,
[data-theme='dark'] {
  --bg: var(--ink-bg);
  --surface: var(--ink-surface);
  --surface-alt: var(--ink-surface-alt);
  --control: var(--ink-control);
  --text: var(--ink-text);
  --text-muted: var(--ink-text-muted);
  --border: var(--ink-border);
  --on-accent: #0f0e15;
  --accent-signal: #f43d3f;
  --accent-haze: #a9c6e8;
  --accent-moss: #7fb069;
  --accent-amber: #e9b44c;
}

[data-theme='light'] {
  --bg: var(--paper-bg);
  --surface: var(--paper-surface);
  --surface-alt: var(--paper-surface-alt);
  --control: var(--paper-control);
  --text: var(--paper-text);
  --text-muted: var(--paper-text-muted);
  --border: var(--paper-border);
  --on-accent: #faf6ee;
  --accent-signal: #cf2b30;
  --accent-haze: #4a6fa5;
  --accent-moss: #3f6b4a;
  --accent-amber: #9a6b12;
}

/* 强调色选择 */
[data-accent='signal'] { --accent: var(--accent-signal); }
[data-accent='haze'] { --accent: var(--accent-haze); }
[data-accent='moss'] { --accent: var(--accent-moss); }
[data-accent='amber'] { --accent: var(--accent-amber); }
```

- [ ] **Step 2: 写 `frontend/src/styles/fonts.css`**

```css
/* Anton 与 Geist Mono 均为 SIL OFL 开源字体，许可证见 assets/fonts/。 */

@font-face {
  font-family: 'Anton';
  src: url('/fonts/Anton-Regular.ttf') format('truetype');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'Geist Mono';
  src: url('/fonts/GeistMono-Variable.ttf') format('truetype');
  font-weight: 100 900;
  font-style: normal;
  font-display: swap;
}
```

> 字体文件需通过 Vite 静态目录暴露：把 `assets/fonts/*.ttf` 复制到 `frontend/public/fonts/`。Step 3 会做。

- [ ] **Step 3: 暴露字体静态资源**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
mkdir -p frontend/public/fonts
cp assets/fonts/Anton-Regular.ttf assets/fonts/GeistMono-Variable.ttf frontend/public/fonts/
```

- [ ] **Step 4: 写 `frontend/src/styles/base.css`**

```css
* {
  box-sizing: border-box;
}

html,
body,
#root {
  height: 100%;
  margin: 0;
  padding: 0;
}

body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-body);
  font-size: 13px;
  line-height: 1.45;
  -webkit-font-smoothing: antialiased;
  overflow: hidden; /* 窗口外壳自行管理滚动 */
}

h1,
h2,
h3,
h4,
p,
figure {
  margin: 0;
}

/* 展示字体：宽字距大写标题，参考站的小标题观感 */
.kicker {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.display {
  font-family: var(--font-display);
  font-weight: 400;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

/* 编号 + ◆ 分隔符的标题行 */
.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.section-title::after {
  content: '';
  flex: 1;
  height: 1px;
  background: var(--border);
}

/* 发丝描边卡片 */
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}

.hairline {
  border: 0;
  border-top: 1px solid var(--border);
  margin: 0;
}

/* 键盘可达的系统焦点环 */
:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--accent) 70%, transparent);
  outline-offset: 1px;
}

::selection {
  background: var(--accent);
  color: var(--on-accent);
}
```

- [ ] **Step 5: 写 `frontend/src/styles/native.css`**

```css
/* macOS 化控件：细滚动条、原生输入框观感、复选框、分段控件。 */

/* 细滚动条，滚动时才显形（macOS overlay 行为） */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 4px;
  border: 2px solid transparent;
  background-clip: content-box;
}

*:hover::-webkit-scrollbar-thumb,
.scrollable:focus-within::-webkit-scrollbar-thumb {
  background-color: var(--text-muted);
  background-clip: content-box;
}

::-webkit-scrollbar-corner {
  background: transparent;
}

/* 文本输入：保留系统字体 / IME / 拼写检查 / 选区 / 光标，只去边框 */
.native-input {
  appearance: none;
  background: var(--control);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 4px 8px;
  font-family: var(--font-body);
  font-size: 13px;
  line-height: 1.4;
  width: 100%;
}

.native-input::placeholder {
  color: var(--text-muted);
}

.native-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 22%, transparent);
}

/* 复选框：自绘 macOS 方块 */
.native-checkbox {
  appearance: none;
  width: 14px;
  height: 14px;
  flex: 0 0 auto;
  border: 1px solid var(--text-muted);
  border-radius: 3px;
  background: transparent;
  cursor: default;
  position: relative;
}

.native-checkbox:hover {
  background: var(--surface-alt);
}

.native-checkbox:checked {
  background: var(--accent);
  border-color: var(--accent);
}

.native-checkbox:checked::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 4px;
  height: 8px;
  border: solid var(--on-accent);
  border-width: 0 1.8px 1.8px 0;
  transform: rotate(43deg);
}

/* 原生按钮：macOS 推入式观感 */
.native-button {
  appearance: none;
  background: var(--control);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 3px 10px;
  font-family: var(--font-body);
  font-size: 12px;
  cursor: default;
}

.native-button:hover:not(:disabled) {
  background: var(--surface-alt);
}

.native-button:active:not(:disabled) {
  background: var(--surface);
}

.native-button:disabled {
  opacity: 0.4;
}

.native-button--accent {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--on-accent);
}

.native-button--accent:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 86%, var(--bg));
}

/* 分段控件 */
.segmented {
  display: inline-flex;
  padding: 1px;
  gap: 1px;
  background: var(--control);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.segmented__item {
  appearance: none;
  border: 0;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: 12px;
  padding: 2px 10px;
  border-radius: 2px;
  cursor: default;
}

.segmented__item[aria-pressed='true'] {
  background: var(--surface-alt);
  color: var(--text);
}
```

- [ ] **Step 6: 接入样式并写测试脚手架**

`frontend/src/main.tsx` 改为：

```tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './styles/tokens.css';
import './styles/fonts.css';
import './styles/base.css';
import './styles/native.css';

const container = document.getElementById('root');
if (!container) {
  throw new Error('找不到 #root 挂载点');
}

ReactDOM.createRoot(container).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

`frontend/src/test/setup.ts`：

```typescript
import '@testing-library/jest-dom/vitest';
```

`frontend/src/styles/tokens.test.ts`：

```typescript
import { describe, expect, it } from 'vitest';
import tokens from './tokens.css?raw';

describe('cindy.cn 设计令牌', () => {
  it('保留 ink / paper 两套底色', () => {
    expect(tokens).toContain('#0f0e15');
    expect(tokens).toContain('#faf6ee');
  });

  it('保留奶油色正文与单一信号红强调', () => {
    expect(tokens).toContain('#f3ede2');
    expect(tokens).toContain('#f43d3f');
  });

  it('提供四档强调色与三档主题', () => {
    for (const accent of ['signal', 'haze', 'moss', 'amber']) {
      expect(tokens).toContain(`[data-accent='${accent}']`);
    }
    for (const theme of ['dark', 'light']) {
      expect(tokens).toContain(`[data-theme='${theme}']`);
    }
  });
});
```

> `?raw` 导入是 Vite 原生能力，vitest 同样支持。

- [ ] **Step 7: 跑测试与类型检查**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test
npm run typecheck
```

Expected：`tokens.test.ts` 3 个用例全 PASS；`typecheck` 退出码 0。

- [ ] **Step 8: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 设计令牌与 macOS 化基础样式

- tokens.css 定义 ink/paper 两套色板 + 四档强调色，主题走 data-theme 属性
- base.css 落排版基线、// 01 编号风格的小标题与发丝描边卡片
- native.css 提供细滚动条、原生输入框、macOS 方块复选框、分段控件
- 引入 Anton + Geist Mono（SIL OFL）

AI-Coding: true
AI-Tool: TophantAI"
```

---

## P1 core 迁移

> **P1 的核心原则**：`core/` 保持「不依赖 tauri」的纯业务层。所有线程、事件、AppHandle 相关的代码一律放在 `system/` 与 `commands/`。这样 core 的单元测试可以在没有 WebView 的情况下运行，也保证将来换前端不动业务逻辑。

### Task 4: 迁移 core 的纯逻辑模块

**Files:**
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/core/platform.rs`
- Create: `src-tauri/src/core/icon.rs`
- Create: `src-tauri/src/core/i18n.rs`
- Create: `src-tauri/src/core/settings.rs`
- Create: `src-tauri/src/core/theme.rs`
- Create: `src-tauri/src/core/plugin.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/core/{i18n,icon,settings,theme,plugin}.rs` 内的 `#[cfg(test)]` 模块

**Interfaces:**
- Consumes: `locales/*.strings`（路径变为 `../../../locales/...`）
- Produces:
  - `core::platform::{open_path, reveal_in_finder, copy_to_clipboard, read_clipboard}`
  - `core::icon::bird_rgba(size: u32) -> (Vec<u8>, u32, u32)`
  - `core::i18n::{Locale, I18n}`，`I18n::t(&self, key) -> String`、`I18n::tf(&self, key, &[&str]) -> String`、`I18n::table(&self) -> HashMap<String, String>`
  - `core::theme::{ThemeMode, Accent}`，两者均 `Serialize + Deserialize` 且带 `ALL` 常量
  - `core::settings::{Settings, SettingsPatch, data_dir, settings_path, clipboard_history_path, plugins_dir}`
  - `core::plugin::{PluginManifest, PluginManager}`

- [ ] **Step 1: 建目录并写 `core/mod.rs`**

```rust
//! 与 UI 无关的业务逻辑。
//!
//! 本层**不得**依赖 `tauri` crate：它只负责取数、解析、持久化，
//! 线程调度与事件派发由 `system/` 负责。

pub mod clipboard;
pub mod files;
pub mod i18n;
pub mod icon;
pub mod monitor;
pub mod plugin;
pub mod platform;
pub mod process;
pub mod settings;
pub mod theme;
```

> 本任务先只创建本步骤列出的 6 个文件，`clipboard/files/monitor/process` 四个模块在 Task 5 创建。若此刻 `mod.rs` 引用了尚不存在的文件会导致编译失败，因此 **Step 1 只写 `pub mod i18n; pub mod icon; pub mod platform; pub mod plugin; pub mod settings; pub mod theme;`**，Task 5 再补齐另外四行。

- [ ] **Step 2: 迁移 `platform.rs` 与 `icon.rs`（原样搬运）**

`src-tauri/src/core/platform.rs` —— 内容与 egui 版 `src/platform.rs` 完全一致（`open_path` / `reveal_in_finder` / `copy_to_clipboard` / `read_clipboard`），模块文档改为：

```rust
//! macOS 平台相关的辅助函数。
//!
//! 对应 Swift 版里直接调用 AppKit 的地方（`NSWorkspace`、`NSPasteboard`）。
//! `open` 与 `arboard` 均为跨平台 crate，无需 objc2 即可完成。
```

`src-tauri/src/core/icon.rs` —— 内容与 egui 版 `src/icon.rs` 完全一致（`bird_rgba` + 两个单元测试）。

- [ ] **Step 3: 迁移 `i18n.rs` 并修正 `include_str!` 路径**

`src-tauri/src/core/i18n.rs` —— 与 egui 版一致，**唯一改动**是两条 `include_str!` 的路径：

```rust
/// 简体中文文案（直接复用 Swift 版的 .strings 文件）。
const ZH_HANS: &str = include_str!("../../../locales/zh-Hans.lproj/Localizable.strings");
/// 英文文案（直接复用 Swift 版的 .strings 文件）。
const EN: &str = include_str!("../../../locales/en.lproj/Localizable.strings");
```

> 路径说明：文件位于 `src-tauri/src/core/i18n.rs`，`../` → `src-tauri/src/`，`../../` → `src-tauri/`，`../../../` → 仓库根，故 `locales/` 在 `../../../locales/`。

同时在 `impl I18n` 追加一个供 `get_strings` command 使用的方法：

```rust
    /// 导出整张文案表（供前端 `get_strings` 使用）。
    pub fn table(&self) -> HashMap<String, String> {
        self.table.clone()
    }
```

- [ ] **Step 4: 迁移 `theme.rs` 并剥离 egui 依赖**

`src-tauri/src/core/theme.rs` —— 只保留枚举，去掉 `Color32` / `Palette` / `visuals()`：

```rust
//! 主题设置。
//!
//! 视觉语言参考 cindy.cn：近黑的「ink」底色 + 奶油色文字 + 单一红色强调色。
//! 这里只保留**设置项**（模式 + 强调色），具体的颜色渲染由前端 `tokens.css`
//! 负责，Rust 侧是这些设置项的唯一权威来源。

use serde::{Deserialize, Serialize};

/// 主题模式。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// 跟随系统
    System,
    /// 浅色模式
    Light,
    /// 深色模式（默认，参考站的 ink 配色）
    #[default]
    Dark,
}

impl ThemeMode {
    /// 全部可选模式。
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark];
}

/// 强调色。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Accent {
    /// 信号红，参考站的主色。
    #[default]
    Signal,
    /// 雾蓝，参考站的次要强调色。
    Haze,
    /// 苔绿。
    Moss,
    /// 琥珀。
    Amber,
}

impl Accent {
    /// 全部强调色。
    pub const ALL: [Accent; 4] = [Accent::Signal, Accent::Haze, Accent::Moss, Accent::Amber];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark_signal() {
        assert_eq!(ThemeMode::default(), ThemeMode::Dark);
        assert_eq!(Accent::default(), Accent::Signal);
    }

    #[test]
    fn serializes_in_stable_wire_format() {
        assert_eq!(
            serde_json::to_string(&ThemeMode::Dark).unwrap(),
            "\"dark\""
        );
        assert_eq!(
            serde_json::to_string(&Accent::Signal).unwrap(),
            "\"signal\""
        );
    }
}
```

> `serde_json` 在 dev-dependencies 中已可用（它是常规依赖，直接用即可）。

- [ ] **Step 5: 迁移 `settings.rs`，改 camelCase 并兼容旧配置**

`src-tauri/src/core/settings.rs`：

```rust
//! 应用设置与持久化。
//!
//! Swift 版用 `@AppStorage` 把设置写在 `UserDefaults` 里；Rust 版写到
//! `~/Library/Application Support/chuaker7/settings.json`。
//!
//! 字段以 camelCase 落盘（与前端契约一致），同时通过 `alias` 接受 egui 版
//! 写下的 snake_case 旧字段，用户设置不会因为改版而丢失。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::i18n::Locale;
use crate::core::theme::{Accent, ThemeMode};

/// 应用数据目录（macOS 下为 `~/Library/Application Support/chuaker7`）。
pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("chuaker7")
}

/// 配置文件路径。
pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

/// 剪贴板历史文件路径。
pub fn clipboard_history_path() -> PathBuf {
    data_dir().join("clipboard_history.json")
}

/// 插件目录（对应 Swift 版的 `Application Support/LuciBox/Plugins`）。
pub fn plugins_dir() -> PathBuf {
    data_dir().join("Plugins")
}

/// 全局设置。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// 是否启用菜单栏模式。
    #[serde(alias = "show_menu_bar")]
    pub show_menu_bar: bool,
    /// 自动刷新间隔（秒）。
    #[serde(alias = "auto_refresh_interval")]
    pub auto_refresh_interval: f64,
    /// 进程列表上限，0 表示不限制。
    #[serde(alias = "process_list_limit")]
    pub process_list_limit: usize,
    /// 是否启用网络监控。
    #[serde(alias = "enable_network_monitor")]
    pub enable_network_monitor: bool,
    /// 主题模式。
    #[serde(alias = "theme_mode")]
    pub theme_mode: ThemeMode,
    /// 强调色。
    pub accent: Accent,
    /// 是否开机自启动。
    #[serde(alias = "launch_at_login")]
    pub launch_at_login: bool,
    /// 是否启用全局快捷键。
    #[serde(alias = "global_shortcut_enabled")]
    pub global_shortcut_enabled: bool,
    /// 界面语言。
    pub locale: Locale,
}

/// 设置补丁：全字段可选，前端只提交实际改动的字段。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    /// 菜单栏模式。
    #[serde(alias = "show_menu_bar")]
    pub show_menu_bar: Option<bool>,
    /// 自动刷新间隔（秒）。
    #[serde(alias = "auto_refresh_interval")]
    pub auto_refresh_interval: Option<f64>,
    /// 进程列表上限。
    #[serde(alias = "process_list_limit")]
    pub process_list_limit: Option<usize>,
    /// 网络监控开关。
    #[serde(alias = "enable_network_monitor")]
    pub enable_network_monitor: Option<bool>,
    /// 主题模式。
    #[serde(alias = "theme_mode")]
    pub theme_mode: Option<ThemeMode>,
    /// 强调色。
    pub accent: Option<Accent>,
    /// 开机自启。
    #[serde(alias = "launch_at_login")]
    pub launch_at_login: Option<bool>,
    /// 全局快捷键。
    #[serde(alias = "global_shortcut_enabled")]
    pub global_shortcut_enabled: Option<bool>,
    /// 界面语言。
    pub locale: Option<Locale>,
}

impl SettingsPatch {
    /// 把补丁合并进设置，返回新设置。
    pub fn apply(self, base: &Settings) -> Settings {
        let mut next = base.clone();
        if let Some(v) = self.show_menu_bar {
            next.show_menu_bar = v;
        }
        if let Some(v) = self.auto_refresh_interval {
            next.auto_refresh_interval = v;
        }
        if let Some(v) = self.process_list_limit {
            next.process_list_limit = v;
        }
        if let Some(v) = self.enable_network_monitor {
            next.enable_network_monitor = v;
        }
        if let Some(v) = self.theme_mode {
            next.theme_mode = v;
        }
        if let Some(v) = self.accent {
            next.accent = v;
        }
        if let Some(v) = self.launch_at_login {
            next.launch_at_login = v;
        }
        if let Some(v) = self.global_shortcut_enabled {
            next.global_shortcut_enabled = v;
        }
        if let Some(v) = self.locale {
            next.locale = v;
        }
        next
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_menu_bar: false,
            auto_refresh_interval: 5.0,
            process_list_limit: 500,
            enable_network_monitor: true,
            theme_mode: ThemeMode::default(),
            accent: Accent::default(),
            launch_at_login: false,
            global_shortcut_enabled: true,
            locale: Locale::detect(),
        }
    }
}

impl Settings {
    /// 从磁盘读取设置，文件缺失或损坏时返回默认值。
    pub fn load() -> Self {
        let Ok(text) = std::fs::read_to_string(settings_path()) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// 写回磁盘，失败时忽略（设置保存失败不应影响使用）。
    pub fn save(&self) {
        let dir = data_dir();
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(settings_path(), text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_only_touches_given_fields() {
        let base = Settings::default();
        let patch = SettingsPatch {
            process_list_limit: Some(100),
            theme_mode: Some(ThemeMode::Light),
            ..Default::default()
        };
        let next = patch.apply(&base);
        assert_eq!(next.process_list_limit, 100);
        assert_eq!(next.theme_mode, ThemeMode::Light);
        assert_eq!(next.auto_refresh_interval, base.auto_refresh_interval);
        assert_eq!(next.accent, base.accent);
    }

    #[test]
    fn reads_legacy_snake_case_config() {
        let legacy = r#"{
            "show_menu_bar": true,
            "auto_refresh_interval": 8.0,
            "process_list_limit": 42,
            "enable_network_monitor": false,
            "theme_mode": "light",
            "accent": "moss",
            "launch_at_login": true,
            "global_shortcut_enabled": false,
            "locale": "en"
        }"#;
        let parsed: Settings = serde_json::from_str(legacy).expect("旧配置应可解析");
        assert!(parsed.show_menu_bar);
        assert_eq!(parsed.auto_refresh_interval, 8.0);
        assert_eq!(parsed.process_list_limit, 42);
        assert!(!parsed.enable_network_monitor);
        assert_eq!(parsed.theme_mode, ThemeMode::Light);
        assert_eq!(parsed.accent, Accent::Moss);
        assert!(parsed.launch_at_login);
        assert!(!parsed.global_shortcut_enabled);
        assert_eq!(parsed.locale, Locale::En);
    }

    #[test]
    fn writes_camel_case_config() {
        let text = serde_json::to_string(&Settings::default()).unwrap();
        assert!(text.contains("\"showMenuBar\""));
        assert!(text.contains("\"autoRefreshInterval\""));
    }
}
```

- [ ] **Step 6: 迁移 `plugin.rs`**

`src-tauri/src/core/plugin.rs` —— 与 egui 版一致，把 `use crate::settings::plugins_dir;` 改成 `use crate::core::settings::plugins_dir;`，并追加 `PluginManifest` 的 `#[serde(default)]` 保持不变。额外为 `PluginManager` 追加一个只读目录查询，供 `PluginRow.dir` 使用：

```rust
    /// 插件目录路径。
    pub fn plugins_dir(&self) -> PathBuf {
        plugins_dir()
    }
```

- [ ] **Step 7: 注册模块并跑测试**

`src-tauri/src/lib.rs` 改为：

```rust
//! chuaker7 —— macOS 系统工具箱（Tauri v2 + React 重写版）。
//!
//! 业务逻辑在 `core/`，Tauri 命令门面在 `commands/`，原生集成在 `system/`。

pub mod core;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running chuaker7");
}
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo test -p chuaker7
cargo clippy -p chuaker7 --all-targets
```

Expected：所有测试 PASS，`clippy` **0 warning**。

- [ ] **Step 8: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 迁移 core 纯逻辑模块（平台/图标/本地化/设置/主题/插件）

- core/ 保持不依赖 tauri，业务逻辑可脱离 WebView 单测
- i18n 的 include_str! 路径随 crate 迁移修正为 ../../../locales/
- Settings 改 camelCase 落盘并用 alias 兼容 egui 版 snake_case 旧配置
- theme 剥离 egui Color32，只保留 ThemeMode / Accent 设置项

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 5: 迁移 core 的取数模块并引入事件枚举

**Files:**
- Create: `src-tauri/src/core/process.rs`
- Create: `src-tauri/src/core/files.rs`
- Create: `src-tauri/src/core/clipboard.rs`
- Create: `src-tauri/src/core/monitor.rs`
- Create: `src-tauri/src/events.rs`
- Modify: `src-tauri/src/core/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: Task 4 的 `core::settings`、`core::platform`
- Produces:
  - `core::process::{PortInfo, ProcessInfo, ProcessManager}`，`ProcessManager::collect(&mut self, limit) -> Vec<ProcessInfo>`（同步）、`ProcessManager::load_blocking(&mut self, limit)`
  - `core::files::{FileItem, FileManager, format_bytes}`，`FileManager::{load_files, navigate_to, go_up, create_folder, create_file, delete_file, rename_file}`
  - `core::clipboard::{HistoryEntry, ClipboardManager}`，`HistoryEntry { id: u64, content: String, timestamp: i64, kind: String }`，`ClipboardManager::{load_history, save_history, add_to_history, delete_history_item(id), clear_history, copy_to_clipboard}`
  - `core::monitor::{SystemInfo, SystemMonitor}`，`SystemMonitor::push_sample`，`core::monitor::sample(...) -> SystemInfo`
  - `events::AppEvent`（crate 级枚举，线程与 UI 之间传递用）

- [ ] **Step 1: 写 `src-tauri/src/events.rs`**

```rust
//! 后台线程与界面之间的事件通道。
//!
//! Swift 版用 `DispatchQueue.main.async` 把后台结果送回主线程；
//! Rust 版用 `std::sync::mpsc` 通道，事件循环每轮轮询一次。

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

use crate::core::monitor::SystemInfo;
use crate::core::process::{PortInfo, ProcessInfo};

/// 后台产生的事件。
pub enum AppEvent {
    /// 进程列表刷新完成。
    Processes(Vec<ProcessInfo>),
    /// 端口信息缓存更新。
    PortCacheUpdated(HashMap<i32, Vec<PortInfo>>),
    /// 进程列表正在加载。
    ProcessLoading(bool),
    /// 系统监控数据更新。
    SystemInfo(SystemInfo),
    /// 剪贴板当前内容变化。
    ClipboardContent(String),
    /// 当前目录内容发生变化。
    FilesChanged,
    /// 菜单栏请求打开主窗口。
    TrayOpen,
    /// 菜单栏请求退出应用。
    TrayQuit,
    /// 全局快捷键被按下。
    Hotkey,
}

/// 创建事件通道。
pub fn channel() -> (Sender<AppEvent>, Receiver<AppEvent>) {
    std::sync::mpsc::channel()
}
```

- [ ] **Step 2: 迁移 `process.rs`，把异步刷新改为同步采集**

`src-tauri/src/core/process.rs` —— 以 egui 版为蓝本，**改动三处**：

1. `use crate::events::AppEvent;` 保留（`AppEvent` 仍在 crate 级）。
2. `ProcessManager::refresh` 由「内部 spawn 线程 + 发事件」改为**同步采集并直接返回结果**：

```rust
    /// 采集一次进程列表（同步）。
    ///
    /// 端口缓存未过期时直接复用，避免重复调用 lsof。
    /// 线程调度由 `system::tasks` 负责，本方法保持纯同步以便单测。
    pub fn collect(&mut self, limit: usize) -> Vec<ProcessInfo> {
        let cached = match self.last_port_fetch {
            Some(ts) if ts.elapsed() < Self::PORT_CACHE_TTL => Some(self.port_cache.clone()),
            _ => None,
        };

        let (port_map, refreshed) = match cached {
            Some(map) => (map, false),
            None => (fetch_listening_ports(), true),
        };

        if refreshed {
            self.port_cache = port_map.clone();
            self.last_port_fetch = Some(Instant::now());
        }

        fetch_all_processes(&port_map, limit)
    }
```

3. 保留 `load_blocking`（首次启动同步采样一次，窗口打开即有数据）与 `kill_process`、`set_port_cache`；保留原有两个单元测试。

- [ ] **Step 3: 迁移 `files.rs`**

`src-tauri/src/core/files.rs` —— 与 egui 版**逐字一致**（`FileItem` / `FileManager` / `format_bytes`），仅模块文档末句改为：

```rust
//! Rust 版用 `std::fs` 读取目录、用 `notify` 监听变更，语义一致。
//! 目录监听线程由 `system::tasks` 负责，本模块只负责读取与变更。
```

> egui 版 `files.rs` 没有单元测试。**本任务补一个** `format_bytes` 的测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_byte_sizes() {
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MB");
    }
}
```

- [ ] **Step 4: 迁移 `clipboard.rs`，给历史条目加稳定 id**

`src-tauri/src/core/clipboard.rs` —— 以 egui 版为蓝本，改动如下：

```rust
/// 剪贴板历史条目。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 稳定标识（删除历史时使用）。
    #[serde(default)]
    pub id: u64,
    /// 文本内容。
    pub content: String,
    /// 时间戳（秒级 UNIX 时间）。
    pub timestamp: i64,
    /// 类型标签。
    pub kind: String,
}
```

`ClipboardManager` 的 `add_to_history` 改为分配 id：

```rust
    /// 向历史记录头部插入一条内容，并裁剪到上限。
    pub fn add_to_history(&mut self, content: &str, kind: &str) {
        // 与 Swift 版一致：不重复记录与上一条相同的内容。
        if self.history.first().map(|item| item.content.as_str()) == Some(content) {
            return;
        }

        self.next_id += 1;
        self.history.insert(
            0,
            HistoryEntry {
                id: self.next_id,
                content: content.to_string(),
                timestamp: HistoryEntry::now(),
                kind: kind.to_string(),
            },
        );

        if self.history.len() > self.max_history_count {
            self.history.truncate(self.max_history_count);
        }

        self.save_history();
    }
```

`load_history` 增加 id 规范化（老文件没有 id，或 id 重复时按顺序重排）：

```rust
    /// 从磁盘载入历史记录。
    pub fn load_history(&mut self) {
        let Ok(text) = std::fs::read_to_string(clipboard_history_path()) else {
            return;
        };
        let Ok(mut history) = serde_json::from_str::<Vec<HistoryEntry>>(&text) else {
            return;
        };
        normalize_ids(&mut history);
        self.next_id = history.iter().map(|item| item.id).max().unwrap_or(0);
        self.history = history;
    }
```

`delete_history_item` 改为按 id 删除：

```rust
    /// 删除某条历史（按稳定 id）。
    pub fn delete_history_item(&mut self, id: u64) {
        if let Some(index) = self.history.iter().position(|item| item.id == id) {
            self.history.remove(index);
            self.save_history();
        }
    }
```

在文件末尾（`#[cfg(test)]` 之前）加自由函数：

```rust
/// 规范化历史条目的 id：缺失（为 0）或重复时按顺序重排为 `1..=n`。
pub fn normalize_ids(history: &mut Vec<HistoryEntry>) {
    let mut seen: Vec<u64> = Vec::with_capacity(history.len());
    let mut needs_rewrite = false;

    for item in history.iter() {
        if item.id == 0 || seen.contains(&item.id) {
            needs_rewrite = true;
            break;
        }
        seen.push(item.id);
    }

    if !needs_rewrite {
        return;
    }

    for (index, item) in history.iter_mut().enumerate() {
        item.id = (index + 1) as u64;
    }
}
```

`ClipboardManager` 结构体加字段：

```rust
pub struct ClipboardManager {
    /// 当前剪贴板文本。
    pub current_content: String,
    /// 历史记录。
    pub history: Vec<HistoryEntry>,
    /// 历史记录最大条数。
    pub max_history_count: usize,
    /// 下一个可用 id。
    next_id: u64,
}
```

`Default` 实现同步补 `next_id: 0`。

**删除** egui 版里的 `pub fn spawn_monitor(tx: Sender<AppEvent>)`（线程归 `system::tasks` 管），改为在 `core::clipboard` 里只留纯读取：

```rust
/// 读取系统剪贴板文本。
pub fn read_current() -> Option<String> {
    crate::core::platform::read_clipboard()
}
```

并补单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_missing_ids() {
        let mut history = vec![
            HistoryEntry { id: 0, content: "a".into(), timestamp: 1, kind: "text".into() },
            HistoryEntry { id: 0, content: "b".into(), timestamp: 2, kind: "text".into() },
        ];
        normalize_ids(&mut history);
        assert_eq!(history[0].id, 1);
        assert_eq!(history[1].id, 2);
    }

    #[test]
    fn keeps_unique_ids() {
        let mut history = vec![
            HistoryEntry { id: 7, content: "a".into(), timestamp: 1, kind: "text".into() },
            HistoryEntry { id: 9, content: "b".into(), timestamp: 2, kind: "text".into() },
        ];
        normalize_ids(&mut history);
        assert_eq!(history[0].id, 7);
        assert_eq!(history[1].id, 9);
    }

    #[test]
    fn duplicate_ids_are_rewritten() {
        let mut history = vec![
            HistoryEntry { id: 3, content: "a".into(), timestamp: 1, kind: "text".into() },
            HistoryEntry { id: 3, content: "b".into(), timestamp: 2, kind: "text".into() },
        ];
        normalize_ids(&mut history);
        assert_eq!(history[0].id, 1);
        assert_eq!(history[1].id, 2);
    }
}
```

- [ ] **Step 5: 迁移 `monitor.rs`，把采样线程改为纯函数**

`src-tauri/src/core/monitor.rs` —— 以 egui 版为蓝本，**删除** `pub fn spawn_monitor(...)`（线程归 `system::tasks`），改为：

```rust
/// 采集一次系统指标。
///
/// `system` 与 `networks` 必须由调用方复用：每次新建会让 CPU / 网络基线被重置，
/// 算出来的使用率与速率永远不可信。
pub fn sample(
    system: &mut System,
    networks: &mut sysinfo::Networks,
    interval: f64,
    network_enabled: bool,
) -> SystemInfo {
    system.refresh_cpu_usage();
    system.refresh_memory();

    let cpu_usage = average_cpu_usage(system);
    let memory_usage = memory_usage(system);
    let disk_usage = disk_usage();

    let (network_upload, network_download) = if network_enabled {
        network_usage(networks, interval)
    } else {
        (0.0, 0.0)
    };

    SystemInfo {
        cpu_usage,
        memory_usage,
        disk_usage,
        network_upload,
        network_download,
    }
}
```

`SystemInfo` 结构体**追加** `Serialize`（供 `system-info` 事件使用）：

```rust
use serde::{Deserialize, Serialize};

/// 一次系统指标快照。
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    /// CPU 使用率（%）。
    pub cpu_usage: f64,
    /// 内存使用率（%）。
    pub memory_usage: f64,
    /// 磁盘使用率（%）。
    pub disk_usage: f64,
    /// 上传速度（MB/s）。
    pub network_upload: f64,
    /// 下载速度（MB/s）。
    pub network_download: f64,
}
```

`SystemMonitor` 保留 `push_sample` 与 120 点上限逻辑。删除 `use std::sync::mpsc::Sender;` 与 `use crate::events::AppEvent;`。

- [ ] **Step 6: 补齐 `core/mod.rs` 与 `lib.rs`**

`src-tauri/src/core/mod.rs` 追加四行：

```rust
pub mod clipboard;
pub mod files;
pub mod monitor;
pub mod process;
```

`src-tauri/src/lib.rs` 追加：

```rust
pub mod events;
```

- [ ] **Step 7: 跑测试与静态检查**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt
cargo test -p chuaker7
cargo clippy -p chuaker7 --all-targets
```

Expected：测试数 ≥ 19（egui 版基线）且全 PASS；`clippy` 0 warning；`cargo fmt --check` 干净。

- [ ] **Step 8: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 迁移 core 取数模块并解耦线程调度

- process / files / clipboard / monitor 原样搬运，线程逻辑上移到 system::tasks
- clipboard 历史条目引入稳定 id，load 时规范化旧文件的缺失/重复 id
- monitor::sample 改为纯函数，SystemInfo 加 camelCase 序列化
- 补 format_bytes、id 规范化等单元测试

AI-Coding: true
AI-Tool: TophantAI"
```

---

## P2 命令、事件与原生集成

### Task 6: DTO 契约层

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/dto.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands/dto.rs` 内的 `#[cfg(test)]` 模块

**Interfaces:**
- Consumes: Task 4/5 的 core 类型
- Produces: `commands::dto::{ProcessRow, PortRow, FileRow, HistoryRow, SystemInfoDto, PluginRow, Notice, ContextTarget}` 及其 `From` 转换

- [ ] **Step 1: 写 `src-tauri/src/commands/dto.rs`**

```rust
//! 前后端契约的数据结构。
//!
//! 所有字段用 camelCase，与 `frontend/src/lib/types.ts` 一一对应。
//! core 内部类型保持自己的形状，转换只在这一层发生。

use serde::{Deserialize, Serialize};

use crate::core::clipboard::HistoryEntry;
use crate::core::files::FileItem;
use crate::core::monitor::SystemInfo;
use crate::core::plugin::PluginManifest;
use crate::core::process::ProcessInfo;

/// 端口信息。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortRow {
    /// 绑定地址，例如 `*`、`127.0.0.1`。
    pub address: String,
    /// 端口号。
    pub port: u16,
}

/// 进程列表的一行。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRow {
    /// 进程 PID。
    pub pid: i32,
    /// 展示名称。
    pub display_name: String,
    /// 可执行文件完整路径。
    pub full_path: String,
    /// 正在监听的端口。
    pub ports: Vec<PortRow>,
}

/// 文件列表的一行。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRow {
    /// 完整路径。
    pub path: String,
    /// 文件名。
    pub name: String,
    /// 是否是目录。
    pub is_dir: bool,
    /// 字节大小。
    pub size: i64,
    /// 人类可读大小（目录为 `-`）。
    pub size_text: String,
    /// 修改时间（秒级 UNIX 时间）。
    pub modified: i64,
}

/// 剪贴板历史的一行。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRow {
    /// 稳定标识。
    pub id: u64,
    /// 文本内容。
    pub content: String,
    /// 时间戳（秒级 UNIX 时间）。
    pub timestamp: i64,
    /// 类型标签。
    pub kind: String,
}

/// 系统指标快照。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoDto {
    /// CPU 使用率（%）。
    pub cpu_usage: f64,
    /// 内存使用率（%）。
    pub memory_usage: f64,
    /// 磁盘使用率（%）。
    pub disk_usage: f64,
    /// 上传速度（MB/s）。
    pub network_upload: f64,
    /// 下载速度（MB/s）。
    pub network_download: f64,
    /// CPU 采样历史。
    pub cpu_history: Vec<f64>,
    /// 内存采样历史。
    pub memory_history: Vec<f64>,
}

/// 插件列表的一行。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginRow {
    /// 插件唯一标识。
    pub id: String,
    /// 插件名称。
    pub name: String,
    /// 插件版本。
    pub version: String,
    /// 作者。
    pub author: Option<String>,
    /// 描述。
    pub description: Option<String>,
    /// 最低兼容的应用版本。
    pub minimum_app_version: Option<String>,
    /// 申请的权限。
    pub permissions: Vec<String>,
    /// 入口文件。
    pub main: Option<String>,
    /// 插件所在目录。
    pub dir: String,
}

/// 界面提示。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    /// 提示类型：`info` 或 `error`。
    pub kind: String,
    /// 提示文案。
    pub message: String,
}

/// 原生右键菜单的目标对象。
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum ContextTarget {
    /// 进程行。
    Process {
        /// 进程 PID。
        pid: i32,
        /// 展示名称。
        display_name: String,
        /// 完整路径。
        full_path: String,
    },
    /// 文件行。
    File {
        /// 完整路径。
        path: String,
        /// 文件名。
        name: String,
        /// 是否是目录。
        is_dir: bool,
    },
    /// 剪贴板历史行。
    Clipboard {
        /// 历史条目 id。
        id: u64,
        /// 内容。
        content: String,
    },
}

impl From<&ProcessInfo> for ProcessRow {
    fn from(value: &ProcessInfo) -> Self {
        Self {
            pid: value.pid,
            display_name: value.display_name.clone(),
            full_path: value.full_path.clone(),
            ports: value
                .ports
                .iter()
                .map(|p| PortRow {
                    address: p.address.clone(),
                    port: p.port,
                })
                .collect(),
        }
    }
}

impl From<&FileItem> for FileRow {
    fn from(value: &FileItem) -> Self {
        let modified = value
            .modified
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            path: value.path.to_string_lossy().to_string(),
            name: value.name.clone(),
            is_dir: value.is_dir,
            size: value.size,
            size_text: value.size_string(),
            modified,
        }
    }
}

impl From<&HistoryEntry> for HistoryRow {
    fn from(value: &HistoryEntry) -> Self {
        Self {
            id: value.id,
            content: value.content.clone(),
            timestamp: value.timestamp,
            kind: value.kind.clone(),
        }
    }
}

impl From<&SystemInfo> for SystemInfoDto {
    fn from(value: &SystemInfo) -> Self {
        Self {
            cpu_usage: value.cpu_usage,
            memory_usage: value.memory_usage,
            disk_usage: value.disk_usage,
            network_upload: value.network_upload,
            network_download: value.network_download,
            cpu_history: Vec::new(),
            memory_history: Vec::new(),
        }
    }
}

impl From<&PluginManifest> for PluginRow {
    fn from(value: &PluginManifest) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            version: value.version.clone(),
            author: value.author.clone(),
            description: value.description.clone(),
            minimum_app_version: value.minimum_app_version.clone(),
            permissions: value.permissions.clone().unwrap_or_default(),
            main: value.main.clone(),
            dir: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_row_serializes_in_camel_case() {
        let info = ProcessInfo {
            pid: 42,
            full_path: "/usr/bin/x".into(),
            display_name: "x".into(),
            ports: vec![crate::core::process::PortInfo {
                address: "*".into(),
                port: 8080,
            }],
        };
        let json = serde_json::to_string(&ProcessRow::from(&info)).unwrap();
        assert!(json.contains("\"displayName\""));
        assert!(json.contains("\"fullPath\""));
        assert!(json.contains("\"ports\""));
    }

    #[test]
    fn file_row_serializes_in_camel_case() {
        let item = FileItem {
            path: std::path::PathBuf::from("/tmp/a.txt"),
            name: "a.txt".into(),
            is_dir: false,
            size: 1024,
            modified: std::time::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&FileRow::from(&item)).unwrap();
        assert!(json.contains("\"isDir\""));
        assert!(json.contains("\"sizeText\""));
        assert!(json.contains("\"1.0 KB\""));
    }

    #[test]
    fn context_target_accepts_tagged_payload() {
        let target: ContextTarget = serde_json::from_str(
            r#"{"kind":"process","payload":{"pid":7,"displayName":"x","fullPath":"/x"}}"#,
        )
        .expect("应能解析带 tag 的负载");
        match target {
            ContextTarget::Process { pid, .. } => assert_eq!(pid, 7),
            _ => panic!("应解析为进程目标"),
        }
    }
}
```

- [ ] **Step 2: 写 `src-tauri/src/commands/mod.rs`**

```rust
//! Tauri command 门面。
//!
//! 这一层只做「编解码 + 转发」：参数来自前端，调用 core 或 system，
//! 返回可序列化的结果。业务逻辑不写在这里。

pub mod dto;
```

- [ ] **Step 3: 注册模块并跑测试**

`src-tauri/src/lib.rs` 追加 `pub mod commands;`。

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo test -p chuaker7 && cargo clippy -p chuaker7 --all-targets
```

Expected：DTO 三个测试 PASS；`clippy` 0 warning。

- [ ] **Step 4: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 新增前后端 DTO 契约层

- ProcessRow / FileRow / HistoryRow / SystemInfoDto / PluginRow 统一 camelCase
- ContextTarget 用 serde 的 tag+content 承载右键菜单目标
- core 类型到 DTO 的转换集中在一处，core 本身不感知序列化

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 7: AppState 与后台任务循环

**Files:**
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/system/mod.rs`
- Create: `src-tauri/src/system/tasks.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: Task 4/5/6 的 core 与 dto
- Produces:
  - `state::AppState`：`settings` / `i18n` / `processes` / `files` / `clipboard` / `monitor` / `plugins` / `sample_interval_ms` / `network_enabled` / `watcher`
  - `system::tasks::{spawn_process_loop, spawn_clipboard_loop, spawn_monitor_loop, spawn_watcher}`
  - 事件名常量 `events::{EV_PROCESSES, EV_SYSTEM_INFO, EV_CLIPBOARD, EV_FILES_CHANGED, EV_TRAY_OPEN, EV_TRAY_QUIT, EV_NOTICE}`

- [ ] **Step 1: 写 `src-tauri/src/state.rs`**

```rust
//! 全局共享状态。
//!
//! 所有 command 与后台线程共享同一个 `AppState`（通过 `tauri::State`），
//! 内部用 `Mutex` 保护可变部分，采样间隔与网络开关用原子变量以便热改。

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex};

use notify::Watcher;

use crate::core::clipboard::ClipboardManager;
use crate::core::files::FileManager;
use crate::core::i18n::I18n;
use crate::core::monitor::SystemMonitor;
use crate::core::plugin::PluginManager;
use crate::core::process::ProcessManager;
use crate::core::settings::Settings;

/// 应用共享状态。
pub struct AppState {
    /// 全局设置。
    pub settings: Mutex<Settings>,
    /// 文案表。
    pub i18n: Mutex<I18n>,
    /// 进程管理器。
    pub processes: Mutex<ProcessManager>,
    /// 文件管理器。
    pub files: Mutex<FileManager>,
    /// 剪贴板管理器。
    pub clipboard: Mutex<ClipboardManager>,
    /// 系统监控。
    pub monitor: Mutex<SystemMonitor>,
    /// 插件管理器。
    pub plugins: Mutex<PluginManager>,
    /// 采样间隔（毫秒），可在设置里实时调整。
    pub sample_interval_ms: Arc<AtomicU64>,
    /// 是否统计网络流量。
    pub network_enabled: Arc<AtomicBool>,
    /// 当前目录的文件变更监听器。
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

impl AppState {
    /// 用磁盘上的设置构造状态。
    pub fn new(settings: Settings) -> Self {
        let sample_interval_ms = Arc::new(AtomicU64::new(
            (settings.auto_refresh_interval * 1000.0) as u64,
        ));
        let network_enabled = Arc::new(AtomicBool::new(settings.enable_network_monitor));

        let locale = settings.locale;
        let mut clipboard = ClipboardManager::default();
        clipboard.load_history();

        let mut plugins = PluginManager::default();
        plugins.load_plugins();

        Self {
            i18n: Mutex::new(I18n::new(locale)),
            processes: Mutex::new(ProcessManager::default()),
            files: Mutex::new(FileManager::default()),
            clipboard: Mutex::new(clipboard),
            monitor: Mutex::new(SystemMonitor::default()),
            plugins: Mutex::new(plugins),
            sample_interval_ms,
            network_enabled,
            watcher: Mutex::new(None),
            settings: Mutex::new(settings),
        }
    }
}
```

- [ ] **Step 2: 写 `src-tauri/src/system/tasks.rs`**

```rust
//! 后台任务循环。
//!
//! 所有周期性与监听型后台任务都在这里启动。线程只做取数与派发，
//! 不直接操作界面；界面通过 Tauri 事件收到更新。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{Event, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

use crate::core::clipboard;
use crate::core::monitor;
use crate::events::{EV_CLIPBOARD, EV_FILES_CHANGED, EV_PROCESSES, EV_SYSTEM_INFO};
use crate::state::AppState;

/// 启动进程列表定时刷新循环。
pub fn spawn_process_loop(app: AppHandle, state: Arc<Mutex<AppState>>) {
    std::thread::spawn(move || loop {
        let (limit, interval_ms) = {
            let guard = state.lock().expect("状态锁中毒");
            let settings = guard.settings.lock().expect("设置锁中毒");
            (settings.process_list_limit, guard.sample_interval_ms.load(Ordering::Relaxed))
        };

        let rows = {
            let mut guard = state.lock().expect("状态锁中毒");
            let mut processes = guard.processes.lock().expect("进程锁中毒");
            processes
                .collect(limit)
                .iter()
                .map(crate::commands::dto::ProcessRow::from)
                .collect::<Vec<_>>()
        };

        let _ = app.emit(EV_PROCESSES, rows);

        std::thread::sleep(Duration::from_millis(interval_ms.max(500)));
    });
}

/// 启动剪贴板轮询循环。
pub fn spawn_clipboard_loop(app: AppHandle, state: Arc<Mutex<AppState>>) {
    std::thread::spawn(move || {
        let mut last = String::new();
        loop {
            std::thread::sleep(Duration::from_secs(1));

            let Some(text) = clipboard::read_current() else {
                continue;
            };
            if text == last {
                continue;
            }
            last = text.clone();

            {
                let guard = state.lock().expect("状态锁中毒");
                let mut manager = guard.clipboard.lock().expect("剪贴板锁中毒");
                manager.add_to_history(&text, "text");
            }

            let _ = app.emit(EV_CLIPBOARD, text);
        }
    });
}

/// 启动系统指标采样循环。
pub fn spawn_monitor_loop(
    app: AppHandle,
    state: Arc<Mutex<AppState>>,
    interval_ms: Arc<AtomicU64>,
    network_enabled: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let mut system = sysinfo::System::new_all();
        // 预热一次，让 CPU 使用率有可比较的基线。
        system.refresh_cpu_usage();

        // 网络计数器必须复用同一个实例：每次新建会让接口基线被重置，
        // 算出来的上传 / 下载速度永远是 0。
        let mut networks = sysinfo::Networks::new_with_refreshed_list();

        let mut first = true;
        loop {
            let interval = (interval_ms.load(Ordering::Relaxed) as f64 / 1000.0).max(0.5);

            if !first {
                std::thread::sleep(Duration::from_secs_f64(interval));
            }
            first = false;

            let info = monitor::sample(
                &mut system,
                &mut networks,
                interval,
                network_enabled.load(Ordering::Relaxed),
            );

            let dto = {
                let guard = state.lock().expect("状态锁中毒");
                let mut mon = guard.monitor.lock().expect("监控锁中毒");
                mon.push_sample(info);
                crate::commands::dto::SystemInfoDto {
                    cpu_usage: mon.system_info.cpu_usage,
                    memory_usage: mon.system_info.memory_usage,
                    disk_usage: mon.system_info.disk_usage,
                    network_upload: mon.system_info.network_upload,
                    network_download: mon.system_info.network_download,
                    cpu_history: mon.cpu_history.clone(),
                    memory_history: mon.memory_history.clone(),
                }
            };

            let _ = app.emit(EV_SYSTEM_INFO, dto);
        }
    });
}

/// 启动当前目录的变更监听。
pub fn spawn_watcher(app: AppHandle, state: Arc<Mutex<AppState>>) {
    let path = {
        let guard = state.lock().expect("状态锁中毒");
        let files = guard.files.lock().expect("文件锁中毒");
        files.current_path.clone()
    };

    let target = path.clone();
    let watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
        if let Ok(event) = result {
            if event.kind == notify::EventKind::Remove(_) {
                let _ = app.emit(EV_FILES_CHANGED, ());
            }
        }
    });

    let Ok(mut watcher) = watcher else {
        eprintln!("[watcher] 无法创建目录监听器");
        return;
    };

    if let Err(err) = watcher.watch(&target, RecursiveMode::NonRecursive) {
        eprintln!("[watcher] 监听 {} 失败: {err}", target.display());
        return;
    }

    let guard = state.lock().expect("状态锁中毒");
    *guard.watcher.lock().expect("监听锁中毒") = Some(watcher);
}
```

- [ ] **Step 3: 写 `src-tauri/src/system/mod.rs`**

```rust
//! 原生集成：菜单栏、托盘、快捷键、自启、激活策略与后台任务。

pub mod activation;
pub mod autolaunch;
pub mod menu;
pub mod shortcut;
pub mod tasks;
pub mod tray;
```

> `activation.rs` / `autolaunch.rs` / `menu.rs` / `shortcut.rs` / `tray.rs` 在 Task 12–15 创建。本步骤**先只写** `pub mod tasks;`，其余在对应任务补齐（否则编译不过）。

- [ ] **Step 4: 在 `events.rs` 追加事件名常量**

```rust
/// 进程列表更新事件。
pub const EV_PROCESSES: &str = "processes";
/// 系统指标更新事件。
pub const EV_SYSTEM_INFO: &str = "system-info";
/// 剪贴板内容变化事件。
pub const EV_CLIPBOARD: &str = "clipboard";
/// 当前目录内容变化事件。
pub const EV_FILES_CHANGED: &str = "files-changed";
/// 托盘请求打开主窗口。
pub const EV_TRAY_OPEN: &str = "tray-open";
/// 托盘请求退出应用。
pub const EV_TRAY_QUIT: &str = "tray-quit";
/// 界面提示事件。
pub const EV_NOTICE: &str = "notice";
```

- [ ] **Step 5: 接入 `lib.rs`，让应用能真正跑起来**

```rust
//! chuaker7 —— macOS 系统工具箱（Tauri v2 + React 重写版）。
//!
//! 业务逻辑在 `core/`，Tauri 命令门面在 `commands/`，原生集成在 `system/`。

pub mod commands;
pub mod core;
pub mod events;
pub mod state;
pub mod system;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use state::AppState;

/// Tauri Builder 装配入口。
pub fn run() {
    let settings = crate::core::settings::Settings::load();
    let app_state = AppState::new(settings);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Mutex::new(app_state)))
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<Arc<Mutex<AppState>>>().inner().clone();

            crate::system::tasks::spawn_process_loop(handle.clone(), state.clone());
            crate::system::tasks::spawn_clipboard_loop(handle.clone(), state.clone());
            crate::system::tasks::spawn_monitor_loop(
                handle.clone(),
                state.clone(),
                state.sample_interval_ms.clone(),
                state.network_enabled.clone(),
            );
            crate::system::tasks::spawn_watcher(handle.clone(), state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running chuaker7");
}
```

> `app.state::<Arc<Mutex<AppState>>>()` 需要 `tauri::Manager` trait 的 `state()` 方法；`Arc<Mutex<AppState>>` 本身实现了 `Send + Sync + 'static`，可作为 Tauri 状态。

- [ ] **Step 6: 编译验证**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
```

Expected：编译通过，0 warning。

> 若 `tauri::State` 无法从 `Arc<Mutex<AppState>>` 取值，改为 `manage(Mutex::new(app_state))` 并在 command 里用 `state.0.lock()`。本步骤先按上面的写法试；`cargo build` 的真实报错为准。

- [ ] **Step 7: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 引入 AppState 与后台任务循环

- state.rs 汇总设置/文案/进程/文件/剪贴板/监控/插件共享状态
- system/tasks.rs 接管进程定时刷新、剪贴板轮询、指标采样、目录监听
- 事件名统一收敛为 events.rs 的常量，避免字符串散落

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 8: 进程命令

**Files:**
- Create: `src-tauri/src/commands/processes.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `state::AppState`、`commands::dto::ProcessRow`
- Produces: command `list_processes` / `refresh_processes` / `kill_process`

- [ ] **Step 1: 写 `src-tauri/src/commands/processes.rs`**

```rust
//! 进程管理命令。

use std::sync::{Arc, Mutex};

use tauri::State;

use crate::commands::dto::ProcessRow;
use crate::core::process::ProcessManager;
use crate::state::AppState;

/// 列出当前进程列表。
#[tauri::command]
pub fn list_processes(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<ProcessRow> {
    let guard = state.lock().expect("状态锁中毒");
    let processes = guard.processes.lock().expect("进程锁中毒");
    processes
        .processes
        .iter()
        .map(ProcessRow::from)
        .collect()
}

/// 触发一次进程列表刷新（同步采集后立即返回）。
#[tauri::command]
pub fn refresh_processes(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<ProcessRow> {
    let guard = state.lock().expect("状态锁中毒");
    let limit = guard
        .settings
        .lock()
        .expect("设置锁中毒")
        .process_list_limit;
    let mut processes = guard.processes.lock().expect("进程锁中毒");
    processes
        .collect(limit)
        .iter()
        .map(ProcessRow::from)
        .collect()
}

/// 结束指定进程。
#[tauri::command]
pub fn kill_process(pid: i32) -> Result<bool, String> {
    if ProcessManager::kill_process(pid) {
        Ok(true)
    } else {
        Err(format!("结束进程 {pid} 失败，可能需要管理员权限"))
    }
}
```

> `refresh_processes` 先做同步采集（前端拿不到实时数据时至少有一次即时反馈），周期刷新由 `system::tasks::spawn_process_loop` 负责。

- [ ] **Step 2: 挂到 invoke_handler**

`src-tauri/src/commands/mod.rs` 追加：

```rust
pub mod processes;

use tauri::ipc::InvokeHandler;

/// 汇总所有 command。
pub fn handler() -> InvokeHandler {
    tauri::generate_handler![
        processes::list_processes,
        processes::refresh_processes,
        processes::kill_process,
    ]
}
```

`src-tauri/src/lib.rs` 的 Builder 追加 `.invoke_handler(commands::handler())`。

- [ ] **Step 3: 编译并提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
git add -A
git commit -m "feat: 进程管理命令

- list_processes / refresh_processes / kill_process 三个 command
- 周期刷新仍由后台循环负责，命令侧提供一次性即时采集

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 9: 文件命令与目录监听重建

**Files:**
- Create: `src-tauri/src/commands/files.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `core::files::FileManager`、`core::platform`、`state::AppState`
- Produces: command `list_dir` / `create_folder` / `create_file` / `rename_path` / `delete_to_trash` / `reveal_in_finder` / `parent_dir` / `copy_text`

- [ ] **Step 1: 写 `src-tauri/src/commands/files.rs`**

```rust
//! 文件管理命令。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::{RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, State};

use crate::commands::dto::FileRow;
use crate::core::platform;
use crate::events::EV_FILES_CHANGED;
use crate::state::AppState;

/// 列出指定目录的内容。
#[tauri::command]
pub fn list_dir(path: String) -> Vec<FileRow> {
    let dir = PathBuf::from(path);
    let mut manager = crate::core::files::FileManager {
        current_path: dir,
        files: Vec::new(),
        selected: None,
        is_loading: false,
    };
    manager.load_files();
    manager.files.iter().map(FileRow::from).collect()
}

/// 新建文件夹。
#[tauri::command]
pub fn create_folder(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    dir: String,
    name: String,
) -> Result<(), String> {
    let created = {
        let guard = state.lock().expect("状态锁中毒");
        let mut files = guard.files.lock().expect("文件锁中毒");
        files.navigate_to(PathBuf::from(dir));
        files.create_folder(&name)
    };

    if created {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err(format!("创建文件夹「{name}」失败"))
    }
}

/// 新建空文件。
#[tauri::command]
pub fn create_file(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    dir: String,
    name: String,
) -> Result<(), String> {
    let created = {
        let guard = state.lock().expect("状态锁中毒");
        let mut files = guard.files.lock().expect("文件锁中毒");
        files.navigate_to(PathBuf::from(dir));
        files.create_file(&name)
    };

    if created {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err(format!("创建文件「{name}」失败"))
    }
}

/// 重命名文件或文件夹。
#[tauri::command]
pub fn rename_path(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    path: String,
    new_name: String,
) -> Result<(), String> {
    let renamed = {
        let guard = state.lock().expect("状态锁中毒");
        let mut files = guard.files.lock().expect("文件锁中毒");
        files.rename_file(Path::new(&path), &new_name)
    };

    if renamed {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err(format!("重命名为「{new_name}」失败"))
    }
}

/// 移到废纸篓。
#[tauri::command]
pub fn delete_to_trash(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    path: String,
) -> Result<(), String> {
    let deleted = {
        let guard = state.lock().expect("状态锁中毒");
        let mut files = guard.files.lock().expect("文件锁中毒");
        files.delete_file(Path::new(&path))
    };

    if deleted {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err("移到废纸篓失败".to_string())
    }
}

/// 在 Finder 中定位。
#[tauri::command]
pub fn reveal_in_finder(path: String) {
    platform::reveal_in_finder(Path::new(&path));
}

/// 返回上级目录，根目录返回 None。
#[tauri::command]
pub fn parent_dir(path: String) -> Option<String> {
    Path::new(&path)
        .parent()
        .filter(|parent| *parent != Path::new(&path))
        .map(|parent| parent.to_string_lossy().to_string())
}

/// 复制文本到系统剪贴板。
#[tauri::command]
pub fn copy_text(text: String) {
    platform::copy_to_clipboard(&text);
}

/// 用户主目录（文件页的初始目录）。
#[tauri::command]
pub fn home_dir() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/"))
        .to_string_lossy()
        .to_string()
}
```

- [ ] **Step 2: 实现目录切换时的监听重建**

在 `src-tauri/src/commands/files.rs` 末尾追加：

```rust
/// 切换监听目录并重新挂载监听器。
#[tauri::command]
pub fn watch_dir(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    path: String,
) {
    let target = PathBuf::from(path);

    let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result {
            if event.kind == notify::EventKind::Remove(_) {
                let _ = app.emit(EV_FILES_CHANGED, ());
            }
        }
    });

    let Ok(mut watcher) = watcher else {
        return;
    };
    if watcher.watch(&target, RecursiveMode::NonRecursive).is_err() {
        return;
    }

    let guard = state.lock().expect("状态锁中毒");
    *guard.watcher.lock().expect("监听锁中毒") = Some(watcher);
}
```

并把 `watch_dir` 加入契约（前端在切换目录后调用）。

- [ ] **Step 3: 挂到 invoke_handler 并提交**

`commands/mod.rs` 追加 `pub mod files;` 与：

```rust
        files::list_dir,
        files::create_folder,
        files::create_file,
        files::rename_path,
        files::delete_to_trash,
        files::reveal_in_finder,
        files::parent_dir,
        files::copy_text,
        files::home_dir,
        files::watch_dir,
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
git add -A
git commit -m "feat: 文件管理命令与目录监听重建

- list_dir / create_folder / create_file / rename_path / delete_to_trash 等命令
- watch_dir 在切换目录时重建 notify 监听器
- 变更后统一发 files-changed 事件驱动前端刷新

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 10: 剪贴板、监控与插件命令

**Files:**
- Create: `src-tauri/src/commands/clipboard.rs`
- Create: `src-tauri/src/commands/monitor.rs`
- Create: `src-tauri/src/commands/plugins.rs`
- Modify: `src-tauri/src/commands/mod.rs`

**Interfaces:**
- Consumes: `core::clipboard`、`core::monitor`、`core::plugin`、`state::AppState`
- Produces: command `clipboard_current` / `clipboard_history` / `remove_history_item` / `clear_history` / `system_snapshot` / `list_plugins` / `reload_plugins`

- [ ] **Step 1: 写 `src-tauri/src/commands/clipboard.rs`**

```rust
//! 剪贴板命令。

use std::sync::{Arc, Mutex};

use tauri::State;

use crate::commands::dto::HistoryRow;
use crate::core::clipboard;
use crate::state::AppState;

/// 读取当前剪贴板文本。
#[tauri::command]
pub fn clipboard_current() -> String {
    clipboard::read_current().unwrap_or_default()
}

/// 读取剪贴板历史。
#[tauri::command]
pub fn clipboard_history(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<HistoryRow> {
    let guard = state.lock().expect("状态锁中毒");
    let manager = guard.clipboard.lock().expect("剪贴板锁中毒");
    manager.history.iter().map(HistoryRow::from).collect()
}

/// 删除一条历史（按稳定 id）。
#[tauri::command]
pub fn remove_history_item(
    state: State<'_, Arc<Mutex<AppState>>>,
    id: u64,
) {
    let guard = state.lock().expect("状态锁中毒");
    let mut manager = guard.clipboard.lock().expect("剪贴板锁中毒");
    manager.delete_history_item(id);
}

/// 清空历史。
#[tauri::command]
pub fn clear_history(state: State<'_, Arc<Mutex<AppState>>>) {
    let guard = state.lock().expect("状态锁中毒");
    let mut manager = guard.clipboard.lock().expect("剪贴板锁中毒");
    manager.clear_history();
}
```

- [ ] **Step 2: 写 `src-tauri/src/commands/monitor.rs`**

```rust
//! 系统监控命令。

use std::sync::{Arc, Mutex};

use tauri::State;

use crate::commands::dto::SystemInfoDto;
use crate::state::AppState;

/// 读取一次系统指标快照（含采样历史）。
#[tauri::command]
pub fn system_snapshot(state: State<'_, Arc<Mutex<AppState>>>) -> SystemInfoDto {
    let guard = state.lock().expect("状态锁中毒");
    let monitor = guard.monitor.lock().expect("监控锁中毒");
    SystemInfoDto {
        cpu_usage: monitor.system_info.cpu_usage,
        memory_usage: monitor.system_info.memory_usage,
        disk_usage: monitor.system_info.disk_usage,
        network_upload: monitor.system_info.network_upload,
        network_download: monitor.system_info.network_download,
        cpu_history: monitor.cpu_history.clone(),
        memory_history: monitor.memory_history.clone(),
    }
}
```

- [ ] **Step 3: 写 `src-tauri/src/commands/plugins.rs`**

```rust
//! 插件命令。

use std::sync::{Arc, Mutex};

use tauri::State;

use crate::commands::dto::PluginRow;
use crate::core::plugin::PluginManager;
use crate::state::AppState;

/// 列出已加载的插件。
#[tauri::command]
pub fn list_plugins(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<PluginRow> {
    let guard = state.lock().expect("状态锁中毒");
    let plugins = guard.plugins.lock().expect("插件锁中毒");
    let dir = plugins.plugins_dir().to_string_lossy().to_string();
    plugins
        .loaded_plugins
        .iter()
        .map(|manifest| {
            let mut row = PluginRow::from(manifest);
            row.dir = dir.clone();
            row
        })
        .collect()
}

/// 重新扫描插件目录。
#[tauri::command]
pub fn reload_plugins(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<PluginRow> {
    let guard = state.lock().expect("状态锁中毒");
    let mut plugins = guard.plugins.lock().expect("插件锁中毒");
    plugins.load_plugins();
    let dir = plugins.plugins_dir().to_string_lossy().to_string();
    plugins
        .loaded_plugins
        .iter()
        .map(|manifest| {
            let mut row = PluginRow::from(manifest);
            row.dir = dir.clone();
            row
        })
        .collect()
}

/// 插件目录路径。
#[tauri::command]
pub fn plugins_dir() -> String {
    PluginManager::default().plugins_dir().to_string_lossy().to_string()
}
```

> `PluginManager::default()` 不读盘，仅用于取目录常量，安全。

- [ ] **Step 4: 挂到 invoke_handler 并提交**

`commands/mod.rs` 追加三个 `pub mod`，以及 handler 列表：

```rust
        clipboard::clipboard_current,
        clipboard::clipboard_history,
        clipboard::remove_history_item,
        clipboard::clear_history,
        monitor::system_snapshot,
        plugins::list_plugins,
        plugins::reload_plugins,
        plugins::plugins_dir,
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
git add -A
git commit -m "feat: 剪贴板 / 监控 / 插件命令

- 剪贴板历史按稳定 id 删除，读取接口返回 DTO
- system_snapshot 带上 CPU / 内存采样历史供前端画曲线
- 插件命令补 plugins_dir，前端可直接展示目录路径

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 11: 设置、文案与系统命令

**Files:**
- Create: `src-tauri/src/commands/settings.rs`
- Create: `src-tauri/src/commands/system.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `core::settings::{Settings, SettingsPatch}`、`core::i18n::Locale`
- Produces: command `get_settings` / `update_settings` / `get_strings` / `set_menu_bar_mode` / `set_shortcut_enabled` / `set_auto_launch` / `hide_main_window` / `show_main_window` / `popup_context_menu`（本任务注册空实现，完整菜单逻辑在 Task 25 落地）

- [ ] **Step 1: 写 `src-tauri/src/commands/settings.rs`**

```rust
//! 设置与文案命令。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};

use crate::core::i18n::{I18n, Locale};
use crate::core::settings::{Settings, SettingsPatch};
use crate::state::AppState;

/// 读取当前设置。
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<Mutex<AppState>>>) -> Settings {
    let guard = state.lock().expect("状态锁中毒");
    let settings = guard.settings.lock().expect("设置锁中毒");
    settings.clone()
}

/// 提交设置补丁，返回合并后的完整设置。
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    patch: SettingsPatch,
) -> Settings {
    let next = {
        let guard = state.lock().expect("状态锁中毒");
        let mut settings = guard.settings.lock().expect("设置锁中毒");
        let next = patch.apply(&settings);
        next.save();
        *settings = next.clone();
        next
    };

    // 语言变了要换文案表。
    {
        let guard = state.lock().expect("状态锁中毒");
        let mut i18n = guard.i18n.lock().expect("文案锁中毒");
        if i18n.locale() != next.locale {
            i18n.set_locale(next.locale);
        }
    }

    // 采样间隔与网络开关热更新。
    {
        let guard = state.lock().expect("状态锁中毒");
        guard
            .sample_interval_ms
            .store((next.auto_refresh_interval * 1000.0) as u64, std::sync::atomic::Ordering::Relaxed);
        guard
            .network_enabled
            .store(next.enable_network_monitor, std::sync::atomic::Ordering::Relaxed);
    }

    // 菜单栏模式与全局快捷键的变化需要立即作用到原生层。
    let previous = {
        let guard = state.lock().expect("状态锁中毒");
        guard.settings.lock().expect("设置锁中毒").clone()
    };

    if previous.show_menu_bar != next.show_menu_bar {
        crate::system::activation::set_accessory_mode(next.show_menu_bar, &app);
    }

    if previous.global_shortcut_enabled != next.global_shortcut_enabled {
        crate::system::shortcut::set_enabled(&app, next.global_shortcut_enabled);
    }

    next
}

/// 取指定语言的整张文案表。
#[tauri::command]
pub fn get_strings(locale: String) -> HashMap<String, String> {
    let parsed = match locale.as_str() {
        "en" => Locale::En,
        _ => Locale::ZhHans,
    };
    I18n::new(parsed).table()
}
```

- [ ] **Step 2: 写 `src-tauri/src/commands/system.rs`**

```rust
//! 系统级命令：窗口显隐、菜单栏模式、快捷键、自启。

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

/// 隐藏主窗口。
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// 显示主窗口并置前。
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// 切换菜单栏模式（隐藏 Dock 图标）。
#[tauri::command]
pub fn set_menu_bar_mode(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    enabled: bool,
) {
    {
        let guard = state.lock().expect("状态锁中毒");
        let mut settings = guard.settings.lock().expect("设置锁中毒");
        settings.show_menu_bar = enabled;
        settings.save();
    }

    crate::system::activation::set_accessory_mode(enabled, &app);

    if enabled {
        let _ = app.get_webview_window("main").map(|w| w.hide());
    } else {
        let _ = app.get_webview_window("main").map(|w| w.show());
    }
}

/// 启用或停用全局快捷键。
#[tauri::command]
pub fn set_shortcut_enabled(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    enabled: bool,
) {
    {
        let guard = state.lock().expect("状态锁中毒");
        let mut settings = guard.settings.lock().expect("设置锁中毒");
        settings.global_shortcut_enabled = enabled;
        settings.save();
    }

    crate::system::shortcut::set_enabled(&app, enabled);
}

/// 启用或停用开机自启。
#[tauri::command]
pub fn set_auto_launch(enabled: bool) -> Result<(), String> {
    if enabled {
        crate::system::autolaunch::enable().map_err(|err| err.to_string())
    } else {
        crate::system::autolaunch::disable().map_err(|err| err.to_string())
    }
}

/// 弹出原生右键菜单（Task 25 实现完整逻辑）。
#[tauri::command]
pub fn popup_context_menu(app: AppHandle, kind: String, payload: serde_json::Value) {
    let _ = (app, kind, payload);
}
```

- [ ] **Step 3: 挂到 invoke_handler 并提交**

`commands/mod.rs` 追加 `pub mod settings;` / `pub mod system;` 与 handler 列表：

```rust
        settings::get_settings,
        settings::update_settings,
        settings::get_strings,
        system::hide_main_window,
        system::show_main_window,
        system::set_menu_bar_mode,
        system::set_shortcut_enabled,
        system::set_auto_launch,
        system::popup_context_menu,
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
git add -A
git commit -m "feat: 设置、文案与系统命令

- get_settings / update_settings 采用补丁合并语义，语言变更同步换文案表
- 采样间隔与网络开关热更新到原子变量，无需重启线程
- 窗口显隐、菜单栏模式、快捷键、自启命令接入原生层

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 12: 原生应用菜单栏

**Files:**
- Create: `src-tauri/src/system/menu.rs`
- Modify: `src-tauri/src/system/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/events.rs`

**Interfaces:**
- Consumes: `core::i18n`（菜单文案）、`tauri::menu::*`
- Produces: `system::menu::build(app: &AppHandle) -> tauri::Result<()>`；事件 `menu-action`（负载 `String`）

- [ ] **Step 1: 追加事件常量**

`src-tauri/src/events.rs` 追加：

```rust
/// 原生菜单动作事件（前端据此切换标签页 / 打开设置）。
pub const EV_MENU_ACTION: &str = "menu-action";
```

- [ ] **Step 2: 写 `src-tauri/src/system/menu.rs`**

```rust
//! 原生应用菜单栏。
//!
//! 使用 Tauri 的 `Menu::set_as_app_menu()` 构建真正的 macOS 菜单栏：
//! 关于、设置、隐藏、退出走 `PredefinedMenuItem`（系统原生行为），
//! 视图切换走带 `Cmd+数字` 加速器的自定义项。

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter};

use crate::events::EV_MENU_ACTION;

/// 构建并安装应用菜单栏。
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let app_menu = Submenu::with_items(
        app,
        "chuaker7",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("chuaker7"), None)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "app:settings", "设置…", true, Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, Some("隐藏 chuaker7"), Some("CmdOrCtrl+H"))?,
            &PredefinedMenuItem::hide_others(app, Some("隐藏其他"), Some("CmdOrCtrl+Alt+H"))?,
            &PredefinedMenuItem::show_all(app, Some("显示全部"), None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("退出 chuaker7"), Some("CmdOrCtrl+Q"))?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "编辑",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let view_menu = Submenu::with_items(
        app,
        "视图",
        true,
        &[
            &MenuItem::with_id(app, "nav:processes", "进程管理", true, Some("CmdOrCtrl+1"))?,
            &MenuItem::with_id(app, "nav:files", "文件管理", true, Some("CmdOrCtrl+2"))?,
            &MenuItem::with_id(app, "nav:clipboard", "剪贴板", true, Some("CmdOrCtrl+3"))?,
            &MenuItem::with_id(app, "nav:monitor", "系统监控", true, Some("CmdOrCtrl+4"))?,
            &MenuItem::with_id(app, "nav:plugins", "插件", true, Some("CmdOrCtrl+5"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "nav:settings", "设置", true, Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;

    let window_menu = Submenu::with_items(
        app,
        "窗口",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &app_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
        ],
    )?;

    menu.set_as_app_menu()?;
    Ok(())
}

/// 处理菜单项点击：把自定义动作转成前端事件。
pub fn on_menu_event(app: &AppHandle, id: &str) {
    if id.starts_with("nav:") || id == "app:settings" {
        let _ = app.emit(EV_MENU_ACTION, id);
    }
}
```

- [ ] **Step 3: 接入 Builder**

`src-tauri/src/lib.rs`：

```rust
use tauri::Manager;
```

Builder 追加：

```rust
        .setup(|app| {
            let handle = app.handle().clone();

            crate::system::menu::build(&handle)?;

            let state = app.state::<Arc<Mutex<AppState>>>().inner().clone();
            crate::system::tasks::spawn_process_loop(handle.clone(), state.clone());
            crate::system::tasks::spawn_clipboard_loop(handle.clone(), state.clone());
            crate::system::tasks::spawn_monitor_loop(
                handle.clone(),
                state.clone(),
                state.sample_interval_ms.clone(),
                state.network_enabled.clone(),
            );
            crate::system::tasks::spawn_watcher(handle.clone(), state);

            Ok(())
        })
        .on_menu_event(|app, event| {
            crate::system::menu::on_menu_event(app, event.id().as_ref());
        })
```

- [ ] **Step 4: 编译并人工验收**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7
npm run tauri dev
```

Expected：窗口出现后，屏幕顶部出现 macOS 菜单栏，含「chuaker7 / 编辑 / 视图 / 窗口」四个下拉；点「视图 → 进程管理」无报错。

- [ ] **Step 5: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 原生 macOS 应用菜单栏

- set_as_app_menu 构建 chuaker7/编辑/视图/窗口四个原生菜单
- 关于/隐藏/退出/剪切/拷贝/粘贴/全选走 PredefinedMenuItem
- 视图项带 Cmd+1..5 加速器，点击后发 menu-action 事件驱动前端切换

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 13: 托盘图标与托盘菜单

**Files:**
- Create: `src-tauri/src/system/tray.rs`
- Modify: `src-tauri/src/system/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `core::icon::bird_rgba`、`tauri::tray::TrayIconBuilder`、`tauri::menu::*`
- Produces: `system::tray::{build, update_metrics, on_tray_event}`；事件 `tray-open` / `tray-quit`

- [ ] **Step 1: 写 `src-tauri/src/system/tray.rs`**

```rust
//! 托盘图标与托盘菜单。
//!
//! 图标沿用 egui 版的「小鸟」像素画（`core::icon::bird_rgba`），
//! 以 macOS 模板图方式挂载，系统自动适配深浅色。

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::core::icon::bird_rgba;
use crate::events::{EV_TRAY_OPEN, EV_TRAY_QUIT};

/// 托盘里需要动态刷新的菜单项。
struct TrayItems<R: Runtime> {
    cpu: MenuItem<R>,
    memory: MenuItem<R>,
    disk: MenuItem<R>,
    upload: MenuItem<R>,
    download: MenuItem<R>,
}

/// 构建托盘图标与菜单。
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let (pixels, width, height) = bird_rgba(22);
    let icon = Image::new_owned(pixels, width, height);

    let title = MenuItem::with_id(app, "tray:title", "chuaker7", false, None::<&str>)?;
    let cpu = MenuItem::with_id(app, "tray:cpu", "CPU  —", false, None::<&str>)?;
    let memory = MenuItem::with_id(app, "tray:memory", "内存  —", false, None::<&str>)?;
    let disk = MenuItem::with_id(app, "tray:disk", "磁盘  —", false, None::<&str>)?;
    let upload = MenuItem::with_id(app, "tray:upload", "上传  —", false, None::<&str>)?;
    let download = MenuItem::with_id(app, "tray:download", "下载  —", false, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &title,
            &cpu,
            &memory,
            &disk,
            &upload,
            &download,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "tray:open", "打开 chuaker7", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some("退出"), None)?,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("chuaker7")
        .menu_on_left_click(false)
        .menu(&menu)
        .build(app)?;

    // 把可更新的菜单项挂到全局状态上，供采样循环更新。
    app.manage(TrayItems {
        cpu,
        memory,
        disk,
        upload,
        download,
    });

    Ok(())
}

/// 更新托盘里的实时指标。
pub fn update_metrics<R: Runtime>(app: &AppHandle<R>, info: &crate::core::monitor::SystemInfo) {
    let Some(items) = app.try_state::<TrayItems<R>>() else {
        return;
    };
    let _ = items.cpu.set_text(format!("CPU  {:.1}%", info.cpu_usage));
    let _ = items.memory.set_text(format!("内存  {:.1}%", info.memory_usage));
    let _ = items.disk.set_text(format!("磁盘  {:.1}%", info.disk_usage));
    let _ = items
        .upload
        .set_text(format!("上传  {:.2} MB/s", info.network_upload));
    let _ = items
        .download
        .set_text(format!("下载  {:.2} MB/s", info.network_download));
}

/// 处理托盘点击与菜单点击。
pub fn on_tray_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_main_window(app);
        let _ = app.emit(EV_TRAY_OPEN, ());
    }
}

/// 处理托盘菜单项点击。
pub fn on_tray_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "tray:open" => {
            show_main_window(app);
            let _ = app.emit(EV_TRAY_OPEN, ());
        }
        "tray:quit" => {
            let _ = app.emit(EV_TRAY_QUIT, ());
            app.exit(0);
        }
        _ => {}
    }
}

/// 显示并置前主窗口。
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
```

- [ ] **Step 2: 接入 Builder 与采样循环**

`src-tauri/src/lib.rs` 的 setup 追加：

```rust
            crate::system::tray::build(&handle)?;
```

Builder 追加：

```rust
        .on_tray_icon_event(|tray, event| {
            crate::system::tray::on_tray_event(tray.app_handle(), &event);
        })
```

`system/tasks.rs` 的 `spawn_monitor_loop` 内，emit 之前追加：

```rust
            crate::system::tray::update_metrics(&app, &info);
```

- [ ] **Step 3: 编译并人工验收**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7
npm run tauri dev
```

Expected：菜单栏右侧出现小鸟图标；左键点击弹出菜单，显示 CPU / 内存 / 磁盘 / 上传 / 下载 五行数值（随采样刷新）；点「打开 chuaker7」窗口置前。

- [ ] **Step 4: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 托盘图标与原生托盘菜单

- 复用 core::icon 的小鸟像素画，以 macOS 模板图挂载
- 托盘菜单实时显示 CPU/内存/磁盘/上传/下载
- 左键打开主窗口、菜单退出走 app.exit

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 14: 全局快捷键与激活策略

**Files:**
- Create: `src-tauri/src/system/shortcut.rs`
- Create: `src-tauri/src/system/activation.rs`
- Modify: `src-tauri/src/system/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `global-hotkey 0.10`、`objc2-app-kit 0.3`
- Produces: `system::shortcut::{register, set_enabled}`、`system::activation::set_accessory_mode`

- [ ] **Step 1: 写 `src-tauri/src/system/shortcut.rs`**

```rust
//! 全局快捷键 ⌘+L。
//!
//! macOS 上 `GlobalHotKeyManager` 必须在主线程创建，否则注册会失败，
//! 因此注册放在 Tauri 的 `setup()` 里（setup 运行在主线程）。

use std::sync::Mutex;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri::{AppHandle, Manager, Runtime};

/// 热键管理器（全局唯一）。
static MANAGER: Mutex<Option<GlobalHotKeyManager>> = Mutex::new(None);
/// 已注册的热键。
static HOTKEY: Mutex<Option<HotKey>> = Mutex::new(None);

/// 注册 ⌘+L 并启动监听线程。
pub fn register<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let manager = GlobalHotKeyManager::new().map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;
    let hotkey = HotKey::new(Some(Modifiers::META), Code::KeyL);
    let id = hotkey.id();
    manager
        .register(hotkey)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!(e)))?;

    *MANAGER.lock().expect("快捷键锁中毒") = Some(manager);
    *HOTKEY.lock().expect("快捷键锁中毒") = Some(hotkey);

    let handle = app.clone();
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if event.id == id && event.state == global_hotkey::HotKeyState::Pressed {
                crate::system::tray::show_main_window(&handle);
            }
        }
    });

    Ok(())
}

/// 启用或停用快捷键。
pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, enabled: bool) {
    if enabled {
        let _ = register(app);
    } else {
        let hotkey = HOTKEY.lock().expect("快捷键锁中毒").take();
        if let (Some(hotkey), Some(manager)) = (
            hotkey,
            MANAGER.lock().expect("快捷键锁中毒").as_ref(),
        ) {
            let _ = manager.unregister(hotkey);
        }
        *MANAGER.lock().expect("快捷键锁中毒") = None;
    }
}
```

- [ ] **Step 2: 写 `src-tauri/src/system/activation.rs`**

```rust
//! Dock / 菜单栏模式切换。
//!
//! Tauri v2 没有暴露 `NSApp.activationPolicy`，这里用 `objc2-app-kit` 直调 AppKit。
//! 必须在主线程调用；失败时降级为「仅隐藏窗口」，行为差异可接受。

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use tauri::{AppHandle, Runtime};

/// 设置激活策略：`true` 为仅菜单栏（不占 Dock），`false` 为常规（占 Dock）。
pub fn set_accessory_mode<R: Runtime>(enabled: bool, _app: &AppHandle<R>) {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("[activation] 不在主线程，跳过激活策略切换");
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let policy = if enabled {
        NSApplicationActivationPolicy::Accessory
    } else {
        NSApplicationActivationPolicy::Regular
    };

    if !app.setActivationPolicy(policy) {
        eprintln!("[activation] 切换激活策略失败，降级为仅隐藏窗口");
    }
}
```

- [ ] **Step 3: 接入 setup**

`src-tauri/src/lib.rs` setup 内追加：

```rust
            {
                let guard = state.lock().expect("状态锁中毒");
                let shortcut_enabled = guard
                    .settings
                    .lock()
                    .expect("设置锁中毒")
                    .global_shortcut_enabled;
                if shortcut_enabled {
                    let _ = crate::system::shortcut::register(&handle);
                }
            }
```

- [ ] **Step 4: 编译并人工验收**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
npm run tauri dev
```

Expected：
- ⌘L 能把窗口从后台唤到前台。
- 在设置里打开「菜单栏模式」后，Dock 图标消失，菜单栏仍有托盘图标；关掉后 Dock 图标回来。

- [ ] **Step 5: 提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "feat: 全局快捷键 ⌘L 与 Dock/菜单栏模式切换

- global-hotkey 在主线程注册，⌘L 唤起主窗口
- objc2-app-kit 直调 NSApplication.setActivationPolicy 实现仅菜单栏模式
- 切换失败时降级为仅隐藏窗口并打印日志

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 15: 开机自启

**Files:**
- Create: `src-tauri/src/system/autolaunch.rs`
- Modify: `src-tauri/src/system/mod.rs`

**Interfaces:**
- Consumes: 无
- Produces: `system::autolaunch::{enable, disable, is_enabled, plist_path}`

- [ ] **Step 1: 写 `src-tauri/src/system/autolaunch.rs`**

```rust
//! 开机自启动。
//!
//! Swift 版优先使用 `SMAppService`（macOS 13+），并保留 LaunchAgent 传统实现。
//! Rust 版统一走 LaunchAgent：在 `~/Library/LaunchAgents` 写 plist 并用 `launchctl` 装载。

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
```

- [ ] **Step 2: 注册模块并提交**

`src-tauri/src/system/mod.rs` 补齐全部模块声明：

```rust
//! 原生集成：菜单栏、托盘、快捷键、自启、激活策略与后台任务。

pub mod activation;
pub mod autolaunch;
pub mod menu;
pub mod shortcut;
pub mod tasks;
pub mod tray;
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo test -p chuaker7 && cargo clippy -p chuaker7 --all-targets
git add -A
git commit -m "feat: 开机自启（LaunchAgent）

- 写 ~/Library/LaunchAgents/com.igigi.chuaker7.plist 并用 launchctl 装载
- 补 XML 转义与 plist 路径单元测试

AI-Coding: true
AI-Tool: TophantAI"
```

---

## P3 前端页面

### Task 16: 前端基础层（类型 / 桥接 / 状态 / 工具函数）

**Files:**
- Create: `frontend/src/lib/types.ts`
- Create: `frontend/src/lib/bridge.ts`
- Create: `frontend/src/lib/store.ts`
- Create: `frontend/src/lib/theme.ts`
- Create: `frontend/src/lib/format.ts`
- Create: `frontend/src/lib/i18n.ts`
- Create: `frontend/src/lib/format.test.ts`
- Create: `frontend/src/lib/i18n.test.ts`

**Interfaces:**
- Consumes: Task 6 的 DTO 契约、Task 11 的 command 清单
- Produces:
  - `types.ts`：`ProcessRow` / `PortRow` / `FileRow` / `HistoryRow` / `SystemInfoDto` / `PluginRow` / `Settings` / `SettingsPatch` / `Notice` / `TabId`
  - `bridge.ts`：`api`（全部 command 的薄封装）、`onEvent(name, handler)`、`EVENT` 常量表
  - `store.ts`：`appStore`（`useSyncExternalStore` 容器）、`useAppState()`、`useAppStore()`
  - `theme.ts`：`resolveTheme(mode)`、`applyTheme(mode, accent)`
  - `format.ts`：`formatBytes` / `formatTimestamp` / `formatPercent` / `formatRate`
  - `i18n.ts`：`interpolate(template, args)`、`useI18n()`

- [ ] **Step 1: 写 `frontend/src/lib/types.ts`**

```typescript
// 与 src-tauri/src/commands/dto.rs 一一对应。

export interface PortRow {
  address: string;
  port: number;
}

export interface ProcessRow {
  pid: number;
  displayName: string;
  fullPath: string;
  ports: PortRow[];
}

export interface FileRow {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
  sizeText: string;
  modified: number;
}

export interface HistoryRow {
  id: number;
  content: string;
  timestamp: number;
  kind: string;
}

export interface SystemInfoDto {
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
  networkUpload: number;
  networkDownload: number;
  cpuHistory: number[];
  memoryHistory: number[];
}

export interface PluginRow {
  id: string;
  name: string;
  version: string;
  author: string | null;
  description: string | null;
  minimumAppVersion: string | null;
  permissions: string[];
  main: string | null;
  dir: string;
}

export type ThemeMode = 'system' | 'light' | 'dark';
export type Accent = 'signal' | 'haze' | 'moss' | 'amber';
export type Locale = 'zh-Hans' | 'en';

export interface Settings {
  showMenuBar: boolean;
  autoRefreshInterval: number;
  processListLimit: number;
  enableNetworkMonitor: boolean;
  themeMode: ThemeMode;
  accent: Accent;
  launchAtLogin: boolean;
  globalShortcutEnabled: boolean;
  locale: Locale;
}

export type SettingsPatch = Partial<Settings>;

export interface Notice {
  kind: 'info' | 'error';
  message: string;
}

export type TabId = 'processes' | 'files' | 'clipboard' | 'monitor' | 'plugins' | 'settings';
```

- [ ] **Step 2: 写 `frontend/src/lib/bridge.ts`**

```typescript
// Tauri 桥接层：所有 invoke / listen 收敛在这里，页面不直接接触 @tauri-apps/api。

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

import type {
  FileRow,
  HistoryRow,
  PluginRow,
  ProcessRow,
  Settings,
  SettingsPatch,
  SystemInfoDto,
} from './types';

/** 事件名常量，与 src-tauri/src/events.rs 一一对应。 */
export const EVENT = {
  processes: 'processes',
  systemInfo: 'system-info',
  clipboard: 'clipboard',
  filesChanged: 'files-changed',
  trayOpen: 'tray-open',
  trayQuit: 'tray-quit',
  notice: 'notice',
  menuAction: 'menu-action',
} as const;

/** 全部 command 的薄封装。 */
export const api = {
  listProcesses: () => invoke<ProcessRow[]>('list_processes'),
  refreshProcesses: () => invoke<ProcessRow[]>('refresh_processes'),
  killProcess: (pid: number) => invoke<boolean>('kill_process', { pid }),

  listDir: (path: string) => invoke<FileRow[]>('list_dir', { path }),
  createFolder: (dir: string, name: string) => invoke<void>('create_folder', { dir, name }),
  createFile: (dir: string, name: string) => invoke<void>('create_file', { dir, name }),
  renamePath: (path: string, newName: string) =>
    invoke<void>('rename_path', { path, newName }),
  deleteToTrash: (path: string) => invoke<void>('delete_to_trash', { path }),
  revealInFinder: (path: string) => invoke<void>('reveal_in_finder', { path }),
  parentDir: (path: string) => invoke<string | null>('parent_dir', { path }),
  copyText: (text: string) => invoke<void>('copy_text', { text }),
  homeDir: () => invoke<string>('home_dir'),
  watchDir: (path: string) => invoke<void>('watch_dir', { path }),

  clipboardCurrent: () => invoke<string>('clipboard_current'),
  clipboardHistory: () => invoke<HistoryRow[]>('clipboard_history'),
  removeHistoryItem: (id: number) => invoke<void>('remove_history_item', { id }),
  clearHistory: () => invoke<void>('clear_history'),

  systemSnapshot: () => invoke<SystemInfoDto>('system_snapshot'),

  listPlugins: () => invoke<PluginRow[]>('list_plugins'),
  reloadPlugins: () => invoke<PluginRow[]>('reload_plugins'),
  pluginsDir: () => invoke<string>('plugins_dir'),

  getSettings: () => invoke<Settings>('get_settings'),
  updateSettings: (patch: SettingsPatch) => invoke<Settings>('update_settings', { patch }),
  getStrings: (locale: string) => invoke<Record<string, string>>('get_strings', { locale }),

  hideMainWindow: () => invoke<void>('hide_main_window'),
  showMainWindow: () => invoke<void>('show_main_window'),
  setMenuBarMode: (enabled: boolean) => invoke<void>('set_menu_bar_mode', { enabled }),
  setShortcutEnabled: (enabled: boolean) => invoke<void>('set_shortcut_enabled', { enabled }),
  setAutoLaunch: (enabled: boolean) => invoke<void>('set_auto_launch', { enabled }),
  popupContextMenu: (kind: string, payload: unknown) =>
    invoke<void>('popup_context_menu', { kind, payload }),

  /** 弹出 macOS 原生目录选择框，取消时返回 null。 */
  pickDirectory: async (): Promise<string | null> => {
    const selected = await open({ directory: true, multiple: false });
    return typeof selected === 'string' ? selected : null;
  },
};

/** 订阅一个后端事件，返回取消订阅函数。 */
export function onEvent<T>(name: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(name, (event) => handler(event.payload));
}
```

- [ ] **Step 3: 写 `frontend/src/lib/store.ts`**

```typescript
// 约 60 行的轻量状态容器：useSyncExternalStore + 不可变更新。

import { useSyncExternalStore } from 'react';

import type { Notice, Settings, TabId } from './types';

export interface AppState {
  /** 完整设置，未加载时为 null。 */
  settings: Settings | null;
  /** 文案表。 */
  strings: Record<string, string>;
  /** 当前标签页。 */
  activeTab: TabId;
  /** 顶部提示。 */
  toast: (Notice & { id: number }) | null;
  /** 界面语言。 */
  locale: 'zh-Hans' | 'en';
}

const initialState: AppState = {
  settings: null,
  strings: {},
  activeTab: 'processes',
  toast: null,
  locale: 'zh-Hans',
};

let state: AppState = initialState;
const listeners = new Set<() => void>();

function getSnapshot(): AppState {
  return state;
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** 合并式更新全局状态。 */
export function setState(patch: Partial<AppState> | ((prev: AppState) => Partial<AppState>)): void {
  const delta = typeof patch === 'function' ? patch(state) : patch;
  state = { ...state, ...delta };
  listeners.forEach((listener) => listener());
}

/** 读取全局状态（订阅式）。 */
export function useAppState(): AppState {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

/** 读取并更新全局状态。 */
export function useAppStore(): [AppState, typeof setState] {
  return [useAppState(), setState];
}

/** 弹出顶部提示；3 秒后自动消失。 */
export function pushToast(kind: Notice['kind'], message: string): void {
  const id = Date.now();
  setState((prev) => ({ toast: { kind, message, id } }));
  window.setTimeout(() => {
    setState((prev) => (prev.toast?.id === id ? { toast: null } : {}));
  }, 3000);
}
```

- [ ] **Step 4: 写 `frontend/src/lib/theme.ts`**

```typescript
import type { Accent, ThemeMode } from './types';

/** 把「跟随系统」解析为具体的深/浅色。 */
export function resolveTheme(mode: ThemeMode): 'dark' | 'light' {
  if (mode === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return mode;
}

/** 把主题与强调色写到 <html> 的 data 属性上，CSS 变量随之生效。 */
export function applyTheme(mode: ThemeMode, accent: Accent): void {
  const root = document.documentElement;
  root.dataset.theme = resolveTheme(mode);
  root.dataset.accent = accent;
}
```

- [ ] **Step 5: 写 `frontend/src/lib/format.ts`**

```typescript
const UNITS = ['bytes', 'KB', 'MB', 'GB', 'TB', 'PB'] as const;

/** 字节数格式化，与 Rust 侧 format_bytes 口径一致。 */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} bytes`;
  }
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(1)} ${UNITS[unit]}`;
}

/** 秒级 UNIX 时间 → `MM-DD HH:mm`。 */
export function formatTimestamp(seconds: number): string {
  if (!seconds) {
    return '—';
  }
  const date = new Date(seconds * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(
    date.getMinutes(),
  )}`;
}

/** 百分比。 */
export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

/** 网络速率。 */
export function formatRate(value: number): string {
  return `${value.toFixed(2)} MB/s`;
}
```

- [ ] **Step 6: 写 `frontend/src/lib/i18n.ts`**

```typescript
// 前端插值：与 Rust 侧 format_placeholders 口径一致，按顺序替换 %@。

import { useAppState } from './store';

/** 把模板中的 `%@` 按出现顺序替换为给定参数。 */
export function interpolate(template: string, args: string[]): string {
  let out = '';
  let rest = template;
  let index = 0;

  for (;;) {
    const pos = rest.indexOf('%@');
    if (pos < 0) {
      break;
    }
    out += rest.slice(0, pos);
    out += args[index] ?? '%@';
    index += 1;
    rest = rest.slice(pos + 2);
  }

  return out + rest;
}

/** 取文案；缺失时返回 key 本身（与 NSLocalizedString 行为一致）。 */
export function translate(
  strings: Record<string, string>,
  key: string,
  args: string[] = [],
): string {
  const template = strings[key] ?? key;
  return args.length > 0 ? interpolate(template, args) : template;
}

/** 在组件里取文案函数。 */
export function useI18n(): {
  t: (key: string, args?: string[]) => string;
  strings: Record<string, string>;
} {
  const { strings } = useAppState();
  return {
    strings,
    t: (key: string, args: string[] = []) => translate(strings, key, args),
  };
}
```

- [ ] **Step 7: 写测试**

`frontend/src/lib/format.test.ts`：

```typescript
import { describe, expect, it } from 'vitest';

import { formatBytes, formatPercent, formatRate, formatTimestamp } from './format';

describe('formatBytes', () => {
  it('小于 1KB 显示原始字节', () => {
    expect(formatBytes(0)).toBe('0 bytes');
    expect(formatBytes(512)).toBe('512 bytes');
  });

  it('按 1024 进位', () => {
    expect(formatBytes(2048)).toBe('2.0 KB');
    expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MB');
    expect(formatBytes(3 * 1024 ** 3)).toBe('3.0 GB');
  });
});

describe('formatTimestamp', () => {
  it('空值显示占位符', () => {
    expect(formatTimestamp(0)).toBe('—');
  });

  it('输出 MM-DD HH:mm', () => {
    expect(formatTimestamp(1700000000)).toMatch(/^\d{2}-\d{2} \d{2}:\d{2}$/);
  });
});

describe('数值格式', () => {
  it('百分比保留一位小数', () => {
    expect(formatPercent(12.34)).toBe('12.3%');
  });

  it('速率保留两位小数', () => {
    expect(formatRate(1.005)).toBe('1.00 MB/s');
  });
});
```

`frontend/src/lib/i18n.test.ts`：

```typescript
import { describe, expect, it } from 'vitest';

import { interpolate, translate } from './i18n';

describe('interpolate', () => {
  it('按出现顺序替换 %@', () => {
    expect(interpolate('结束进程 %@ (%@)?', ['Finder', '42'])).toBe('结束进程 Finder (42)?');
  });

  it('参数不足时保留占位符', () => {
    expect(interpolate('结束进程 %@', [])).toBe('结束进程 %@');
  });
});

describe('translate', () => {
  it('命中 key 返回文案', () => {
    expect(translate({ 'tab.process': '进程管理' }, 'tab.process')).toBe('进程管理');
  });

  it('缺失 key 返回 key 本身', () => {
    expect(translate({}, 'missing.key')).toBe('missing.key');
  });

  it('带参数时自动插值', () => {
    expect(translate({ 'process.count': '共 %@ 个进程' }, 'process.count', ['12'])).toBe('共 12 个进程');
  });
});
```

- [ ] **Step 8: 跑测试并提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 前端基础层 —— 类型、桥接、状态与工具函数

- types.ts 镜像 Rust DTO，bridge.ts 收敛全部 invoke / listen
- store.ts 用 useSyncExternalStore 实现约 60 行状态容器
- format / i18n 提供纯函数实现并配套单元测试

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 17: 通用组件

**Files:**
- Create: `frontend/src/components/VirtualList.tsx`
- Create: `frontend/src/components/DataTable.tsx`
- Create: `frontend/src/components/SearchField.tsx`
- Create: `frontend/src/components/Kicker.tsx`
- Create: `frontend/src/components/Card.tsx`
- Create: `frontend/src/components/Checkbox.tsx`
- Create: `frontend/src/components/Segmented.tsx`
- Create: `frontend/src/components/Modal.tsx`
- Create: `frontend/src/components/Sparkline.tsx`
- Create: `frontend/src/components/VirtualList.test.tsx`
- Create: `frontend/src/components/Modal.test.tsx`

**Interfaces:**
- Consumes: Task 3 的 CSS 变量、Task 16 的工具函数
- Produces: 9 个可复用组件，props 见下

- [ ] **Step 1: 写 `VirtualList.tsx`（固定行高虚拟滚动）**

```tsx
import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

export interface VirtualListProps<T> {
  /** 数据源。 */
  items: T[];
  /** 行高（px），固定值。 */
  rowHeight?: number;
  /** 视口外额外渲染的行数。 */
  overscan?: number;
  /** 容器高度（px）。 */
  height: number;
  /** 行渲染函数。 */
  renderRow: (item: T, index: number) => ReactNode;
  /** 行 key。 */
  getKey: (item: T, index: number) => string;
  /** 空态内容。 */
  empty?: ReactNode;
}

/**
 * 固定行高的虚拟列表。
 *
 * 500 行只渲染约 30 个 DOM 节点（26px 行高 + overscan 8），
 * 保证滚动不掉帧。
 */
export function VirtualList<T>({
  items,
  rowHeight = 26,
  overscan = 8,
  height,
  renderRow,
  getKey,
  empty,
}: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const frameRef = useRef<number | null>(null);

  const onScroll = useCallback((event: React.UIEvent<HTMLDivElement>) => {
    const top = event.currentTarget.scrollTop;
    if (frameRef.current !== null) {
      cancelAnimationFrame(frameRef.current);
    }
    frameRef.current = requestAnimationFrame(() => {
      setScrollTop(top);
    });
  }, []);

  useEffect(
    () => () => {
      if (frameRef.current !== null) {
        cancelAnimationFrame(frameRef.current);
      }
    },
    [],
  );

  const { start, end, offsetY, totalHeight } = useMemo(() => {
    const visible = Math.ceil(height / rowHeight);
    const first = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
    const last = Math.min(items.length, first + visible + overscan * 2);
    return {
      start: first,
      end: last,
      offsetY: first * rowHeight,
      totalHeight: items.length * rowHeight,
    };
  }, [height, items.length, overscan, rowHeight, scrollTop]);

  if (items.length === 0) {
    return (
      <div className="scrollable" style={{ height, overflowY: 'auto' }}>
        <div className="empty-state">{empty ?? '暂无数据'}</div>
      </div>
    );
  }

  const slice = items.slice(start, end);

  return (
    <div className="scrollable" style={{ height, overflowY: 'auto' }} onScroll={onScroll}>
      <div style={{ height: totalHeight, position: 'relative' }}>
        <div style={{ position: 'absolute', top: offsetY, left: 0, right: 0 }}>
          {slice.map((item, index) => {
            const realIndex = start + index;
            return (
              <div key={getKey(item, realIndex)} style={{ height: rowHeight }}>
                {renderRow(item, realIndex)}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 写 `DataTable.tsx`**

```tsx
import type { ReactNode } from 'react';

export interface Column<T> {
  /** 列 key。 */
  key: string;
  /** 表头文案。 */
  title: string;
  /** 列宽，`remainder` 表示占满剩余空间。 */
  width?: number | 'remainder';
  /** 单元格内容。 */
  render: (item: T, index: number) => ReactNode;
  /** 右对齐。 */
  align?: 'left' | 'right';
}

export interface DataTableProps<T> {
  /** 列定义。 */
  columns: Column<T>[];
  /** 数据源。 */
  items: T[];
  /** 行 key。 */
  getKey: (item: T, index: number) => string;
  /** 行点击（可选）。 */
  onRowClick?: (item: T, index: number) => void;
  /** 行右键（可选，Task 25 接入原生菜单）。 */
  onRowContextMenu?: (item: T, index: number, event: React.MouseEvent) => void;
  /** 选中行的 key。 */
  selectedKey?: string | null;
  /** 空态内容。 */
  empty?: ReactNode;
}

/** 发丝描边的数据表格。 */
export function DataTable<T>({
  columns,
  items,
  getKey,
  onRowClick,
  onRowContextMenu,
  selectedKey,
  empty,
}: DataTableProps<T>) {
  return (
    <table className="data-table">
      <colgroup>
        {columns.map((column) => (
          <col
            key={column.key}
            style={
              column.width === 'remainder'
                ? undefined
                : { width: `${column.width ?? 100}px`, flex: '0 0 auto' }
            }
          />
        ))}
      </colgroup>
      <thead>
        <tr>
          {columns.map((column) => (
            <th key={column.key} style={{ textAlign: column.align ?? 'left' }}>
              {column.title}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {items.length === 0 ? (
          <tr>
            <td colSpan={columns.length} className="empty-state">
              {empty ?? '暂无数据'}
            </td>
          </tr>
        ) : (
          items.map((item, index) => {
            const key = getKey(item, index);
            return (
              <tr
                key={key}
                data-selected={selectedKey === key ? 'true' : undefined}
                onClick={onRowClick ? () => onRowClick(item, index) : undefined}
                onContextMenu={
                  onRowContextMenu
                    ? (event) => {
                        event.preventDefault();
                        onRowContextMenu(item, index, event);
                      }
                    : undefined
                }
              >
                {columns.map((column) => (
                  <td key={column.key} style={{ textAlign: column.align ?? 'left' }}>
                    {column.render(item, index)}
                  </td>
                ))}
              </tr>
            );
          })
        )}
      </tbody>
    </table>
  );
}
```

- [ ] **Step 3: 写 `SearchField.tsx`**

```tsx
export interface SearchFieldProps {
  /** 当前搜索词。 */
  value: string;
  /** 输入变化回调。 */
  onChange: (value: string) => void;
  /** 占位文案。 */
  placeholder?: string;
}

/** 搜索框：原生 input，保留系统 IME 与拼写检查。 */
export function SearchField({ value, onChange, placeholder }: SearchFieldProps) {
  return (
    <div className="search-field">
      <span className="search-field__icon" aria-hidden="true">
        ⌕
      </span>
      <input
        className="native-input"
        type="search"
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        spellCheck={false}
      />
    </div>
  );
}
```

- [ ] **Step 4: 写 `Kicker.tsx` / `Card.tsx` / `Checkbox.tsx` / `Segmented.tsx`**

```tsx
// Kicker.tsx
export interface KickerProps {
  /** 编号，如 "01"。 */
  index?: string;
  /** 标题。 */
  children: React.ReactNode;
}

export function Kicker({ index, children }: KickerProps) {
  return (
    <div className="section-title">
      {index ? <span className="kicker">{index}</span> : null}
      <span className="kicker">◆</span>
      <span className="kicker">{children}</span>
    </div>
  );
}
```

```tsx
// Card.tsx
export interface CardProps {
  /** 卡片标题。 */
  title?: React.ReactNode;
  /** 右上角操作区。 */
  actions?: React.ReactNode;
  children?: React.ReactNode;
  className?: string;
}

export function Card({ title, actions, children, className }: CardProps) {
  return (
    <section className={`card ${className ?? ''}`}>
      {title || actions ? (
        <header className="card__head">
          <span className="card__title">{title}</span>
          {actions ? <div className="card__actions">{actions}</div> : null}
        </header>
      ) : null}
      <div className="card__body">{children}</div>
    </section>
  );
}
```

```tsx
// Checkbox.tsx
export interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: React.ReactNode;
  disabled?: boolean;
}

/** macOS 方块复选框。 */
export function Checkbox({ checked, onChange, label, disabled }: CheckboxProps) {
  return (
    <label className="checkbox">
      <input
        className="native-checkbox"
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      {label ? <span className="checkbox__label">{label}</span> : null}
    </label>
  );
}
```

```tsx
// Segmented.tsx
export interface SegmentedOption<T extends string> {
  value: T;
  label: string;
}

export interface SegmentedProps<T extends string> {
  options: SegmentedOption<T>[];
  value: T;
  onChange: (value: T) => void;
}

/** macOS 分段控件。 */
export function Segmented<T extends string>({ options, value, onChange }: SegmentedProps<T>) {
  return (
    <div className="segmented" role="group">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          className="segmented__item"
          aria-pressed={option.value === value}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
```

- [ ] **Step 5: 写 `Modal.tsx`**

```tsx
import { useEffect, type ReactNode } from 'react';

export interface ModalProps {
  open: boolean;
  title: string;
  children: ReactNode;
  /** 底部按钮。 */
  footer?: ReactNode;
  onClose: () => void;
}

/** 半透明遮罩 + 圆角卡片，Esc 关闭。 */
export function Modal({ open, title, children, footer, onClose }: ModalProps) {
  useEffect(() => {
    if (!open) {
      return;
    }
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={onClose} role="presentation">
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal__head">{title}</header>
        <div className="modal__body">{children}</div>
        {footer ? <footer className="modal__foot">{footer}</footer> : null}
      </div>
    </div>
  );
}
```

- [ ] **Step 6: 写 `Sparkline.tsx`（Canvas 手绘曲线）**

```tsx
import { useEffect, useRef } from 'react';

export interface SparklineProps {
  /** 采样序列。 */
  values: number[];
  /** 画布宽（CSS px）。 */
  width?: number;
  /** 画布高（CSS px）。 */
  height?: number;
  /** 线条颜色，缺省取当前强调色。 */
  color?: string;
  /** 上限。 */
  max?: number;
}

/** Canvas 手绘曲线，避免 SVG 逐点重排。 */
export function Sparkline({ values, width = 240, height = 48, color, max = 100 }: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    if (values.length < 2) {
      return;
    }

    const stroke = color ?? getComputedStyle(canvas).getPropertyValue('--accent').trim() ?? '#F43D3F';
    const step = width / (values.length - 1);
    const y = (value: number) => height - (Math.min(value, max) / max) * height;

    ctx.beginPath();
    ctx.moveTo(0, y(values[0]));
    values.forEach((value, index) => {
      ctx.lineTo(index * step, y(value));
    });
    ctx.strokeStyle = stroke;
    ctx.lineWidth = 1.25;
    ctx.stroke();

    ctx.lineTo(width, height);
    ctx.lineTo(0, height);
    ctx.closePath();
    ctx.fillStyle = `${stroke}1f`;
    ctx.fill();
  }, [values, width, height, color, max]);

  return <canvas ref={canvasRef} className="sparkline" style={{ width, height }} />;
}
```

- [ ] **Step 7: 写组件测试**

`frontend/src/components/VirtualList.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { VirtualList } from './VirtualList';

describe('VirtualList', () => {
  const items = Array.from({ length: 500 }, (_, index) => index);

  it('只渲染视口附近的行，而不是全部 500 行', () => {
    render(
      <VirtualList
        items={items}
        height={260}
        rowHeight={26}
        overscan={8}
        getKey={(item) => String(item)}
        renderRow={(item) => <span>行 {item}</span>}
      />,
    );

    // 26px 行高 + 260px 视口 + overscan 8 → 渲染约 28 行。
    expect(screen.getByText('行 0')).toBeInTheDocument();
    expect(screen.queryByText('行 499')).not.toBeInTheDocument();
    expect(screen.getAllByText(/^行 \d+$/).length).toBeLessThan(60);
  });

  it('空数据时展示空态', () => {
    render(
      <VirtualList
        items={[]}
        height={200}
        getKey={(item: number) => String(item)}
        renderRow={(item: number) => <span>{item}</span>}
        empty="没有匹配的进程"
      />,
    );
    expect(screen.getByText('没有匹配的进程')).toBeInTheDocument();
  });
});
```

`frontend/src/components/Modal.test.tsx`：

```tsx
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Modal } from './Modal';

describe('Modal', () => {
  it('关闭时不渲染', () => {
    render(
      <Modal open={false} title="确认" onClose={() => {}}>
        内容
      </Modal>,
    );
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('点遮罩触发 onClose', () => {
    const onClose = vi.fn();
    render(
      <Modal open title="确认" onClose={onClose}>
        内容
      </Modal>,
    );
    fireEvent.click(screen.getByRole('presentation'));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Esc 触发 onClose', () => {
    const onClose = vi.fn();
    render(
      <Modal open title="确认" onClose={onClose}>
        内容
      </Modal>,
    );
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
```

- [ ] **Step 8: 补组件样式并提交**

在 `frontend/src/styles/base.css` 末尾追加：

```css
/* 表格 */
.data-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  font-size: 12px;
}

.data-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--surface);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 400;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  text-align: left;
  padding: 6px 10px;
  border-bottom: 1px solid var(--border);
}

.data-table td {
  padding: 0 10px;
  height: var(--row-height);
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.data-table tbody tr:hover {
  background: var(--surface-alt);
}

.data-table tbody tr[data-selected='true'] {
  background: color-mix(in srgb, var(--accent) 16%, transparent);
}

/* 搜索框 */
.search-field {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 220px;
}

.search-field__icon {
  color: var(--text-muted);
  font-size: 12px;
}

/* 卡片 */
.card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
}

.card__title {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.card__body {
  padding: 10px 12px;
}

/* 复选框 */
.checkbox {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: default;
}

/* 弹窗 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.42);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.modal {
  width: 360px;
  max-width: calc(100vw - 48px);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 18px 48px rgba(0, 0, 0, 0.45);
  overflow: hidden;
}

.modal__head {
  padding: 10px 14px;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

.modal__body {
  padding: 14px;
  font-size: 13px;
}

.modal__foot {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 14px;
  border-top: 1px solid var(--border);
}

/* 空态 */
.empty-state {
  padding: 28px 12px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

/* 曲线 */
.sparkline {
  display: block;
}
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 前端通用组件

- VirtualList 固定行高虚拟滚动，500 行只渲染约 28 个节点
- DataTable / SearchField / Kicker / Card / Checkbox / Segmented / Modal
- Sparkline 用 Canvas 手绘曲线，避免 SVG 逐点重排
- 配套虚拟列表与弹窗的组件测试

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 18: 应用外壳（侧栏 / 顶栏 / 提示 / 导航）

**Files:**
- Create: `frontend/src/app/navigation.ts`
- Create: `frontend/src/app/Sidebar.tsx`
- Create: `frontend/src/app/TopBar.tsx`
- Create: `frontend/src/app/Toast.tsx`
- Create: `frontend/src/app/AppShell.tsx`
- Modify: `frontend/src/App.tsx`
- Modify: `frontend/index.html`
- Create: `frontend/src/app/AppShell.test.tsx`

**Interfaces:**
- Consumes: Task 16 的 `store` / `i18n` / `theme`，Task 17 的组件
- Produces: `navigation.ts` 的 `TABS` 列表与 `TAB_IDS`；`AppShell` 组件；`App.tsx` 挂载 `AppShell`

- [ ] **Step 1: 写 `navigation.ts`**

```typescript
import type { TabId } from '../lib/types';

export interface TabDef {
  id: TabId;
  /** 侧栏文案 key。 */
  labelKey: string;
  /** 侧栏图标（几何符号，呼应设计语言的 ◆）。 */
  glyph: string;
  /** 分组：工具 / 系统。 */
  group: 'tools' | 'system';
}

/** 六个标签页的定义，顺序即侧栏顺序。 */
export const TABS: TabDef[] = [
  { id: 'processes', labelKey: 'nav.process', glyph: '◆', group: 'tools' },
  { id: 'files', labelKey: 'nav.file', glyph: '◆', group: 'tools' },
  { id: 'clipboard', labelKey: 'nav.clipboard', glyph: '◆', group: 'tools' },
  { id: 'monitor', labelKey: 'nav.monitor', glyph: '◆', group: 'tools' },
  { id: 'plugins', labelKey: 'nav.plugins', glyph: '◆', group: 'system' },
  { id: 'settings', labelKey: 'nav.settings', glyph: '◆', group: 'system' },
];

/** 标签页 id 列表。 */
export const TAB_IDS: TabId[] = TABS.map((tab) => tab.id);
```

- [ ] **Step 2: 写 `Sidebar.tsx`**

```tsx
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import { TABS } from './navigation';

export function Sidebar() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();

  const tools = TABS.filter((tab) => tab.group === 'tools');
  const system = TABS.filter((tab) => tab.group === 'system');

  const renderGroup = (labelKey: string, items: typeof TABS) => (
    <div className="sidebar__group">
      <div className="sidebar__group-label">{t(labelKey)}</div>
      {items.map((tab) => (
        <button
          key={tab.id}
          type="button"
          className="sidebar__item"
          data-active={state.activeTab === tab.id ? 'true' : undefined}
          onClick={() => setState({ activeTab: tab.id })}
        >
          <span className="sidebar__glyph" aria-hidden="true">
            {tab.glyph}
          </span>
          <span>{t(tab.labelKey)}</span>
        </button>
      ))}
    </div>
  );

  return (
    <nav className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__brand-name">{t('app.brand')}</div>
        <div className="sidebar__brand-tag">{t('app.tagline')}</div>
      </div>
      {renderGroup('nav.section.tools', tools)}
      {renderGroup('nav.section.system', system)}
    </nav>
  );
}
```

- [ ] **Step 3: 写 `TopBar.tsx`**

```tsx
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import { TABS } from './navigation';

export function TopBar() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const current = TABS.find((tab) => tab.id === state.activeTab);

  return (
    <header className="topbar" data-tauri-drag-region>
      <div className="topbar__spacer" data-tauri-drag-region />
      <div className="topbar__title">
        <span className="kicker">{current ? t(current.labelKey) : ''}</span>
      </div>
      <div className="topbar__right" data-tauri-drag-region />
    </header>
  );
}
```

> `data-tauri-drag-region` 让顶栏可拖动窗口（需要在 `tauri.conf.json` 的 capabilities 里保留 `core:window:allow-start-dragging`）。

- [ ] **Step 4: 写 `Toast.tsx`**

```tsx
import { useAppStore } from '../lib/store';

export function Toast() {
  const [state] = useAppStore();
  if (!state.toast) {
    return null;
  }
  return (
    <div className={`toast toast--${state.toast.kind}`} role="status">
      {state.toast.message}
    </div>
  );
}
```

- [ ] **Step 5: 写 `AppShell.tsx`（负责启动引导与事件订阅）**

```tsx
import { useEffect } from 'react';

import { EVENT, api, onEvent } from '../lib/bridge';
import { applyTheme } from '../lib/theme';
import { pushToast, setState, useAppStore } from '../lib/store';
import { Sidebar } from './Sidebar';
import { Toast } from './Toast';
import { TopBar } from './TopBar';
import { Clipboard } from '../pages/Clipboard';
import { Files } from '../pages/Files';
import { Monitor } from '../pages/Monitor';
import { Plugins } from '../pages/Plugins';
import { Processes } from '../pages/Processes';
import { Settings } from '../pages/Settings';

const PAGES = {
  processes: Processes,
  files: Files,
  clipboard: Clipboard,
  monitor: Monitor,
  plugins: Plugins,
  settings: Settings,
} as const;

export function AppShell() {
  const [state] = useAppStore();

  // 启动引导：拉设置与文案，并应用主题。
  useEffect(() => {
    let disposed = false;

    void (async () => {
      const settings = await api.getSettings();
      const strings = await api.getStrings(settings.locale);
      if (disposed) {
        return;
      }
      applyTheme(settings.themeMode, settings.accent);
      setState({ settings, strings, locale: settings.locale });
    })();

    return () => {
      disposed = true;
    };
  }, []);

  // 订阅后端事件。onEvent 返回 Promise<UnlistenFn>，把取消函数收集起来在卸载时统一调用。
  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    const track = (promise: Promise<() => void>) => {
      void promise.then((unlisten) => unlisteners.push(unlisten));
    };

    track(
      onEvent<string>(EVENT.menuAction, (action) => {
        if (action.startsWith('nav:')) {
          setState({ activeTab: action.slice(4) as never });
        }
      }),
    );

    track(
      onEvent<{ kind: 'info' | 'error'; message: string }>(EVENT.notice, (notice) => {
        pushToast(notice.kind, notice.message);
      }),
    );

    track(onEvent(EVENT.trayOpen, () => setState({})));

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  const Page = PAGES[state.activeTab] ?? Processes;

  return (
    <div className="app-shell">
      <Sidebar />
      <div className="app-shell__main">
        <TopBar />
        <main className="app-shell__content">
          <Page />
        </main>
      </div>
      <Toast />
    </div>
  );
}
```

- [ ] **Step 6: 挂载 `App.tsx`**

```tsx
import { AppShell } from './app/AppShell';

export function App() {
  return <AppShell />;
}
```

同时给 `frontend/index.html` 的 `<html>` 补上默认主题属性：

```html
<html lang="zh-CN" data-theme="dark" data-accent="signal">
```

> `tokens.css` 里 `--accent` 只在 `[data-accent='signal|haze|moss|amber']` 下定义，`:root` **没有兜底值**。
> 若首帧不带 `data-accent`，`var(--accent)` 未定义，焦点环、选区、输入框聚焦、复选框选中与强调色按钮这 9 处消费点会同时失效。
> 这里把默认值直接写进 HTML，保证**首帧就有正确配色**；Step 5 的启动引导随后用设置里的实际值覆盖这两个属性。

- [ ] **Step 7: 补外壳样式**

在 `frontend/src/styles/base.css` 末尾追加：

```css
/* 应用外壳 */
.app-shell {
  display: flex;
  height: 100%;
}

.app-shell__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.app-shell__content {
  flex: 1;
  min-height: 0;
  padding: 14px 18px 18px;
  overflow: hidden;
}

/* 侧栏 */
.sidebar {
  width: var(--sidebar-width);
  flex: 0 0 var(--sidebar-width);
  border-right: 1px solid var(--border);
  padding: 14px 10px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: var(--surface);
}

.sidebar__brand-name {
  font-family: var(--font-display);
  font-size: 22px;
  letter-spacing: 0.08em;
  color: var(--text);
}

.sidebar__brand-tag {
  font-family: var(--font-mono);
  font-size: 9px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-top: 2px;
}

.sidebar__group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar__group-label {
  font-family: var(--font-mono);
  font-size: 9px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--text-muted);
  padding: 4px 8px;
}

.sidebar__item {
  appearance: none;
  border: 0;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: 12px;
  text-align: left;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  cursor: default;
  display: flex;
  align-items: center;
  gap: 7px;
}

.sidebar__item:hover {
  background: var(--surface-alt);
  color: var(--text);
}

.sidebar__item[data-active='true'] {
  background: var(--control);
  color: var(--text);
}

.sidebar__glyph {
  color: var(--accent);
  font-size: 8px;
}

/* 顶栏 */
.topbar {
  height: var(--topbar-height);
  flex: 0 0 var(--topbar-height);
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--border);
  padding: 0 12px 0 78px; /* 给红绿灯留位 */
}

.topbar__spacer {
  flex: 1;
  height: 100%;
}

.topbar__title {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 顶部提示 */
.toast {
  position: fixed;
  top: 14px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 14px;
  border-radius: var(--radius-md);
  background: var(--surface-alt);
  border: 1px solid var(--border);
  font-size: 12px;
  z-index: 60;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
}

.toast--error {
  border-color: var(--danger);
  color: var(--danger);
}
```

- [ ] **Step 8: 写测试并提交**

`frontend/src/app/AppShell.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: {
    processes: 'processes',
    systemInfo: 'system-info',
    clipboard: 'clipboard',
    filesChanged: 'files-changed',
    trayOpen: 'tray-open',
    trayQuit: 'tray-quit',
    notice: 'notice',
    menuAction: 'menu-action',
  },
  api: {
    getSettings: vi.fn().mockResolvedValue({
      showMenuBar: false,
      autoRefreshInterval: 5,
      processListLimit: 500,
      enableNetworkMonitor: true,
      themeMode: 'dark',
      accent: 'signal',
      launchAtLogin: false,
      globalShortcutEnabled: true,
      locale: 'zh-Hans',
    }),
    getStrings: vi.fn().mockResolvedValue({
      'nav.process': '进程管理',
      'nav.file': '文件管理',
      'nav.clipboard': '剪贴板',
      'nav.monitor': '系统监控',
      'nav.plugins': '插件',
      'nav.settings': '设置',
      'nav.section.tools': '工具',
      'nav.section.system': '系统',
      'app.brand': 'CHUAKER7',
      'app.tagline': 'MACOS 系统工具箱',
    }),
    // 默认激活的是进程页，AppShell 会把它挂载起来，
    // 页面在 useEffect 里调用的每个 api 方法都必须有桩，否则测试会抛
    // "api.xxx is not a function"。
    listProcesses: vi.fn().mockResolvedValue([]),
    refreshProcesses: vi.fn().mockResolvedValue([]),
    killProcess: vi.fn().mockResolvedValue(true),
    popupContextMenu: vi.fn().mockResolvedValue(undefined),
    listDir: vi.fn().mockResolvedValue([]),
    homeDir: vi.fn().mockResolvedValue('/tmp'),
    watchDir: vi.fn().mockResolvedValue(undefined),
    pickDirectory: vi.fn().mockResolvedValue(null),
    clipboardCurrent: vi.fn().mockResolvedValue(''),
    clipboardHistory: vi.fn().mockResolvedValue([]),
    systemSnapshot: vi.fn().mockResolvedValue({
      cpuUsage: 0,
      memoryUsage: 0,
      diskUsage: 0,
      networkUpload: 0,
      networkDownload: 0,
      cpuHistory: [],
      memoryHistory: [],
    }),
    listPlugins: vi.fn().mockResolvedValue([]),
    pluginsDir: vi.fn().mockResolvedValue('/tmp/plugins'),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'nav.process': '进程管理',
      'nav.file': '文件管理',
      'nav.clipboard': '剪贴板',
      'nav.monitor': '系统监控',
      'nav.plugins': '插件',
      'nav.settings': '设置',
      'nav.section.tools': '工具',
      'nav.section.system': '系统',
      'app.brand': 'CHUAKER7',
      'app.tagline': 'MACOS 系统工具箱',
    },
    activeTab: 'processes',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { AppShell } from './AppShell';

describe('AppShell', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染品牌与六个标签页', async () => {
    render(<AppShell />);
    expect(await screen.findByText('CHUAKER7')).toBeInTheDocument();
    for (const label of ['进程管理', '文件管理', '剪贴板', '系统监控', '插件', '设置']) {
      expect(await screen.findByText(label)).toBeInTheDocument();
    }
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 应用外壳 —— 侧栏 / 顶栏 / 提示与启动引导

- 侧栏按工具 / 系统两段呈现六个标签页
- 启动时拉取设置与文案并写入 data-theme / data-accent
- 订阅 menu-action 与 notice 事件，接管原生菜单与提示

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 19: 进程管理页

**Files:**
- Create: `frontend/src/pages/Processes.tsx`
- Create: `frontend/src/pages/Processes.test.tsx`
- Modify: `frontend/src/styles/base.css`

**Interfaces:**
- Consumes: `api.listProcesses` / `api.refreshProcesses` / `api.killProcess`；`EVENT.processes`
- Produces: `Processes` 组件（默认导出给 `AppShell` 的 `PAGES` 映射）

- [ ] **Step 1: 写 `frontend/src/pages/Processes.tsx`**

```tsx
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Card, Kicker, Modal, SearchField, Segmented, VirtualList } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { ProcessRow } from '../lib/types';

type SortKey = 'pid' | 'name' | 'port';

export function Processes() {
  const { t } = useI18n();
  const [rows, setRows] = useState<ProcessRow[]>([]);
  const [search, setSearch] = useState('');
  const [sortKey, setSortKey] = useState<SortKey>('pid');
  const [pendingKill, setPendingKill] = useState<ProcessRow | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);
  const [viewportHeight, setViewportHeight] = useState(420);

  // 首次拉取 + 订阅后端推送。
  useEffect(() => {
    let disposed = false;
    void api.listProcesses().then((list) => {
      if (!disposed) {
        setRows(list);
      }
    });
    void onEvent<ProcessRow[]>(EVENT.processes, (list) => setRows(list));
    return () => {
      disposed = true;
    };
  }, []);

  // 列表视口高度随窗口变化，虚拟列表按实际高度计算可见行数。
  useEffect(() => {
    const element = listRef.current;
    if (!element) {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry && entry.contentRect.height > 0) {
        setViewportHeight(entry.contentRect.height);
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return rows;
    }
    return rows.filter((row) => {
      if (row.displayName.toLowerCase().includes(keyword)) {
        return true;
      }
      if (String(row.pid).includes(keyword)) {
        return true;
      }
      return row.ports.some((port) => String(port.port).includes(keyword));
    });
  }, [rows, search]);

  const sorted = useMemo(() => {
    const copy = [...filtered];
    copy.sort((a, b) => {
      if (sortKey === 'name') {
        return a.displayName.localeCompare(b.displayName);
      }
      if (sortKey === 'port') {
        return (a.ports[0]?.port ?? 65535) - (b.ports[0]?.port ?? 65535);
      }
      return a.pid - b.pid;
    });
    return copy;
  }, [filtered, sortKey]);

  const onRefresh = useCallback(async () => {
    const list = await api.refreshProcesses();
    setRows(list);
  }, []);

  const confirmKill = useCallback(async () => {
    if (!pendingKill) {
      return;
    }
    const target = pendingKill;
    setPendingKill(null);
    try {
      await api.killProcess(target.pid);
      pushToast('info', t('process.kill.done', [target.displayName]));
      await onRefresh();
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [onRefresh, pendingKill, t]);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="01">{t('process.title')}</Kicker>
        <div className="page__tools">
          <SearchField
            value={search}
            onChange={setSearch}
            placeholder={t('process.search.placeholder')}
          />
          <button type="button" className="native-button" onClick={() => void onRefresh()}>
            {t('process.refresh')}
          </button>
        </div>
      </div>

      <Card
        title={t('process.count', [String(sorted.length)])}
        className="page__card"
        actions={
          <div className="page__tools">
            <Segmented<SortKey>
              value={sortKey}
              onChange={setSortKey}
              options={[
                { value: 'pid', label: t('process.pid') },
                { value: 'name', label: t('process.name') },
                { value: 'port', label: t('process.port') },
              ]}
            />
          </div>
        }
      >
        <div className="vtable" ref={listRef}>
          <div className="vrow vrow--head">
            <span className="vrow__pid">{t('process.pid')}</span>
            <span className="vrow__name">{t('process.name')}</span>
            <span className="vrow__port">{t('process.port')}</span>
          </div>
          <VirtualList
            items={sorted}
            height={viewportHeight}
            rowHeight={26}
            overscan={8}
            getKey={(row) => String(row.pid)}
            renderRow={(row) => (
              <div
                className="vrow"
                onClick={() => setPendingKill(row)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  void api.popupContextMenu('process', {
                    pid: row.pid,
                    displayName: row.displayName,
                    fullPath: row.fullPath,
                  });
                }}
              >
                <span className="mono vrow__pid">{row.pid}</span>
                <span className="vrow__name" title={row.fullPath}>
                  {row.displayName}
                </span>
                <span className="mono vrow__port">
                  {row.ports.length === 0
                    ? '—'
                    : row.ports.map((port) => `${port.address}:${port.port}`).join('  ')}
                </span>
              </div>
            )}
            empty={t('process.empty')}
          />
        </div>
      </Card>

      <Modal
        open={pendingKill !== null}
        title={t('process.kill.title')}
        onClose={() => setPendingKill(null)}
        footer={
          <>
            <button
              type="button"
              className="native-button"
              onClick={() => setPendingKill(null)}
            >
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="native-button native-button--accent"
              onClick={() => void confirmKill()}
            >
              {t('common.confirm')}
            </button>
          </>
        }
      >
        {pendingKill
          ? t('process.kill.confirm', [pendingKill.displayName, String(pendingKill.pid)])
          : ''}
      </Modal>
    </div>
  );
}
```

- [ ] **Step 2: 建组件统一出口 `frontend/src/components/index.ts`**

```typescript
export { Card } from './Card';
export { Checkbox } from './Checkbox';
export { DataTable, type Column } from './DataTable';
export { Kicker } from './Kicker';
export { Modal } from './Modal';
export { SearchField } from './SearchField';
export { Segmented } from './Segmented';
export { Sparkline } from './Sparkline';
export { VirtualList } from './VirtualList';
```

- [ ] **Step 3: 补页面样式**

在 `frontend/src/styles/base.css` 末尾追加：

```css
/* 页面骨架 */
.page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  min-height: 0;
}

.page__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.page__tools {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page__card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.page__card .card__body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0;
}

.mono {
  font-family: var(--font-mono);
  font-size: 11px;
}

.text-muted {
  color: var(--text-muted);
}

/* 虚拟列表的表格骨架：表头 + 行共用同一套栅格 */
.vtable {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}

.vrow {
  display: grid;
  grid-template-columns: 72px 1fr 140px;
  align-items: center;
  height: var(--row-height);
  padding: 0 10px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}

.vrow:hover {
  background: var(--surface-alt);
}

.vrow--head {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--surface);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  height: 24px;
}

.vrow__pid {
  text-align: right;
  padding-right: 10px;
}

.vrow__name,
.vrow__port {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
```

- [ ] **Step 4: 写测试并提交**

`frontend/src/pages/Processes.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// jsdom 不实现 ResizeObserver；进程页用它测量列表视口高度。
globalThis.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

vi.mock('../lib/bridge', () => ({
  EVENT: { processes: 'processes' },
  api: {
    listProcesses: vi.fn().mockResolvedValue([
      { pid: 12, displayName: 'Finder', fullPath: '/System/Finder', ports: [] },
      { pid: 340, displayName: 'node', fullPath: '/usr/local/bin/node', ports: [{ address: '*', port: 8080 }] },
    ]),
    refreshProcesses: vi.fn().mockResolvedValue([]),
    killProcess: vi.fn().mockResolvedValue(true),
    popupContextMenu: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'process.title': '进程管理',
      'process.pid': 'PID',
      'process.name': '进程名称',
      'process.port': '监听端口',
      'process.refresh': '刷新',
      'process.search.placeholder': '搜索进程名称、PID 或端口号',
      'process.count': '共 %@ 个进程',
      'process.empty': '没有匹配的进程',
      'process.kill.title': '确认结束进程',
      'process.kill.confirm': '确定要结束进程 %@ (PID: %@) 吗？',
      'process.kill.done': '已结束进程 %@',
      'common.cancel': '取消',
      'common.confirm': '确认',
    },
    activeTab: 'processes',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Processes } from './Processes';

describe('Processes', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染进程表头与行', async () => {
    render(<Processes />);
    expect(await screen.findByText('PID')).toBeInTheDocument();
    expect(await screen.findByText('Finder')).toBeInTheDocument();
    expect(await screen.findByText('*:8080')).toBeInTheDocument();
  });

  it('显示进程总数', async () => {
    render(<Processes />);
    expect(await screen.findByText('共 2 个进程')).toBeInTheDocument();
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 进程管理页

- 表格展示 PID / 进程名称 / 监听端口，支持按名称、PID、端口本地搜索
- 点击行弹出结束进程确认框，确认后调用 kill_process 并刷新
- 进程列表由后端 processes 事件推送，前端只持有渲染快照

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 20: 文件管理页

**Files:**
- Create: `frontend/src/pages/Files.tsx`
- Create: `frontend/src/pages/Files.test.tsx`

**Interfaces:**
- Consumes: `api.listDir` / `api.createFolder` / `api.createFile` / `api.renamePath` / `api.deleteToTrash` / `api.revealInFinder` / `api.parentDir` / `api.copyText` / `api.watchDir`
- Produces: `Files` 组件

- [ ] **Step 1: 写 `frontend/src/pages/Files.tsx`**

```tsx
import { useCallback, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Kicker, Modal, SearchField, type Column } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { FileRow } from '../lib/types';

export function Files() {
  const { t } = useI18n();
  const [cwd, setCwd] = useState<string>('');
  const [rows, setRows] = useState<FileRow[]>([]);
  const [search, setSearch] = useState('');
  const [renaming, setRenaming] = useState<FileRow | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [creating, setCreating] = useState<'folder' | 'file' | null>(null);
  const [createValue, setCreateValue] = useState('');

  const load = useCallback(async (path: string) => {
    const list = await api.listDir(path);
    setRows(list);
  }, []);

  // 初始目录：用户主目录。
  useEffect(() => {
    let disposed = false;
    void (async () => {
      const target = await api.homeDir();
      if (disposed || !target) {
        return;
      }
      setCwd(target);
      await load(target);
      await api.watchDir(target);
    })();
    return () => {
      disposed = true;
    };
  }, [load]);

  useEffect(() => {
    void onEvent(EVENT.filesChanged, () => {
      if (cwd) {
        void load(cwd);
      }
    });
  }, [cwd, load]);

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return rows;
    }
    return rows.filter((row) => row.name.toLowerCase().includes(keyword));
  }, [rows, search]);

  const columns = useMemo<Column<FileRow>[]>(
    () => [
      {
        key: 'name',
        title: t('file.column.name'),
        width: 'remainder',
        render: (row) => (
          <span className="file-name">
            <span className="file-name__glyph" aria-hidden="true">
              {row.isDir ? '◆' : '·'}
            </span>
            {row.name}
          </span>
        ),
      },
      {
        key: 'size',
        title: t('file.column.size'),
        width: 100,
        align: 'right',
        render: (row) => <span className="mono">{row.sizeText}</span>,
      },
      {
        key: 'modified',
        title: t('file.column.modified'),
        width: 130,
        render: (row) => <span className="mono">{formatTime(row.modified)}</span>,
      },
      {
        key: 'action',
        title: t('file.column.action'),
        width: 180,
        render: (row) => (
          <div className="row-actions">
            {row.isDir ? (
              <button
                type="button"
                className="native-button"
                onClick={(event) => {
                  event.stopPropagation();
                  void enter(row);
                }}
              >
                {t('file.openFolder')}
              </button>
            ) : (
              <button
                type="button"
                className="native-button"
                onClick={(event) => {
                  event.stopPropagation();
                  void api.revealInFinder(row.path);
                }}
              >
                {t('file.reveal')}
              </button>
            )}
            <button
              type="button"
              className="native-button"
              onClick={(event) => {
                event.stopPropagation();
                setRenaming(row);
                setRenameValue(row.name);
              }}
            >
              {t('file.rename')}
            </button>
            <button
              type="button"
              className="native-button"
              onClick={(event) => {
                event.stopPropagation();
                void remove(row);
              }}
            >
              {t('file.delete')}
            </button>
          </div>
        ),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [t, cwd],
  );

  const enter = useCallback(
    async (row: FileRow) => {
      setCwd(row.path);
      await load(row.path);
      await api.watchDir(row.path);
    },
    [load],
  );

  const goUp = useCallback(async () => {
    const parent = await api.parentDir(cwd);
    if (!parent) {
      return;
    }
    setCwd(parent);
    await load(parent);
    await api.watchDir(parent);
  }, [cwd, load]);

  const remove = useCallback(
    async (row: FileRow) => {
      try {
        await api.deleteToTrash(row.path);
        pushToast('info', t('file.deleted'));
        await load(cwd);
      } catch (error) {
        pushToast('error', String(error));
      }
    },
    [cwd, load, t],
  );

  const submitRename = useCallback(async () => {
    if (!renaming || !renameValue.trim()) {
      return;
    }
    try {
      await api.renamePath(renaming.path, renameValue.trim());
      pushToast('info', t('file.renamed', [renameValue.trim()]));
      setRenaming(null);
      await load(cwd);
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [cwd, load, renameValue, renaming, t]);

  const submitCreate = useCallback(async () => {
    if (!creating || !createValue.trim()) {
      return;
    }
    try {
      if (creating === 'folder') {
        await api.createFolder(cwd, createValue.trim());
      } else {
        await api.createFile(cwd, createValue.trim());
      }
      pushToast('info', t('file.created', [createValue.trim()]));
      setCreating(null);
      setCreateValue('');
      await load(cwd);
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [createValue, creating, cwd, load, t]);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="02">{t('file.title')}</Kicker>
        <div className="page__tools">
          <button
            type="button"
            className="native-button"
            onClick={async () => {
              const picked = await api.pickDirectory();
              if (!picked) {
                return;
              }
              setCwd(picked);
              await load(picked);
              await api.watchDir(picked);
            }}
          >
            {t('file.openFolder')}
          </button>
          <button type="button" className="native-button" onClick={() => void goUp()}>
            {t('common.back')}
          </button>
          <button
            type="button"
            className="native-button"
            onClick={() => {
              setCreating('file');
              setCreateValue('');
            }}
          >
            {t('file.newFile')}
          </button>
          <button
            type="button"
            className="native-button"
            onClick={() => {
              setCreating('folder');
              setCreateValue('');
            }}
          >
            {t('file.newFolder')}
          </button>
          <SearchField value={search} onChange={setSearch} placeholder={t('file.search.placeholder')} />
        </div>
      </div>

      <Card title={cwd} className="page__card">
        <DataTable
          columns={columns}
          items={filtered}
          getKey={(row) => row.path}
          onRowClick={(row) => {
            if (row.isDir) {
              void enter(row);
            }
          }}
          empty={t('common.empty')}
        />
      </Card>

      <Modal
        open={renaming !== null}
        title={t('file.rename.title')}
        onClose={() => setRenaming(null)}
        footer={
          <>
            <button type="button" className="native-button" onClick={() => setRenaming(null)}>
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="native-button native-button--accent"
              onClick={() => void submitRename()}
            >
              {t('common.save')}
            </button>
          </>
        }
      >
        <input
          className="native-input"
          value={renameValue}
          placeholder={t('file.rename.hint')}
          onChange={(event) => setRenameValue(event.target.value)}
        />
      </Modal>

      <Modal
        open={creating !== null}
        title={creating === 'folder' ? t('file.newFolder.title') : t('file.newFile')}
        onClose={() => setCreating(null)}
        footer={
          <>
            <button type="button" className="native-button" onClick={() => setCreating(null)}>
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="native-button native-button--accent"
              onClick={() => void submitCreate()}
            >
              {t('common.create')}
            </button>
          </>
        }
      >
        <input
          className="native-input"
          value={createValue}
          placeholder={t('file.newFolder.placeholder')}
          onChange={(event) => setCreateValue(event.target.value)}
        />
      </Modal>
    </div>
  );
}

function formatTime(seconds: number): string {
  if (!seconds) {
    return '—';
  }
  const date = new Date(seconds * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(
    date.getMinutes(),
  )}`;
}
```

- [ ] **Step 2: 补样式**

在 `frontend/src/styles/base.css` 末尾追加：

```css
.file-name {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.file-name__glyph {
  color: var(--accent);
  font-size: 9px;
}

.row-actions {
  display: flex;
  gap: 6px;
}
```

- [ ] **Step 3: 写测试并提交**

`frontend/src/pages/Files.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: { filesChanged: 'files-changed' },
  api: {
    listDir: vi.fn().mockResolvedValue([
      { path: '/tmp/a.txt', name: 'a.txt', isDir: false, size: 12, sizeText: '12 bytes', modified: 0 },
      { path: '/tmp/sub', name: 'sub', isDir: true, size: 0, sizeText: '-', modified: 0 },
    ]),
    watchDir: vi.fn().mockResolvedValue(undefined),
    homeDir: vi.fn().mockResolvedValue('/tmp'),
    pickDirectory: vi.fn().mockResolvedValue(null),
    parentDir: vi.fn().mockResolvedValue('/tmp'),
    revealInFinder: vi.fn().mockResolvedValue(undefined),
    renamePath: vi.fn().mockResolvedValue(undefined),
    deleteToTrash: vi.fn().mockResolvedValue(undefined),
    createFolder: vi.fn().mockResolvedValue(undefined),
    createFile: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'file.title': '文件管理',
      'file.column.name': '名称',
      'file.column.size': '大小',
      'file.column.modified': '修改时间',
      'file.column.action': '操作',
      'file.newFile': '新建文件',
      'file.newFolder': '新建文件夹',
      'file.openFolder': '进入文件夹',
      'file.rename': '重命名',
      'file.delete': '删除',
      'file.reveal': '在 Finder 中显示',
      'file.search.placeholder': '搜索文件',
      'common.back': '返回',
      'common.empty': '暂无数据',
    },
    activeTab: 'files',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Files } from './Files';

describe('Files', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染文件表头', async () => {
    render(<Files />);
    expect(await screen.findByText('名称')).toBeInTheDocument();
    expect(await screen.findByText('大小')).toBeInTheDocument();
  });

  it('显示新建文件夹与新建文件按钮', async () => {
    render(<Files />);
    expect(await screen.findByText('新建文件夹')).toBeInTheDocument();
    expect(await screen.findByText('新建文件')).toBeInTheDocument();
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 文件管理页

- 目录浏览、进入上级、新建文件/文件夹、重命名、移到废纸篓
- 切换目录时重建后端监听，files-changed 事件驱动刷新
- 操作结果统一走顶部提示

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 21: 剪贴板页

**Files:**
- Create: `frontend/src/pages/Clipboard.tsx`
- Create: `frontend/src/pages/Clipboard.test.tsx`

**Interfaces:**
- Consumes: `api.clipboardCurrent` / `api.clipboardHistory` / `api.removeHistoryItem` / `api.clearHistory` / `api.copyText`；`EVENT.clipboard`
- Produces: `Clipboard` 组件

- [ ] **Step 1: 写 `frontend/src/pages/Clipboard.tsx`**

```tsx
import { useCallback, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Kicker, SearchField, type Column } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { HistoryRow } from '../lib/types';

export function Clipboard() {
  const { t } = useI18n();
  const [current, setCurrent] = useState('');
  const [history, setHistory] = useState<HistoryRow[]>([]);
  const [search, setSearch] = useState('');

  useEffect(() => {
    let disposed = false;
    void api.clipboardCurrent().then((text) => {
      if (!disposed) {
        setCurrent(text);
      }
    });
    void api.clipboardHistory().then((list) => {
      if (!disposed) {
        setHistory(list);
      }
    });
    void onEvent<string>(EVENT.clipboard, (text) => setCurrent(text));
    return () => {
      disposed = true;
    };
  }, []);

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return history;
    }
    return history.filter((row) => row.content.toLowerCase().includes(keyword));
  }, [history, search]);

  const reload = useCallback(async () => {
    setHistory(await api.clipboardHistory());
  }, []);

  const columns = useMemo<Column<HistoryRow>[]>(
    () => [
      {
        key: 'content',
        title: t('clipboard.history'),
        width: 'remainder',
        render: (row) => <span title={row.content}>{row.content}</span>,
      },
      {
        key: 'kind',
        title: t('clipboard.current'),
        width: 90,
        render: (row) => <span className="text-muted">{row.kind}</span>,
      },
      {
        key: 'time',
        title: t('common.more'),
        width: 130,
        render: (row) => <span className="mono">{formatTime(row.timestamp)}</span>,
      },
      {
        key: 'action',
        title: t('file.column.action'),
        width: 140,
        render: (row) => (
          <div className="row-actions">
            <button
              type="button"
              className="native-button"
              onClick={async (event) => {
                event.stopPropagation();
                await api.copyText(row.content);
                pushToast('info', t('clipboard.copied'));
              }}
            >
              {t('clipboard.copy')}
            </button>
            <button
              type="button"
              className="native-button"
              onClick={async (event) => {
                event.stopPropagation();
                await api.removeHistoryItem(row.id);
                await reload();
              }}
            >
              {t('clipboard.delete')}
            </button>
          </div>
        ),
      },
    ],
    [reload, t],
  );

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="03">{t('clipboard.title')}</Kicker>
        <div className="page__tools">
          <SearchField
            value={search}
            onChange={setSearch}
            placeholder={t('clipboard.search.placeholder')}
          />
          <button
            type="button"
            className="native-button"
            onClick={async () => {
              await api.clearHistory();
              await reload();
            }}
          >
            {t('clipboard.clearHistory')}
          </button>
        </div>
      </div>

      <Card title={t('clipboard.current')} className="clipboard-current">
        <div className="clipboard-current__text">
          {current.length > 0 ? current : t('clipboard.empty')}
        </div>
      </Card>

      <Card title={t('clipboard.history.count', [String(filtered.length)])} className="page__card">
        <DataTable
          columns={columns}
          items={filtered}
          getKey={(row) => String(row.id)}
          empty={t('clipboard.empty')}
        />
      </Card>
    </div>
  );
}

function formatTime(seconds: number): string {
  if (!seconds) {
    return '—';
  }
  const date = new Date(seconds * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(
    date.getMinutes(),
  )}`;
}
```

- [ ] **Step 2: 补样式**

```css
.clipboard-current {
  flex: 0 0 auto;
}

.clipboard-current__text {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text);
  max-height: 84px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
```

- [ ] **Step 3: 写测试并提交**

`frontend/src/pages/Clipboard.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: { clipboard: 'clipboard' },
  api: {
    clipboardCurrent: vi.fn().mockResolvedValue('hello chuaker7'),
    clipboardHistory: vi.fn().mockResolvedValue([
      { id: 1, content: '第一条历史', timestamp: 1700000000, kind: 'text' },
    ]),
    removeHistoryItem: vi.fn().mockResolvedValue(undefined),
    clearHistory: vi.fn().mockResolvedValue(undefined),
    copyText: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'clipboard.title': '剪贴板管理',
      'clipboard.current': '当前剪贴板',
      'clipboard.history': '历史记录',
      'clipboard.history.count': '共 %@ 条历史',
      'clipboard.search.placeholder': '搜索历史记录',
      'clipboard.clearHistory': '清空历史',
      'clipboard.copy': '复制',
      'clipboard.delete': '删除',
      'clipboard.empty': '（空）',
      'clipboard.copied': '已复制到剪贴板',
      'file.column.action': '操作',
      'common.more': '更多',
    }},
    activeTab: 'clipboard',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Clipboard } from './Clipboard';

describe('Clipboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('展示当前剪贴板内容', async () => {
    render(<Clipboard />);
    expect(await screen.findByText('hello chuaker7')).toBeInTheDocument();
  });

  it('展示历史条目与计数', async () => {
    render(<Clipboard />);
    expect(await screen.findByText('第一条历史')).toBeInTheDocument();
    expect(await screen.findByText('共 1 条历史')).toBeInTheDocument();
  });

  it('渲染清空历史按钮', async () => {
    render(<Clipboard />);
    expect(await screen.findByText('清空历史')).toBeInTheDocument();
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 剪贴板页

- 顶部展示当前剪贴板内容，下方为可搜索的历史列表
- 支持复制单条、删除单条、清空历史
- clipboard 事件实时更新当前内容

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 22: 系统监控页

**Files:**
- Create: `frontend/src/pages/Monitor.tsx`
- Create: `frontend/src/pages/Monitor.test.tsx`

**Interfaces:**
- Consumes: `api.systemSnapshot`；`EVENT.systemInfo`
- Produces: `Monitor` 组件

- [ ] **Step 1: 写 `frontend/src/pages/Monitor.tsx`**

```tsx
import { useEffect, useState } from 'react';

import { Card, Kicker, Sparkline } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { formatPercent, formatRate } from '../lib/format';
import type { SystemInfoDto } from '../lib/types';

const EMPTY: SystemInfoDto = {
  cpuUsage: 0,
  memoryUsage: 0,
  diskUsage: 0,
  networkUpload: 0,
  networkDownload: 0,
  cpuHistory: [],
  memoryHistory: [],
};

export function Monitor() {
  const { t } = useI18n();
  const [info, setInfo] = useState<SystemInfoDto>(EMPTY);

  useEffect(() => {
    let disposed = false;
    void api.systemSnapshot().then((snapshot) => {
      if (!disposed) {
        setInfo(snapshot);
      }
    });
    void onEvent<SystemInfoDto>(EVENT.systemInfo, (snapshot) => setInfo(snapshot));
    return () => {
      disposed = true;
    };
  }, []);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="04">{t('monitor.title')}</Kicker>
      </div>

      <div className="metric-grid">
        <Metric label={t('monitor.cpu')} value={formatPercent(info.cpuUsage)} />
        <Metric label={t('monitor.memory')} value={formatPercent(info.memoryUsage)} />
        <Metric label={t('monitor.disk')} value={formatPercent(info.diskUsage)} />
        <Metric label={t('monitor.upload')} value={formatRate(info.networkUpload)} />
        <Metric label={t('monitor.download')} value={formatRate(info.networkDownload)} />
      </div>

      <div className="chart-grid">
        <Card title={t('monitor.cpu.history')}>
          <Sparkline values={info.cpuHistory} width={420} height={72} />
        </Card>
        <Card title={t('monitor.memory.history')}>
          <Sparkline values={info.memoryHistory} width={420} height={72} />
        </Card>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="card metric">
      <div className="metric__label">{label}</div>
      <div className="metric__value">{value}</div>
    </div>
  );
}
```

- [ ] **Step 2: 补样式**

```css
.metric-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 10px;
  flex: 0 0 auto;
}

.metric__label {
  font-family: var(--font-mono);
  font-size: 9px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.metric__value {
  font-family: var(--font-display);
  font-size: 26px;
  letter-spacing: 0.04em;
  margin-top: 4px;
}

.chart-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
```

- [ ] **Step 3: 写测试并提交**

`frontend/src/pages/Monitor.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: { systemInfo: 'system-info' },
  api: {
    systemSnapshot: vi.fn().mockResolvedValue({
      cpuUsage: 12.34,
      memoryUsage: 56.78,
      diskUsage: 90.12,
      networkUpload: 1.5,
      networkDownload: 2.25,
      cpuHistory: [1, 2, 3],
      memoryHistory: [4, 5, 6],
    }),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'monitor.title': '系统监控',
      'monitor.cpu': 'CPU使用率',
      'monitor.memory': '内存使用率',
      'monitor.disk': '磁盘使用率',
      'monitor.upload': '上传',
      'monitor.download': '下载',
      'monitor.cpu.history': 'CPU 采样',
      'monitor.memory.history': '内存采样',
    }},
    activeTab: 'monitor',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Monitor } from './Monitor';

describe('Monitor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染五个指标标签', async () => {
    render(<Monitor />);
    expect(await screen.findByText('CPU使用率')).toBeInTheDocument();
    expect(await screen.findByText('内存使用率')).toBeInTheDocument();
    expect(await screen.findByText('磁盘使用率')).toBeInTheDocument();
    expect(await screen.findByText('上传')).toBeInTheDocument();
    expect(await screen.findByText('下载')).toBeInTheDocument();
  });

  it('按固定小数位展示数值', async () => {
    render(<Monitor />);
    expect(await screen.findByText('12.3%')).toBeInTheDocument();
    expect(await screen.findByText('56.8%')).toBeInTheDocument();
    expect(await screen.findByText('1.50 MB/s')).toBeInTheDocument();
    expect(await screen.findByText('2.25 MB/s')).toBeInTheDocument();
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 系统监控页

- 五个指标卡展示 CPU / 内存 / 磁盘 / 上传 / 下载
- CPU 与内存各配一条 Canvas 手绘曲线
- 数据来自 system-info 事件推送

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 23: 插件页

**Files:**
- Create: `frontend/src/pages/Plugins.tsx`
- Create: `frontend/src/pages/Plugins.test.tsx`

**Interfaces:**
- Consumes: `api.listPlugins` / `api.reloadPlugins` / `api.pluginsDir`
- Produces: `Plugins` 组件

- [ ] **Step 1: 写 `frontend/src/pages/Plugins.tsx`**

```tsx
import { useCallback, useEffect, useState } from 'react';

import { Card, Kicker } from '../components';
import { api } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { PluginRow } from '../lib/types';

export function Plugins() {
  const { t } = useI18n();
  const [plugins, setPlugins] = useState<PluginRow[]>([]);
  const [dir, setDir] = useState('');

  useEffect(() => {
    void api.listPlugins().then(setPlugins);
    void api.pluginsDir().then(setDir);
  }, []);

  const reload = useCallback(async () => {
    const list = await api.reloadPlugins();
    setPlugins(list);
    pushToast('info', t('plugin.manifest'));
  }, [t]);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="05">{t('nav.plugins')}</Kicker>
        <div className="page__tools">
          <button type="button" className="native-button" onClick={() => void reload()}>
            {t('process.refresh')}
          </button>
        </div>
      </div>

      <Card title={t('settings.plugin.dir')} className="page__card">
        <div className="plugin-dir">{dir || '—'}</div>

        {plugins.length === 0 ? (
          <div className="empty-state">{t('settings.plugin.empty')}</div>
        ) : (
          plugins.map((plugin) => (
            <div key={plugin.id} className="plugin-item">
              <div className="plugin-item__name">
                <span className="file-name__glyph" aria-hidden="true">
                  ◆
                </span>
                {plugin.name}
              </div>
              <dl className="plugin-item__meta">
                <dt>{t('plugin.version')}</dt>
                <dd>{plugin.version}</dd>
                <dt>{t('plugin.author')}</dt>
                <dd>{plugin.author ?? '—'}</dd>
                <dt>{t('plugin.permissions')}</dt>
                <dd>{plugin.permissions.length > 0 ? plugin.permissions.join('、') : '—'}</dd>
                <dt>{t('plugin.minVersion')}</dt>
                <dd>{plugin.minimumAppVersion ?? '—'}</dd>
              </dl>
            </div>
          ))
        )}
      </Card>
    </div>
  );
}
```

- [ ] **Step 2: 补样式**

```css
.plugin-dir {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  padding-bottom: 8px;
}

.plugin-item {
  border-top: 1px solid var(--border);
  padding: 10px 0;
}

.plugin-item__name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  margin-bottom: 6px;
}

.plugin-item__meta {
  display: grid;
  grid-template-columns: 90px 1fr;
  gap: 2px 12px;
  margin: 0;
  font-size: 11px;
}

.plugin-item__meta dt {
  font-family: var(--font-mono);
  font-size: 9px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.plugin-item__meta dd {
  margin: 0;
  font-family: var(--font-mono);
}
```

- [ ] **Step 3: 写测试并提交**

`frontend/src/pages/Plugins.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: {},
  api: {
    listPlugins: vi.fn().mockResolvedValue([
      {
        id: 'demo',
        name: '示例插件',
        version: '1.0.0',
        author: 'Igigi',
        description: null,
        minimumAppVersion: null,
        permissions: ['fs'],
        main: null,
        dir: '/tmp/plugins',
      },
    ]),
    reloadPlugins: vi.fn().mockResolvedValue([]),
    pluginsDir: vi.fn().mockResolvedValue('/tmp/plugins'),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'nav.plugins': '插件',
      'settings.plugin.dir': '插件目录',
      'settings.plugin.empty': '尚未安装插件',
      'plugin.manifest': '清单信息',
      'plugin.version': '版本',
      'plugin.author': '作者',
      'plugin.permissions': '权限',
      'plugin.minVersion': '最低应用版本',
      'process.refresh': '刷新',
    }},
    activeTab: 'plugins',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Plugins } from './Plugins';

describe('Plugins', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('展示插件目录与清单信息', async () => {
    render(<Plugins />);
    expect(await screen.findByText('/tmp/plugins')).toBeInTheDocument();
    expect(await screen.findByText('示例插件')).toBeInTheDocument();
    expect(await screen.findByText('1.0.0')).toBeInTheDocument();
  });

  it('没有插件时展示空态', async () => {
    const { api } = await import('../lib/bridge');
    vi.mocked(api.listPlugins).mockResolvedValueOnce([]);
    render(<Plugins />);
    expect(await screen.findByText('尚未安装插件')).toBeInTheDocument();
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 插件页

- 展示插件目录、名称、版本、作者、权限与最低应用版本
- 提供重新扫描按钮与空态提示

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 24: 设置页

**Files:**
- Create: `frontend/src/pages/Settings.tsx`
- Create: `frontend/src/pages/Settings.test.tsx`

**Interfaces:**
- Consumes: `api.updateSettings` / `api.setAutoLaunch`；`store` 的 `settings`
- Produces: `Settings` 组件

- [ ] **Step 1: 写 `frontend/src/pages/Settings.tsx`**

```tsx
import { useCallback, useEffect } from 'react';

import { Card, Checkbox, Kicker, Segmented } from '../components';
import { api } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { applyTheme } from '../lib/theme';
import { pushToast, setState, useAppStore } from '../lib/store';
import type { Accent, Settings as SettingsDto, ThemeMode } from '../lib/types';

export function Settings() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const settings = state.settings;

  useEffect(() => {
    if (settings) {
      applyTheme(settings.themeMode, settings.accent);
    }
  }, [settings]);

  const patch = useCallback(async (delta: Partial<SettingsDto>) => {
    const next = await api.updateSettings(delta);
    applyTheme(next.themeMode, next.accent);
    setState({ settings: next });
  }, []);

  if (!settings) {
    return (
      <div className="page">
        <Kicker index="06">{t('settings.title')}</Kicker>
        <div className="empty-state">{t('common.loading')}</div>
      </div>
    );
  }

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="06">{t('settings.title')}</Kicker>
      </div>

      <div className="settings-grid">
        <Card title={t('settings.appearance')}>
          <div className="setting-row">
            <span className="setting-label">{t('settings.theme.mode')}</span>
            <Segmented<ThemeMode>
              value={settings.themeMode}
              onChange={(value) => void patch({ themeMode: value })}
              options={[
                { value: 'system', label: t('settings.theme.system') },
                { value: 'light', label: t('settings.theme.light') },
                { value: 'dark', label: t('settings.theme.dark') },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.accent')}</span>
            <Segmented<Accent>
              value={settings.accent}
              onChange={(value) => void patch({ accent: value })}
              options={[
                { value: 'signal', label: t('settings.accent.signal') },
                { value: 'haze', label: t('settings.accent.haze') },
                { value: 'moss', label: t('settings.accent.moss') },
                { value: 'amber', label: t('settings.accent.amber') },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.locale')}</span>
            <Segmented
              value={settings.locale}
              onChange={(value) => void patch({ locale: value })}
              options={[
                { value: 'zh-Hans', label: '简体中文' },
                { value: 'en', label: 'English' },
              ]}
            />
          </div>
        </Card>

        <Card title={t('settings.performance')}>
          <div className="setting-row">
            <span className="setting-label">{t('settings.refresh.interval')}</span>
            <Segmented
              value={String(settings.autoRefreshInterval)}
              onChange={(value) => void patch({ autoRefreshInterval: Number(value) })}
              options={[
                { value: '1', label: '1s' },
                { value: '5', label: '5s' },
                { value: '10', label: '10s' },
                { value: '30', label: '30s' },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.processLimit')}</span>
            <Segmented
              value={String(settings.processListLimit)}
              onChange={(value) => void patch({ processListLimit: Number(value) })}
              options={[
                { value: '100', label: '100' },
                { value: '500', label: '500' },
                { value: '0', label: t('settings.noLimit') },
              ]}
            />
          </div>

          <div className="setting-row">
            <span className="setting-label">{t('settings.networkMonitor')}</span>
            <Checkbox
              checked={settings.enableNetworkMonitor}
              onChange={(checked) => void patch({ enableNetworkMonitor: checked })}
            />
          </div>
          <p className="setting-hint">{t('settings.performance.hint')}</p>
        </Card>

        <Card title={t('settings.startup')}>
          <div className="setting-row">
            <span className="setting-label">{t('settings.enableMenuBar')}</span>
            <Checkbox
              checked={settings.showMenuBar}
              onChange={(checked) => void patch({ showMenuBar: checked })}
            />
          </div>
          <p className="setting-hint">{t('settings.menuBar.hint')}</p>

          <div className="setting-row">
            <span className="setting-label">{t('settings.autoLaunch')}</span>
            <Checkbox
              checked={settings.launchAtLogin}
              onChange={async (checked) => {
                try {
                  await api.setAutoLaunch(checked);
                  await patch({ launchAtLogin: checked });
                } catch (error) {
                  pushToast('error', String(error));
                }
              }}
            />
          </div>
          <p className="setting-hint">{t('settings.autoLaunch.hint')}</p>

          <div className="setting-row">
            <span className="setting-label">{t('settings.globalShortcut')}</span>
            <Checkbox
              checked={settings.globalShortcutEnabled}
              onChange={(checked) => void patch({ globalShortcutEnabled: checked })}
            />
          </div>
          <p className="setting-hint">{t('settings.globalShortcut.hint')}</p>
        </Card>

        <Card title={t('settings.about')}>
          <dl className="plugin-item__meta">
            <dt>{t('settings.version')}</dt>
            <dd>1.2.0</dd>
            <dt>{t('settings.bundleId')}</dt>
            <dd>com.igigi.chuaker7</dd>
            <dt>{t('settings.copyright')}</dt>
            <dd>MIT</dd>
          </dl>
        </Card>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 补样式**

```css
.settings-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  overflow: auto;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 5px 0;
}

.setting-label {
  font-size: 12px;
}

.setting-hint {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 2px;
}
```

- [ ] **Step 3: 写测试并提交**

`frontend/src/pages/Settings.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: {},
  api: {
    updateSettings: vi.fn().mockResolvedValue({
      showMenuBar: false,
      autoRefreshInterval: 5,
      processListLimit: 500,
      enableNetworkMonitor: true,
      themeMode: 'dark',
      accent: 'signal',
      launchAtLogin: false,
      globalShortcutEnabled: true,
      locale: 'zh-Hans',
    }),
    setAutoLaunch: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: {
      showMenuBar: false,
      autoRefreshInterval: 5,
      processListLimit: 500,
      enableNetworkMonitor: true,
      themeMode: 'dark',
      accent: 'signal',
      launchAtLogin: false,
      globalShortcutEnabled: true,
      locale: 'zh-Hans',
    },
    strings: {
      'settings.title': '设置',
      'settings.appearance': '外观',
      'settings.performance': '性能',
      'settings.startup': '启动项',
      'settings.about': '关于',
      'settings.theme.mode': '主题模式',
      'settings.theme.system': '跟随系统',
      'settings.theme.light': '浅色模式',
      'settings.theme.dark': '深色模式',
      'settings.accent': '强调色',
      'settings.accent.signal': '信号红',
      'settings.accent.haze': '雾蓝',
      'settings.accent.moss': '苔绿',
      'settings.accent.amber': '琥珀',
      'settings.locale': '界面语言',
      'settings.refresh.interval': '自动刷新间隔',
      'settings.processLimit': '进程列表上限',
      'settings.noLimit': '无限制',
      'settings.networkMonitor': '网络监控',
      'settings.performance.hint': '降低刷新频率和限制可减少 CPU 占用',
      'settings.enableMenuBar': '启用菜单栏模式',
      'settings.menuBar.hint': '启用后，应用将在菜单栏显示图标',
      'settings.autoLaunch': '开机自启动',
      'settings.autoLaunch.hint': '启用后，应用将在登录时自动启动',
      'settings.globalShortcut': '全局快捷键',
      'settings.globalShortcut.hint': '启用后按 ⌘+L 可快速呼出应用',
      'settings.version': '版本',
      'settings.bundleId': 'Bundle ID',
      'settings.copyright': '版权',
    },
    activeTab: 'settings',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Settings } from './Settings';

describe('Settings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染四张设置卡片', async () => {
    render(<Settings />);
    expect(await screen.findByText('外观')).toBeInTheDocument();
    expect(await screen.findByText('性能')).toBeInTheDocument();
    expect(await screen.findByText('启动项')).toBeInTheDocument();
    expect(await screen.findByText('关于')).toBeInTheDocument();
  });

  it('渲染主题分段控件的三个选项', async () => {
    render(<Settings />);
    expect(await screen.findByText('跟随系统')).toBeInTheDocument();
    expect(await screen.findByText('浅色模式')).toBeInTheDocument();
    expect(await screen.findByText('深色模式')).toBeInTheDocument();
  });

  it('渲染关于信息中的 bundle id', async () => {
    render(<Settings />);
    expect(await screen.findByText('com.igigi.chuaker7')).toBeInTheDocument();
  });
});
```

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run test && npm run typecheck
git add -A
git commit -m "feat: 设置页

- 外观：主题模式、强调色、界面语言
- 性能：刷新间隔、进程上限、网络监控开关
- 启动项：菜单栏模式、开机自启、全局快捷键；附关于信息

AI-Coding: true
AI-Tool: TophantAI"
```

---

## P4 打磨与交付

### Task 25: 原生右键菜单

**Files:**
- Modify: `src-tauri/src/commands/system.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/system/menu.rs`
- Modify: `frontend/src/pages/Processes.tsx`
- Modify: `frontend/src/pages/Files.tsx`
- Modify: `frontend/src/pages/Clipboard.tsx`

**Interfaces:**
- Consumes: `commands::dto::ContextTarget`、`tauri::menu::*`、`Window::popup_menu`
- Produces: command `popup_context_menu` 的完整实现；菜单 id `ctx.process.copyPath` / `ctx.process.kill` / `ctx.file.reveal` / `ctx.file.copyPath` / `ctx.file.rename` / `ctx.file.delete` / `ctx.clipboard.copy` / `ctx.clipboard.delete`

- [ ] **Step 1: 给 `AppState` 加待处理右键目标**

`src-tauri/src/state.rs`：

```rust
use crate::commands::dto::ContextTarget;

/// 应用共享状态。
pub struct AppState {
    // ...既有字段...
    /// 当前弹出的原生右键菜单所对应的目标。
    pub pending_context: Mutex<Option<ContextTarget>>,
}
```

`AppState::new` 中追加：

```rust
            pending_context: Mutex::new(None),
```

- [ ] **Step 2: 实现 `popup_context_menu`**

`src-tauri/src/commands/system.rs`：

```rust
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::Manager;

use crate::commands::dto::ContextTarget;

/// 弹出原生右键菜单。
///
/// 菜单项点击由 `system::menu::on_menu_event` 统一处理，
/// 目标对象暂存在 `AppState::pending_context`。
#[tauri::command]
pub fn popup_context_menu(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    kind: String,
    payload: serde_json::Value,
) {
    let raw = serde_json::json!({ "kind": kind, "payload": payload });
    let Ok(target) = serde_json::from_value::<ContextTarget>(raw) else {
        return;
    };

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let menu = match build_context_menu(&app, &target) {
        Ok(menu) => menu,
        Err(err) => {
            eprintln!("[menu] 构建右键菜单失败: {err}");
            return;
        }
    };

    {
        let guard = state.lock().expect("状态锁中毒");
        *guard.pending_context.lock().expect("右键目标锁中毒") = Some(target);
    }

    let _ = window.popup_menu(&menu);
}

/// 构建与目标匹配的原生菜单。
///
/// `Menu::with_items` 要求 `&[&dyn IsMenuItem<R>]` 切片，同类型项才能放进同一个数组；
/// 这里改用 `Menu::new` + `append`，可以混合 `MenuItem` 与 `PredefinedMenuItem`。
fn build_context_menu(app: &AppHandle, target: &ContextTarget) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;

    match target {
        ContextTarget::Process { .. } => {
            menu.append(&MenuItem::with_id(
                app,
                "ctx.process.copyPath",
                "复制完整路径",
                true,
                None::<&str>,
            )?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.process.kill",
                "结束进程",
                true,
                None::<&str>,
            )?)?;
        }
        ContextTarget::File { .. } => {
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.reveal",
                "在 Finder 中显示",
                true,
                None::<&str>,
            )?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.copyPath",
                "复制路径",
                true,
                None::<&str>,
            )?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.rename",
                "重命名",
                true,
                None::<&str>,
            )?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.file.delete",
                "移到废纸篓",
                true,
                None::<&str>,
            )?)?;
        }
        ContextTarget::Clipboard { .. } => {
            menu.append(&MenuItem::with_id(
                app,
                "ctx.clipboard.copy",
                "复制",
                true,
                None::<&str>,
            )?)?;
            menu.append(&PredefinedMenuItem::separator(app)?)?;
            menu.append(&MenuItem::with_id(
                app,
                "ctx.clipboard.delete",
                "删除",
                true,
                None::<&str>,
            )?)?;
        }
    }

    Ok(menu)
}
```

- [ ] **Step 3: 处理菜单项点击**

`src-tauri/src/system/menu.rs` 追加：

```rust
use crate::core::platform;
use crate::events::EV_NOTICE;
use tauri::Manager;

/// 处理右键菜单项点击。
pub fn on_context_menu_event(app: &AppHandle, id: &str) {
    use crate::core::process::ProcessManager;

    let guard = app.state::<std::sync::Arc<std::sync::Mutex<crate::state::AppState>>>();
    let target = {
        let state = guard.lock().expect("状态锁中毒");
        let mut pending = state.pending_context.lock().expect("右键目标锁中毒");
        pending.take()
    };

    let Some(target) = target else {
        return;
    };

    match (id, &target) {
        ("ctx.process.copyPath", ContextTarget::Process { full_path, .. }) => {
            platform::copy_to_clipboard(full_path);
            notify(app, "info", "已复制完整路径");
        }
        ("ctx.process.kill", ContextTarget::Process { pid, display_name, .. }) => {
            if ProcessManager::kill_process(*pid) {
                notify(app, "info", &format!("已结束进程 {display_name}"));
                // 立刻重采一次并把新列表推给前端，不要等下一个刷新周期。
                let rows = {
                    let state = guard.lock().expect("状态锁中毒");
                    let limit = state
                        .settings
                        .lock()
                        .expect("设置锁中毒")
                        .process_list_limit;
                    let mut processes = state.processes.lock().expect("进程锁中毒");
                    processes
                        .collect(limit)
                        .iter()
                        .map(crate::commands::dto::ProcessRow::from)
                        .collect::<Vec<_>>()
                };
                let _ = app.emit(crate::events::EV_PROCESSES, rows);
            } else {
                notify(app, "error", "结束进程失败，可能需要管理员权限");
            }
        }
        ("ctx.file.reveal", ContextTarget::File { path, .. }) => {
            platform::reveal_in_finder(std::path::Path::new(path));
        }
        ("ctx.file.copyPath", ContextTarget::File { path, .. }) => {
            platform::copy_to_clipboard(path);
            notify(app, "info", "已复制路径");
        }
        ("ctx.file.rename", ContextTarget::File { .. }) => {
            let _ = app.emit(EV_MENU_ACTION, "file:rename");
        }
        ("ctx.file.delete", ContextTarget::File { .. }) => {
            let _ = app.emit(EV_MENU_ACTION, "file:delete");
        }
        ("ctx.clipboard.copy", ContextTarget::Clipboard { content, .. }) => {
            platform::copy_to_clipboard(content);
            notify(app, "info", "已复制到剪贴板");
        }
        ("ctx.clipboard.delete", ContextTarget::Clipboard { id, .. }) => {
            {
                let state = guard.lock().expect("状态锁中毒");
                let mut manager = state.clipboard.lock().expect("剪贴板锁中毒");
                manager.delete_history_item(*id);
            }
            notify(app, "info", "已删除");
        }
        _ => {}
    }
}

/// 向前端回传提示。
fn notify(app: &AppHandle, kind: &str, message: &str) {
    let _ = app.emit(
        EV_NOTICE,
        crate::commands::dto::Notice {
            kind: kind.to_string(),
            message: message.to_string(),
        },
    );
}
```

`src-tauri/src/lib.rs` 的 `on_menu_event` 改为：

```rust
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            crate::system::menu::on_menu_event(app, id);
            if id.starts_with("ctx.") {
                crate::system::menu::on_context_menu_event(app, id);
            }
        })
```

- [ ] **Step 4: 前端接入右键**

`frontend/src/pages/Processes.tsx` 的 `DataTable` 追加：

```tsx
          onRowContextMenu={(row) => {
            void api.popupContextMenu('process', {
              pid: row.pid,
              displayName: row.displayName,
              fullPath: row.fullPath,
            });
          }}
```

`frontend/src/pages/Files.tsx`：

```tsx
          onRowContextMenu={(row) => {
            setContextRow(row);
            void api.popupContextMenu('file', {
              path: row.path,
              name: row.name,
              isDir: row.isDir,
            });
          }}
```

并在 `Files` 中新增 `const [contextRow, setContextRow] = useState<FileRow | null>(null);`，
再新增一个 `useEffect` 监听 `EVENT.menuAction`：

```tsx
  useEffect(() => {
    void onEvent<string>(EVENT.menuAction, (action) => {
      if (action === 'file:rename' && contextRow) {
        setRenaming(contextRow);
        setRenameValue(contextRow.name);
      }
      if (action === 'file:delete' && contextRow) {
        void remove(contextRow);
      }
    });
  }, [contextRow]);
```

`frontend/src/pages/Clipboard.tsx`：

```tsx
          onRowContextMenu={(row) => {
            void api.popupContextMenu('clipboard', { id: row.id, content: row.content });
          }}
```

- [ ] **Step 5: 验收并提交**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt && cargo build -p chuaker7 && cargo clippy -p chuaker7 --all-targets
source ~/.nvm/nvm.sh && nvm use && npm run test && npm run typecheck
npm run tauri dev
```

Expected：进程行、文件行、剪贴板行右键均弹出 macOS 原生菜单；「复制完整路径」后系统剪贴板内容正确；「重命名」与「移到废纸篓」由前端接管并弹窗确认。

```bash
git add -A
git commit -m "feat: 原生右键菜单

- 进程 / 文件 / 剪贴板行各自弹出 macOS 原生菜单
- 复制路径、结束进程等直接由 Rust 执行并回传 notice 提示
- 重命名与删除需要前端确认，Rust 发 menu-action 由前端接管

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 26: 性能调优与质量门禁

**Files:**
- Modify: `frontend/src/components/DataTable.tsx`
- Modify: `frontend/src/pages/Processes.tsx`
- Modify: `frontend/src/pages/Files.tsx`
- Modify: `frontend/src/pages/Clipboard.tsx`
- Create: `docs/PERFORMANCE.md`

**Interfaces:**
- Consumes: Task 17–24 的页面实现
- Produces: 稳定的渲染性能与一份可复核的性能记录

- [ ] **Step 1: 行组件 memo 化**

`frontend/src/components/DataTable.tsx` 内新增：

```tsx
const TableRow = memo(function TableRow<T>({
  columns,
  item,
  index,
  selected,
  onRowClick,
  onRowContextMenu,
}: {
  columns: Column<T>[];
  item: T;
  index: number;
  selected: boolean;
  onRowClick?: (item: T, index: number) => void;
  onRowContextMenu?: (item: T, index: number) => void;
}) {
  return (
    <tr
      data-selected={selected ? 'true' : undefined}
      onClick={onRowClick ? () => onRowClick(item, index) : undefined}
      onContextMenu={
        onRowContextMenu
          ? (event) => {
              event.preventDefault();
              onRowContextMenu(item, index);
            }
          : undefined
      }
    >
      {columns.map((column) => (
        <td key={column.key} style={{ textAlign: column.align ?? 'left' }}>
          {column.render(item, index)}
        </td>
      ))}
    </tr>
  );
});
```

并在文件顶部加 `import { memo } from 'react';`，把 `<tbody>` 里的 `<tr>` 换成 `<TableRow ... />`。

- [ ] **Step 2: 搜索输入走 useDeferredValue**

`frontend/src/pages/Processes.tsx`：

```tsx
import { useDeferredValue } from 'react';
```

```tsx
  const deferredSearch = useDeferredValue(search);
```

并把 `filtered` 的依赖从 `search` 换成 `deferredSearch`：

```tsx
  const filtered = useMemo(() => {
    const keyword = deferredSearch.trim().toLowerCase();
    // ...
  }, [rows, deferredSearch]);
```

`Files.tsx` 与 `Clipboard.tsx` 同样处理。

- [ ] **Step 3: 进程页接入虚拟列表（已由 Task 19 落地，本任务只做行组件 memo 化）**

进程页在 Task 19 已经用 `VirtualList` 渲染，虚拟滚动与行高 26px / overscan 8
在 Task 19 就已就位（满足「500 行 DOM 节点 ≤ 60」的性能红线）。

本任务只把行渲染抽成 memo 组件，避免无关行在 `rows` 变化时重复渲染：

`frontend/src/pages/Processes.tsx` 顶部加 `import { memo } from 'react';`，
并新增：

```tsx
const ProcessRowView = memo(function ProcessRowView({
  row,
  onSelect,
}: {
  row: ProcessRow;
  onSelect: (row: ProcessRow) => void;
}) {
  return (
    <div
      className="vrow"
      onClick={() => onSelect(row)}
      onContextMenu={(event) => {
        event.preventDefault();
        void api.popupContextMenu('process', {
          pid: row.pid,
          displayName: row.displayName,
          fullPath: row.fullPath,
        });
      }}
    >
      <span className="mono vrow__pid">{row.pid}</span>
      <span className="vrow__name" title={row.fullPath}>
        {row.displayName}
      </span>
      <span className="mono vrow__port">
        {row.ports.length === 0
          ? '—'
          : row.ports.map((port) => `${port.address}:${port.port}`).join('  ')}
      </span>
    </div>
  );
});
```

然后把 `VirtualList` 的 `renderRow` 换成：

```tsx
            renderRow={(row) => (
              <ProcessRowView row={row} onSelect={setPendingKill} />
            )}
```

> `onSelect` 传的是 `setPendingKill`（稳定引用），`row` 是数据引用——
> `rows` 数组不变时行引用稳定，memo 才能真正生效。

- [ ] **Step 4: 记录性能基线**

创建 `docs/PERFORMANCE.md`：

```markdown
# chuaker7 性能记录

## 测量环境

- macOS 26.3.1（darwin 25.3.0 arm64）
- Node v24.17.0 / Vite 7.1.5
- rustc 1.90.0
- 窗口尺寸 1080×720

## 基线指标

| 指标 | 目标（验收线） | 实测（Step 4 填写） |
| --- | --- | --- |
| 进程列表 500 行时的 DOM 节点数 | ≤ 60 | |
| 首帧可见时间 | ≤ 300ms | |
| 滚动帧率 | ≥ 55fps | |
| 进程事件负载大小 | ≤ 80KB | |

> Step 4 必须把四个「实测」格填上真实数字后才能提交本任务；任何一项不达标时，
> 在「结论」里写明差距与后续处理，而不是留空。

## 测量方法

1. `npm run tauri dev` 启动应用。
2. 进程页：`document.querySelectorAll('*').length` 记录 DOM 节点总数。
3. 滚动帧率：在 Web Inspector 的 Performance 面板录制滚动过程。
4. 事件负载：在 DevTools Network 面板查看 `processes` 事件消息体大小。

## 结论

待实测后填写。
```

- [ ] **Step 5: 跑全量质量门禁**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cargo fmt --check
cargo clippy --all-targets
cargo test
source ~/.nvm/nvm.sh && nvm use
npm run typecheck
npm run test
npm run build
```

Expected：Rust 与前端全部通过，0 warning。

- [ ] **Step 6: 提交**

```bash
git add -A
git commit -m "perf: 渲染性能调优与质量门禁

- 表格行组件 memo 化，搜索输入走 useDeferredValue
- 进程列表改用虚拟列表，500 行只渲染约 28 个节点
- 新增 docs/PERFORMANCE.md 记录基线与测量方法

AI-Coding: true
AI-Tool: TophantAI"
```

---

### Task 27: 视觉验收、README 与打包发布

**Files:**
- Create: `docs/screenshots/tauri-processes.png`
- Create: `docs/screenshots/tauri-files.png`
- Create: `docs/screenshots/tauri-clipboard.png`
- Create: `docs/screenshots/tauri-monitor.png`
- Create: `docs/screenshots/tauri-plugins.png`
- Create: `docs/screenshots/tauri-settings.png`
- Modify: `README.md`
- Modify: `.gitignore`
- Create: `release/chuaker7.zip`

**Interfaces:**
- Consumes: 全部前置任务
- Produces: 六个页面的新截图、更新的 README、可分发 `.app` 包

- [ ] **Step 1: 启动前端并截图**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run dev &
sleep 4
```

逐页截图（用 Chrome headless）：

```bash
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
for page in processes files clipboard monitor plugins settings; do
  "$CHROME" --headless --disable-gpu --hide-scrollbars \
    --screenshot="docs/screenshots/tauri-${page}.png" \
    --window-size=1080,720 \
    "http://localhost:1420/?page=${page}"
done
```

> 前端需支持 `?page=` 参数直接定位标签页。在 `frontend/src/app/AppShell.tsx` 的启动引导里追加：

```tsx
      const requested = new URLSearchParams(window.location.search).get('page');
      if (requested && (requested in PAGES)) {
        setState({ activeTab: requested as never });
      }
```

重新截完图后：

```bash
kill %1
```

- [ ] **Step 2: 与 egui 版对照**

把新截图与 `docs/screenshots/{processes,files,monitor,settings}.png` 并排比对，确认：

- ink 底 `#0F0E15` 与奶油色文字 `#F3EDE2` 一致；
- 强调色为信号红 `#F43D3F`；
- 发丝描边、小圆角、`// 01` 编号、`◆` 分隔符的排版语言保留；
- 行高、字号、留白与 egui 版接近。

发现偏差时只调整 CSS 变量与尺寸，**不改视觉语言本身**。

- [ ] **Step 3: 更新 README**

README 需要包含：

- 项目简介：chuaker7 是 LuciBox 的 Rust + Tauri v2 重写版；
- 技术栈表格（Tauri v2 / React 19 / TypeScript 5.9 / Rust 1.90）；
- 功能清单（进程管理、文件管理、剪贴板、系统监控、插件、设置）与原生能力清单（应用菜单栏、原生右键菜单、托盘菜单、⌘L 全局快捷键、菜单栏模式、开机自启）；
- 开发指南（`nvm use` → `npm install` → `npm run tauri dev`）；
- 构建指南（`npm run tauri build` → `src-tauri/target/release/bundle/`）；
- 六张新截图（替换 egui 版截图）；
- 许可证说明（MIT，字体为 SIL OFL）。

- [ ] **Step 4: 更新 `.gitignore`**

追加：

```gitignore
# Tauri 构建产物
src-tauri/target/
src-tauri/gen/
```

> 根 `/target/` 已在忽略列表中；`src-tauri/target/` 需要单独忽略。

- [ ] **Step 5: 构建可分发包**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
source ~/.nvm/nvm.sh && nvm use
npm run tauri build
```

Expected：
- `src-tauri/target/release/bundle/macos/chuaker7.app` 生成；
- `src-tauri/target/release/bundle/dmg/chuaker7_1.2.0_*.dmg` 生成。

验证应用可运行：

```bash
open src-tauri/target/release/bundle/macos/chuaker7.app
sleep 3
osascript -e 'tell application "System Events" to get name of every process whose background only is false'
```

Expected：进程列表包含 `chuaker7`。

打包 zip：

```bash
cd src-tauri/target/release/bundle/macos
zip -r /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react/release/chuaker7.zip chuaker7.app
```

- [ ] **Step 6: 提交并推送**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git add -A
git commit -m "docs: 更新 README 与界面预览，产出可分发 .app 包

- README 补充 Tauri 架构说明、原生能力清单与开发构建指南
- 六张页面截图替换 egui 版预览
- tauri build 产出 chuaker7.app 与 dmg

AI-Coding: true
AI-Tool: TophantAI"
```

推送当前分支（用 `GIT_ASKPASS` 注入凭证，脚本用完即删）：

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
cat > /tmp/k7-askpass.sh <<'EOF'
#!/bin/sh
case "$1" in
  *Username*) echo "alishiguang" ;;
  *Password*) echo "$CHUAKER7_TOKEN" ;;
esac
EOF
chmod +x /tmp/k7-askpass.sh
CHUAKER7_TOKEN="<从安全位置读取的 PAT>" GIT_ASKPASS=/tmp/k7-askpass.sh GIT_TERMINAL_PROMPT=0 git push -u origin feat/tauri-react-rewrite
"/Users/lishiguang/.minimax/bin/mavis-trash" -- "/tmp/k7-askpass.sh"
```

> PAT 不得写进脚本、提交或日志。执行前从密码管理器 / 密钥链读取并只放进环境变量；脚本用完立刻用 `mavis-trash` 删除。
> 合并回 `main` 由 `superpowers:finishing-a-development-branch` 收尾时处理（合并 → 跑测试 → 推送 main）。

- [ ] **Step 7: 收尾核对**

```bash
cd /Users/lishiguang/aiworkspace/Ki6/.worktrees/tauri-react
git status --short          # 应为空
git log --oneline -5        # 确认新提交
git branch -a               # legacy-egui 仍在
ls -la release/             # chuaker7.zip 存在
```

---

## 计划自检记录

### 1. 设计文档覆盖对照

| 设计文档章节 | 覆盖任务 |
| --- | --- |
| §3 技术选型 | Task 1（Rust 侧）、Task 2（前端）、Task 16（状态）、Task 17（虚拟列表） |
| §4 目录结构 | Task 1 / 2 / 4 / 5 / 6（偏离两处已在「目录结构总览」说明） |
| §5 进程模型与数据流 | Task 7（后台循环 + 事件）、Task 8–11（命令） |
| §6.1 Commands | Task 8（进程）、Task 9（文件）、Task 10（剪贴板/监控/插件）、Task 11（设置/文案/系统） |
| §6.2 Events | Task 7（事件常量）、Task 12（menu-action）、Task 25（notice） |
| §6.3 数据结构 | Task 6（DTO）、Task 16（TS 镜像） |
| §7 原生感落地清单 | Task 12（应用菜单栏）、Task 13（托盘）、Task 14（⌘L + 激活策略）、Task 15（自启）、Task 17（控件 macOS 化）、Task 20（原生目录对话框）、Task 25（原生右键菜单）、Task 18（首帧直出）、Task 19/26（虚拟列表）、Task 22（Canvas 曲线） |
| §8.1 状态 | Task 16（store.ts） |
| §8.2 样式 | Task 3（tokens/base/native）、Task 18（主题属性） |
| §8.3 性能 | Task 17（虚拟列表）、Task 26（memo / useDeferredValue） |
| §8.4 测试 | Task 4–6（Rust 单测）、Task 16–24（vitest）、Task 26（`tsc --noEmit`） |
| §9 视觉验收 | Task 27（Chrome headless 截图 + 与 egui 版对照） |
| §10 实施阶段 P0–P4 | Task 1–3（P0）、Task 4–6（P1）、Task 7–15（P2）、Task 16–24（P3）、Task 25–27（P4） |
| §11 仓库策略 | Task 4 Step 1（legacy-egui 存档）、全局提交规范 |
| §12 风险对策 | Task 14（激活策略降级）、Task 27（Chrome headless 替代 egui 截图）、Task 7/26（事件负载控制） |

### 2. 与设计文档的契约增补（需在实现时同步回设计文档）

| 增补项 | 原因 | 落点 |
| --- | --- | --- |
| event `notice` | 原生右键菜单由 Rust 弹出，动作结果需要回传提示文案 | Task 25 |
| event `menu-action` | 原生应用菜单栏的「视图 → 某页」需要驱动前端切换标签页 | Task 12 |
| command `watch_dir` | 切换目录时需要重建后端 `notify` 监听器 | Task 9 |
| command `plugins_dir` | 前端需要展示插件目录路径 | Task 10 |
| `Settings` 字段加 `alias` | 兼容 egui 版写下的 snake_case 旧配置，避免用户设置丢失 | Task 4 |
| `HistoryEntry` 增加 `id` | 删除历史需要稳定标识，索引在插入/截断后会漂移 | Task 5 |
| `ContextTarget` 枚举 | 右键菜单目标需要带类型判别 | Task 6 |

### 3. 类型一致性核对

- Rust `ProcessRow` ↔ TS `ProcessRow`：字段 `pid` / `displayName` / `fullPath` / `ports` 完全一致。
- Rust `FileRow` ↔ TS `FileRow`：字段 `path` / `name` / `isDir` / `size` / `sizeText` / `modified` 一致。
- Rust `HistoryRow` ↔ TS `HistoryRow`：字段 `id` / `content` / `timestamp` / `kind` 一致。
- Rust `SystemInfoDto` ↔ TS `SystemInfoDto`：`cpuUsage` / `memoryUsage` / `diskUsage` / `networkUpload` / `networkDownload` / `cpuHistory` / `memoryHistory` 一致。
- Rust `PluginRow` ↔ TS `PluginRow`：`id` / `name` / `version` / `author` / `description` / `minimumAppVersion` / `permissions` / `main` / `dir` 一致。
- Rust `Settings` ↔ TS `Settings`：九个字段 camelCase 一一对应；`ThemeMode` 取值 `system` / `light` / `dark`，`Accent` 取值 `signal` / `haze` / `moss` / `amber`，`Locale` 取值 `zh-Hans` / `en`。
- 事件名：Rust `events.rs` 常量值与 TS `bridge.ts` 的 `EVENT` 常量逐条相同。
- command 名：Rust `#[tauri::command]` 函数名与 TS `bridge.ts` 的 `api.*` 中 `invoke` 的第一个参数逐条相同。

### 4. 执行顺序依赖

```
Task 1 → Task 2 → Task 3
Task 1 → Task 4 → Task 5 → Task 6
Task 5 + 6 → Task 7 → Task 8/9/10/11 → Task 12/13/14/15
Task 16 → Task 17 → Task 18 → Task 19/20/21/22/23/24
Task 8/9/10 + Task 19/20/21 → Task 25
Task 24 → Task 26 → Task 27
```

Task 4–15 可与 Task 16–24 并行（Rust 侧与前端侧文件不重叠），但 Task 25 起需要两侧合并。
