# chuaker7 架构改版设计：Tauri v2 + React + TypeScript

- 日期：2026-11-24
- 状态：已批准
- 取代：`b36b8dc`（egui 0.33 立即模式版，存档于 `legacy-egui` 分支）

## 1. 背景与动机

egui 版功能对等移植完成，但实际使用感受不佳，用户明确指出三个问题：

1. **不像原生 macOS 应用** —— 输入框、复选框、滚动条全部自绘，与系统控件手感不一致。
2. **有卡顿或闪烁** —— 页面首帧空白、滚动掉帧、点击反馈迟钝。
3. **缺少原生交互** —— 没有原生菜单栏、原生右键菜单，键盘操作能力弱。

同时用户决定：**视觉风格继续沿用 cindy.cn 的 ink/paper 暗色系**（不改设计语言），**前端构建链可以放宽**（允许 Tauri 这类 Web 前端方案）。

因此本次改版的目标是：**换一个真正的前端架构，让交互层回到 macOS 原生能力上，视觉层保持 cindy.cn 不变。**

## 2. 目标与非目标

### 目标

- 用 Tauri v2（WKWebView）+ React + TypeScript 重建整个前端层。
- Rust 侧继续承担全部业务逻辑与系统集成，作为唯一权威状态源。
- 交付原生应用菜单栏、原生右键菜单、原生托盘菜单、原生文件对话框。
- 消除首帧空白与滚动掉帧。
- 功能范围与 Swift 原版 LuciBox 严格对等，不增不减。

### 非目标

- 不做跨平台（只服务 macOS）。
- 不做前端 E2E 测试框架。
- 不重构主题系统与插件系统。
- 不引入第三方状态管理库、虚拟列表库、UI 组件库。

## 3. 技术选型

| 层 | 选型 | 理由 |
| --- | --- | --- |
| 应用壳 | Tauri v2.12（Rust） | WKWebView = Safari 同引擎，滚动条/文本框/输入法全部系统级 |
| 渲染 | React 19 + TypeScript | 6 个页面 + 共享状态，模型最契合；生态与长期维护成本最低 |
| 构建 | Vite | 冷启动快，产物小 |
| 状态 | React 内置（`useSyncExternalStore`） | 够用，不引依赖 |
| 虚拟列表 | 自研（~120 行，固定行高） | 需求简单，避免依赖 |
| 原生能力 | Tauri `menu` / `tray` / `dialog` + `objc2-app-kit` | `menu.popup()` 提供原生右键菜单；`objc2` 只用于切换 `NSApp.activationPolicy` |

已核实的 Tauri v2 能力（对照官方文档）：

- `Menu.setAsAppMenu()` —— macOS 全局菜单栏
- `Menu.popup(at?)` —— **原生右键菜单**，替换 WebView 默认菜单
- `PredefinedMenuItem` —— 原生 Copy / Paste / SelectAll / Quit / About
- `MenuItem.accelerator` —— `CmdOrCtrl+L` 等快捷键
- `TrayIconBuilder` —— 托盘图标 + 托盘菜单

## 4. 目录结构

