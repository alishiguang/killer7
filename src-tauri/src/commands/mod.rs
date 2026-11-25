//! Tauri command 门面。
//!
//! 这一层只做「编解码 + 转发」：参数来自前端，调用 core 或 system，
//! 返回可序列化的结果。业务逻辑不写在这里。

pub mod dto;
