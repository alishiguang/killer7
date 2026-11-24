//! 剪贴板管理，对应 Swift 版的 `ClipboardManager.swift`。
//!
//! Swift 版每秒比对 `NSPasteboard.changeCount` 判断剪贴板是否变化；Rust 版没有直接
//! 的 changeCount 接口，改为每秒读取一次剪贴板文本并与上次内容比较，效果等价。

use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::events::AppEvent;
use crate::settings::clipboard_history_path;

/// 剪贴板历史条目。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 文本内容。
    pub content: String,
    /// 时间戳（秒级 UNIX 时间）。
    pub timestamp: i64,
    /// 类型标签。
    pub kind: String,
}

impl HistoryEntry {
    /// 当前时间戳（秒）。
    pub fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

/// 剪贴板管理器。
pub struct ClipboardManager {
    /// 当前剪贴板文本。
    pub current_content: String,
    /// 历史记录。
    pub history: Vec<HistoryEntry>,
    /// 历史记录最大条数。
    pub max_history_count: usize,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self {
            current_content: String::new(),
            history: Vec::new(),
            max_history_count: 100,
        }
    }
}

impl ClipboardManager {
    /// 从磁盘载入历史记录。
    pub fn load_history(&mut self) {
        let Ok(text) = std::fs::read_to_string(clipboard_history_path()) else {
            return;
        };
        if let Ok(history) = serde_json::from_str::<Vec<HistoryEntry>>(&text) {
            self.history = history;
        }
    }

    /// 把历史记录写回磁盘。
    pub fn save_history(&self) {
        let path = clipboard_history_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&self.history) {
            let _ = std::fs::write(path, text);
        }
    }

    /// 向历史记录头部插入一条内容，并裁剪到上限。
    pub fn add_to_history(&mut self, content: &str, kind: &str) {
        // 与 Swift 版一致：不重复记录与上一条相同的内容。
        if self.history.first().map(|item| item.content.as_str()) == Some(content) {
            return;
        }

        self.history.insert(
            0,
            HistoryEntry {
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

    /// 删除某条历史。
    pub fn delete_history_item(&mut self, index: usize) {
        if index < self.history.len() {
            self.history.remove(index);
            self.save_history();
        }
    }

    /// 清空历史。
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.save_history();
    }

    /// 复制指定内容到系统剪贴板。
    pub fn copy_to_clipboard(&mut self, content: &str) {
        if crate::platform::copy_to_clipboard(content) {
            self.current_content = content.to_string();
        }
    }
}

/// 启动剪贴板监听线程：每秒检查一次，变化时通过通道通知 UI。
pub fn spawn_monitor(tx: Sender<AppEvent>) {
    std::thread::spawn(move || {
        let mut clipboard = arboard::Clipboard::new().ok();
        let mut last = String::new();

        loop {
            std::thread::sleep(Duration::from_secs(1));

            let text = match clipboard.as_mut() {
                Some(clipboard) => clipboard.get_text().ok(),
                None => crate::platform::read_clipboard(),
            };

            let Some(text) = text else { continue };
            if text == last {
                continue;
            }

            last = text.clone();
            let _ = tx.send(AppEvent::ClipboardContent(text));
        }
    });
}
