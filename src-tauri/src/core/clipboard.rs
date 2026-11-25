//! 剪贴板管理，对应 Swift 版的 `ClipboardManager.swift`。
//!
//! Swift 版每秒比对 `NSPasteboard.changeCount` 判断剪贴板是否变化；Rust 版没有直接
//! 的 changeCount 接口，改为每秒读取一次剪贴板文本并与上次内容比较，效果等价。

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::core::settings::clipboard_history_path;

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
    /// 下一个可用 id。
    next_id: u64,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self {
            current_content: String::new(),
            history: Vec::new(),
            max_history_count: 100,
            next_id: 0,
        }
    }
}

impl ClipboardManager {
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

    /// 把历史记录写回磁盘。
    pub fn save_history(&self) {
        persist_history(&self.history);
    }

    /// 向历史记录头部插入一条内容，并裁剪到上限。
    ///
    /// 只改内存，**不落盘** —— 落盘由调用方在释放锁之后调用 [`persist_history`]，
    /// 避免磁盘 IO 期间占着剪贴板锁。返回是否真的插入了一条。
    pub fn add_to_history(&mut self, content: &str, kind: &str) -> bool {
        // 与 Swift 版一致：不重复记录与上一条相同的内容。
        if self.history.first().map(|item| item.content.as_str()) == Some(content) {
            return false;
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

        true
    }

    /// 删除某条历史（按稳定 id）。
    ///
    /// 只改内存，**不落盘** —— 落盘由调用方在释放锁之后调用 [`persist_history`]。
    /// 返回是否真的删掉了一条。
    pub fn delete_history_item(&mut self, id: u64) -> bool {
        let Some(index) = self.history.iter().position(|item| item.id == id) else {
            return false;
        };
        self.history.remove(index);
        true
    }

    /// 清空历史。
    ///
    /// 只改内存，**不落盘** —— 落盘由调用方在释放锁之后调用 [`persist_history`]。
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 复制指定内容到系统剪贴板。
    pub fn copy_to_clipboard(&mut self, content: &str) {
        if crate::core::platform::copy_to_clipboard(content) {
            self.current_content = content.to_string();
        }
    }
}

/// 读取系统剪贴板文本。
pub fn read_current() -> Option<String> {
    crate::core::platform::read_clipboard()
}

/// 把历史记录写到磁盘。
///
/// 单独拆成自由函数，是为了让调用方能在**释放剪贴板锁之后**再落盘：
/// 磁盘 IO 可能耗时几十毫秒，持锁期间会把剪贴板轮询与所有剪贴板命令一起堵住。
pub fn persist_history(history: &[HistoryEntry]) {
    let path = clipboard_history_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(history) {
        let _ = std::fs::write(path, text);
    }
}

/// 规范化历史条目的 id：缺失（为 0）或重复时按顺序重排为 `1..=n`。
pub fn normalize_ids(history: &mut [HistoryEntry]) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_missing_ids() {
        let mut history = vec![
            HistoryEntry {
                id: 0,
                content: "a".into(),
                timestamp: 1,
                kind: "text".into(),
            },
            HistoryEntry {
                id: 0,
                content: "b".into(),
                timestamp: 2,
                kind: "text".into(),
            },
        ];
        normalize_ids(&mut history);
        assert_eq!(history[0].id, 1);
        assert_eq!(history[1].id, 2);
    }

    #[test]
    fn keeps_unique_ids() {
        let mut history = vec![
            HistoryEntry {
                id: 7,
                content: "a".into(),
                timestamp: 1,
                kind: "text".into(),
            },
            HistoryEntry {
                id: 9,
                content: "b".into(),
                timestamp: 2,
                kind: "text".into(),
            },
        ];
        normalize_ids(&mut history);
        assert_eq!(history[0].id, 7);
        assert_eq!(history[1].id, 9);
    }

    #[test]
    fn duplicate_ids_are_rewritten() {
        let mut history = vec![
            HistoryEntry {
                id: 3,
                content: "a".into(),
                timestamp: 1,
                kind: "text".into(),
            },
            HistoryEntry {
                id: 3,
                content: "b".into(),
                timestamp: 2,
                kind: "text".into(),
            },
        ];
        normalize_ids(&mut history);
        assert_eq!(history[0].id, 1);
        assert_eq!(history[1].id, 2);
    }
}
