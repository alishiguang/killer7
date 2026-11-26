# Ki6 性能记录

## 测量环境

- macOS 26.3.1（darwin 25.3.0 arm64）
- 测量引擎：Chromium 148 / Electron 42.11.16（会话内置 Browser 面板）
- Node v24.17.0 / Vite 7.1.5 / React 19.3.1
- rustc 1.90.0
- 数据规模：进程列表 **500 行**（`processListLimit` 上限值）

> **为什么不是 WKWebView**
>
> 测的是 `npm run build` 产出的**同一份前端 bundle**，装在会话内置的
> Chromium/Electron 里跑；Tauri 打包后内嵌的是 WKWebView，**引擎不同**。
> DOM 结构、React 渲染行为、虚拟列表算法完全一致，所以「渲染了多少行 /
> 多少 DOM 元素 / 推送后多久更新」这三项与引擎无关，结论可直接迁移；
> **帧率与首帧时间与引擎强相关**，下面标注了实测值来源，换引擎需复测。

## 基线指标

| 指标 | 目标（验收线） | 实测 | 判定 |
| --- | --- | --- | --- |
| 进程列表 500 行时**渲染出的行数** | ≤ 60 | **33**（列表视口 397px）/ **44**（视口 679px） | ✅ |
| 进程列表 500 行时**列表区域 DOM 元素数** | （参考） | 172（397px 视口）/ 227（679px 视口） | 参考值 |
| 进程列表 500 行时**整页 DOM 元素数** | （参考） | 240（397px 视口）/ 295（679px 视口） | 参考值 |
| 首帧可见时间（FCP） | ≤ 300ms | **168ms**（`first-paint` 同为 168ms，无空白首帧） | ✅ |
| 滚动帧率 | ≥ 55fps | **120fps**（平均 8.33ms/帧，p95 9.8ms，最差 10.4ms，长帧 0） | ✅ |
| 进程事件负载大小 | ≤ 80KB | **51.8KB**（500 行，51,846 字节，紧凑 JSON） | ✅ |
| 后端推送 500 行 → DOM 更新完成 | （参考） | 平均 **8.3ms**（最大 8.4ms） | ✅ |

### 「DOM 节点 ≤ 60」的口径

计划里写的是「进程列表 500 行时 DOM 节点 ≤ 60（虚拟列表，行高 26px，overscan 8）」，
Task 19 的提交说明写的是「500 行只渲染约 28 个节点」—— **这里指渲染出的行数**，
不是 DOM 元素个数。

本项目一行进程是 4 个 DOM 元素（`.vrow` + 三个 `span`），所以：

- **行数口径：33 / 44 ≤ 60 ✅**
- 元素个数口径：172 / 227（必然是行数的约 4 倍，与「≤ 60」不可能同时成立）

虚拟列表在 500 行时只渲染 **33 行**（视口 397px：可见 16 行 + overscan 8），
其余 467 行不在 DOM 里。

## 测量方法

1. 构建前端产物：`npm run build` → `frontend/dist/`。
2. **注入 IPC 模拟层**：把 `frontend/dist` 复制到临时目录，在 `index.html` 里
   `type="module"` 脚本**之前**注入一段 `window.__TAURI_INTERNALS__` 模拟代码：
   - `invoke(cmd)` 按命令名返回假数据（`list_processes` 返回 500 行、
     `get_settings` / `get_strings` 返回真实值 —— 文案表直接从
     `locales/zh-Hans.lproj/Localizable.strings` 解析，保证测的是真实文案）；
   - `transformCallback(cb)` 记录回调，以便**模拟后端推送**。
   - 模拟层只存在于临时目录，**不进仓库**。
3. 本地起一个静态服务（`python3 -m http.server`），用会话内置 Browser 打开。
4. 在页面里量：

   | 指标 | 取值方式 |
   | --- | --- |
   | 渲染行数 | `document.querySelectorAll('.vtable .vrow:not(.vrow--head)').length` |
   | DOM 元素数 | `document.querySelectorAll('.vtable *').length` / `document.querySelectorAll('*').length` |
   | 首帧 | `performance.getEntriesByType('paint')` 的 `first-contentful-paint` |
   | 滚动帧率 | 连续 2 秒滚动，按 `requestAnimationFrame` 的帧间隔统计均值 / p95 / 最差值 / 长帧数 |
   | 事件负载 | `new Blob([JSON.stringify(500 行)]).size`（与 Rust 侧 `serde_json` 的紧凑输出一致） |
   | 推送延迟 | 调用监听 `processes` 的回调推入新数据，轮询 DOM 直到内容更新，返回耗时 |

5. 复现脚本与临时目录：`/tmp/perf/`（一次性，不入库）。

## 实现要点

| 手段 | 作用 |
| --- | --- |
| `components/VirtualList.tsx` 自研虚拟列表（行高 26px / overscan 8） | 500 行只渲染 33 行 |
| `components/DataTable.tsx` 的 `TableRow` 用 `memo` 包装 | 排序或选中变化时，不变的行不重绘 |
| `pages/Processes.tsx` 的 `ProcessRowView` 用 `memo` 包装 | 同上（虚拟列表的渲染行） |
| 三个页面的搜索词走 `useDeferredValue` | 打字时不阻塞输入框，长列表过滤滞后一拍 |
| `useI18n()` 的 `t` 用 `useCallback` 固定引用 | 各页 `useMemo` / `useCallback` 不再每次渲染失效 |

### 一处诚实的说明：`memo` 对「后端整轮刷新」帮不上忙

`ProcessRowView` 的 `memo` 比较的是 `row` 的**引用**。后端每轮推送的进程列表
是 serde 重新反序列化出来的**全新对象**，引用全变 → memo 全部失效，行照样重绘。

- 推一轮**全新 500 行**：平均 **8.3ms**（≈1 帧）
- 推一轮**完全相同的内容**（对象也是新的）：**8.2ms** —— 与上面基本一致，
  印证了上面这条判断

结论：memo 真正生效的场景是**排序变化**与**选中变化**（这两种情况下
`sorted` 只是重排、行对象引用不变）；对「整轮数据刷新」无优化。
即便如此，44 行的重绘只要 8.3ms，与默认 2 秒的刷新间隔相比完全可以忽略，
**没有为了指标好看而做无效优化**。

## 结论

四条验收线（渲染行数、首帧、滚动帧率、事件负载）**全部达标**：

- 500 行只渲染 33 行（≤ 60）
- 首帧 168ms，无空白首帧（≤ 300ms）
- 滚动 120fps，零长帧（≥ 55fps）
- 事件负载 51.8KB（≤ 80KB）

**未在 WKWebView 里复测**：帧率与首帧时间依赖渲染引擎，打包后的 `.app`
需要再测一次（Task 27 的视觉验收环节一并做）。DOM 规模与推送延迟与引擎无关，
结论可直接迁移。