```
chuaker7/
├── src-tauri/                 # Rust 侧（Tauri v2 应用）
│   ├── Cargo.toml
│   ├── tauri.conf.json        # 窗口、托盘、权限、bundle 配置
│   ├── build.rs
│   ├── icons/                 # 应用图标 + 托盘模板图
│   └── src/
│       ├── main.rs            # Tauri Builder 装配、插件注册
│       ├── lib.rs
│       ├── state.rs           # AppState：全局共享状态
│       ├── commands/          # Tauri command 门面（薄，仅编解码）
│       │   ├── mod.rs
│       │   ├── processes.rs
│       │   ├── files.rs
│       │   ├── clipboard.rs
│       │   ├── monitor.rs
│       │   ├── plugins.rs
│       │   └── settings.rs
│       ├── events.rs          # 事件名与负载定义
│       ├── core/              # 与 UI 无关的业务逻辑（原样迁移）
│       │   ├── process.rs     # ps + lsof 取数，端口解析
│       │   ├── files.rs       # 目录浏览与文件操作
│       │   ├── clipboard.rs   # 历史记录
│       │   ├── monitor.rs     # sysinfo 采样
│       │   ├── plugin.rs      # manifest 解析
│       │   ├── i18n.rs        # .strings 解析 + 占位符填充
│       │   ├── settings.rs    # settings.json 持久化
│       │   ├── platform.rs    # Finder / 剪贴板 / 打开路径
│       │   └── icon.rs        # 托盘图标位图
│       └── system/            # 原生集成
│           ├── menu.rs        # 应用菜单栏（加速器、动态项）
│           ├── tray.rs        # 托盘图标 + 托盘菜单
│           ├── shortcut.rs    # 全局快捷键 ⌘+L
│           ├── autolaunch.rs  # LaunchAgent 开机自启
│           └── activation.rs  # Dock / 菜单栏模式切换（objc2）
├── frontend/                  # Web 前端
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── app/               # 应用外壳
│       │   ├── Sidebar.tsx
│       │   ├── TopBar.tsx
│       │   ├── Toast.tsx
│       │   └── navigation.ts
│       ├── components/        # 通用组件
│       │   ├── VirtualList.tsx
│       │   ├── DataTable.tsx
│       │   ├── SearchField.tsx
│       │   ├── Kicker.tsx
│       │   ├── Card.tsx
│       │   ├── Checkbox.tsx
│       │   ├── Segmented.tsx
│       │   ├── Modal.tsx
│       │   └── Sparkline.tsx
│       ├── pages/
│       │   ├── Processes.tsx
│       │   ├── Files.tsx
│       │   ├── Clipboard.tsx
│       │   ├── Monitor.tsx
│       │   ├── Plugins.tsx
│       │   └── Settings.tsx
│       ├── lib/
│       │   ├── bridge.ts      # invoke / listen 封装
│       │   ├── store.ts       # 轻量状态容器
│       │   ├── format.ts      # 字节 / 时间格式化
│       │   ├── i18n.ts        # 前端插值（%@）
│       │   └── native.ts      # 原生菜单 / 托盘 / 窗口调用
│       └── styles/
│           ├── tokens.css     # cindy.cn 设计令牌（CSS 变量）
│           ├── base.css       # 排版与布局基线
│           └── native.css     # macOS 化控件与滚动条
├── assets/fonts/              # 复用现有 Anton + Geist Mono（SIL OFL）
├── locales/                   # 简体中文 / English 文案表（不变）
└── docs/
```

### 4.1 从 egui 版迁移的模块

| 现有位置 | 新位置 | 处理方式 |
| --- | --- | --- |
| `src/process.rs` `files.rs` `clipboard.rs` `monitor.rs` `plugin.rs` `i18n.rs` `settings.rs` `platform.rs` `icon.rs` | `src-tauri/src/core/` | **原样搬运**，仅调整 crate 内引用路径；保留全部单元测试 |
| `src/menubar.rs` `shortcut.rs` `autolaunch.rs` | `src-tauri/src/system/` | 改写为 Tauri 原生 API（`TrayIconBuilder` / 菜单加速器 / LaunchAgent） |
| `src/app.rs` `ui_kit.rs` `theme.rs` `fonts.rs` `views/*` | `frontend/` + `src-tauri/src/styles` | **整体废弃**，由前端重写 |
| `src/events.rs` | `src-tauri/src/events.rs` | 改为 Tauri 事件定义 |

预计 Rust 侧净减少约 2700 行（egui 层整体废弃），前端新增约 2500 行 TS/TSX，另加约 800 行 Tauri 桥接与原生集成代码。

## 5. 进程模型与数据流

```
Rust 后台线程                      Tauri 桥接                      前端
──────────────                    ──────────                      ────
monitor 采样 ──┐
clipboard 轮询 ─┤
notify 目录监听 ─┼─► emit(event) ──► listen() ──► store ──► React 渲染
process 定时刷新 ─┘
                    ◄──── invoke(cmd, args) ◄──── 用户交互
```

规则：

