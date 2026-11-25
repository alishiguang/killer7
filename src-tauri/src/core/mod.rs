//! 与 UI 无关的业务逻辑。
//!
//! 本层**不得**依赖 `tauri` crate：它只负责取数、解析、持久化，
//! 线程调度与事件派发由 `system/` 负责。

pub mod i18n;
pub mod icon;
pub mod platform;
pub mod plugin;
pub mod settings;
pub mod theme;