1. **Rust 是唯一权威状态源。** 前端不缓存业务真相，只持有"当前渲染快照"。
2. **搜索与排序在前端本地完成**，不回传后端 → 输入零延迟，不产生 IPC 压力。
3. **事件负载保持轻量**：进程列表 500 行约 60KB JSON / 5 秒一次，可接受。
4. 后台线程与 UI 解耦：Rust 侧状态用 `Arc<Mutex<…>>`，采样线程只写状态并发事件。

## 6. Command 与 Event 契约

### 6.1 Commands

| 分组 | 命令 | 参数 | 返回 |
| --- | --- | --- | --- |
| 进程 | `list_processes` | — | `Vec<ProcessRow>` |
| | `refresh_processes` | — | `Vec<ProcessRow>` |
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
| | `remove_history_item` | `id: u64` | `Result<(), String>` |
| | `clear_history` | — | `Result<(), String>` |
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

### 6.2 Events

| 事件 | 负载 | 触发时机 |
| --- | --- | --- |
| `processes` | `Vec<ProcessRow>` | 定时刷新完成 |
| `system-info` | `SystemInfo` | 采样周期到达 |
| `clipboard` | `String` | 剪贴板内容变化 |
| `files-changed` | `()` | 当前目录增删 |
| `tray-open` | `()` | 点击托盘图标 |
| `tray-quit` | `()` | 托盘菜单退出 |

### 6.3 数据结构

Rust 侧类型用 `serde::Serialize` 直接导出给前端（`camelCase`），前端用 TypeScript interface 镜像定义，集中在 `frontend/src/lib/types.ts`。

`SettingsPatch` 定义为「全字段可选」的补丁结构（`Option<T>` 包裹每个可调项），前端只提交实际改动的字段，Rust 侧合并后落盘 `settings.json` 并返回完整 `Settings`。

## 7. 原生感落地清单

这是本次改版的核心，逐条对应用户提出的痛点。

| 元素 | 实现 | 对应痛点 |
| --- | --- | --- |
| 应用菜单栏 | Tauri `setAsAppMenu()`：关于 chuaker7 / 设置… / 隐藏 / 退出，含 ⌘, ⌘L, ⌘W 加速器 | 缺少原生交互 |
| 右键菜单 | `Menu.popup()`：进程行、文件行、剪贴板条目各自一套原生菜单 | 缺少原生交互 |
| 托盘菜单 | `TrayIconBuilder` + 原生菜单，实时显示 CPU / 内存 / 磁盘 | 缺少原生交互 |
| 文件对话框 | Tauri dialog 插件 | 缺少原生交互 |
| 文本输入 | 原生 `<input>`，只去边框保留系统字体 / IME / 拼写检查 / 选区 / 光标 | 不像原生 |
| 滚动条 | `::-webkit-scrollbar` 细窄样式，滚动时显形（macOS overlay 行为） | 不像原生 |
| 复选框 | 自绘 macOS 方块：hover 变灰底、选中填强调色 + ✓，键盘可聚焦 | 不像原生 |
| 分段控件 / 下拉 | 自绘 popup，macOS 菜单观感；不用原生 `<select>` | 不像原生 |
| 弹窗 | 半透明遮罩 + 圆角卡片 + 阴影（sheet 观感），Esc 关闭 | 不像原生 |
| 焦点环 | `:focus-visible` 使用系统蓝 focus ring | 不像原生 |
| 标题栏 | 优先用 Tauri 能力；若不支持透明标题栏则保留系统标题栏，不引入 objc2 hack | 不像原生 |
| 首帧 | WebView 直接渲染 HTML，无多遍布局丢弃 → 不存在 egui 版的整帧空白 | 卡顿闪烁 |
| 大列表 | 虚拟滚动，500 行只渲染约 30 个 DOM 节点 | 卡顿闪烁 |
| 曲线 | Canvas 手绘，非 SVG 逐点 | 卡顿闪烁 |

**激活策略**：`objc2-app-kit` 直接调用 `NSApplication.setActivationPolicy_()` 在 `Regular`（占 Dock）与 `Accessory`（仅菜单栏）之间切换。`objc2` 依赖仅用于这一处。

## 8. 前端工程约定

### 8.1 状态

- `store.ts`：约 60 行的 `useSyncExternalStore` 容器，持有 `theme` / `accent` / `locale` / `settings` / `activeTab` / `toast`。
- 页面数据：`useCommand(cmd, args, deps)` hook 调 Rust；`useTauriEvent(name, handler)` hook 订阅事件。
- 不引状态库、不引路由库（6 个标签页用一个 `activeTab` 状态切换，不做 URL 路由）。

### 8.2 样式

- `tokens.css`：cindy.cn 设计令牌 → CSS 变量（ink / paper 两套，三档主题 + 4 强调色）。
- 主题切换：改 `document.documentElement.dataset.theme`，CSS 变量自动生效，不触发 React 重渲染。
- 字体：`@font-face` 引入 Anton + Geist Mono；中文走系统字体栈（PingFang / Hiragino / Songti）。
- 不引 CSS 框架，样式手写以精确匹配参考站。

### 8.3 性能

- 虚拟列表固定行高 26px，overscan 8 行。
- 行组件 `React.memo`，行数据引用稳定。
- 过滤 / 排序用 `useMemo`，输入框 `useDeferredValue`。
- 监控曲线 Canvas 重绘走 `requestAnimationFrame`。

### 8.4 测试

- 前端：`vitest`，只覆盖纯函数（`format.ts`、`i18n.ts` 插值）与组件冒烟渲染。
- Rust：`cargo test` 保留 core 的全部现有测试（19 个），新增 command 层的序列化测试。
- 类型：`tsc --noEmit` 必须零错误。

## 9. 视觉验收

- 前端用 `vite dev` 起服务，本机 Google Chrome headless 按 1080×720（与应用窗口同尺寸）截图比对。
- 原生部分（菜单栏、托盘、右键菜单、激活策略切换）人工验收。
- 每阶段产出的截图归档到 `docs/screenshots/`，与 egui 版对照。

## 10. 实施阶段

| 阶段 | 内容 | 验收标准 |
| --- | --- | --- |
| P0 | Tauri v2 + React + TS + Vite 空壳 | `npm run tauri dev` 窗口正常显示 |
| P1 | `core/` 迁移，移除 egui 依赖 | `cargo test` ≥ 19 通过，逻辑零回归 |
| P2 | commands / events / 原生菜单·托盘·快捷键·自启·激活策略 | ⌘L 唤起、托盘菜单、应用菜单栏、原生右键菜单均可用 |
| P3 | 6 个页面 + 设计令牌 + macOS 化控件 | 功能与原版逐项对等，截图比对通过 |
| P4 | 虚拟列表、曲线、性能调优、README 更新、.app 打包 | 500 行滚动流畅；`tauri build` 产出可运行 .app |

## 11. 仓库策略

- 同一个 `chuaker7/Ki6` 仓库继续演进，主分支 `main`。（GitHub 仓库名是 `Ki6`；包名 / crate 名 / 类型名 / 数据目录 / LaunchAgent 标签等**一切标识符统一为 `chuaker7`**。两者刻意不同，不要"顺手统一"。）
- 当前 egui 版保留在 `legacy-egui` 分支（指向 `b36b8dc`），随时可对照或回退。
- 提交信息中文，结尾附 `AI-Coding: true` / `AI-Tool: TophantAI`。

## 12. 风险与对策

| 风险 | 影响 | 对策 |
| --- | --- | --- |
| Tauri v2 对 `NSApp.activationPolicy` 无直接 API | 菜单栏模式切换需绕路 | 已确认用 `objc2-app-kit` 直调；失败则降级为"仅隐藏窗口"（窗口不占 Dock，行为差异可接受） |
| WKWebView 无法被 `EFRAME_SCREENSHOT_TO` 式截图 | 自动化视觉验收受限 | 前端用 Chrome headless 截图；完整窗口截图需系统录屏权限，作为可选增强 |
| 500 行进程数据的 IPC 序列化开销 | 每 5 秒一次全量传输 | 负载约 60KB，可接受；若实测偏高改为增量（仅传变化的行） |
| npm 依赖树体积 | 安装慢、CI 慢 | 只引 `react` `react-dom` + `vite` + `typescript` + `vitest`，其余自研 |
