//! 文件管理，对应 Swift 版的 `FileManagerView.swift`。
//!
//! Swift 版用 `FileManager.contentsOfDirectory` + `DispatchSource` 监听目录变更；
//! Rust 版用 `std::fs` 读取目录、用 `notify` 监听变更，语义一致。

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 目录中的一个条目。
#[derive(Clone, Debug)]
pub struct FileItem {
    /// 完整路径。
    pub path: PathBuf,
    /// 文件名。
    pub name: String,
    /// 是否是目录。
    pub is_dir: bool,
    /// 字节大小。
    pub size: i64,
    /// 修改时间。
    pub modified: SystemTime,
}

impl FileItem {
    /// 人类可读的大小文本，目录显示 `-`。
    pub fn size_string(&self) -> String {
        if self.is_dir {
            return "-".to_string();
        }
        format_bytes(self.size)
    }
}

/// 把字节数格式化成 `1.2 KB` 风格。
pub fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 6] = ["bytes", "KB", "MB", "GB", "TB", "PB"];
    if bytes < 1024 {
        return format!("{} bytes", bytes);
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", value, UNITS[unit])
}

/// 文件管理器。
pub struct FileManager {
    /// 当前所在目录。
    pub current_path: PathBuf,
    /// 当前目录下的条目。
    pub files: Vec<FileItem>,
    /// 选中的条目路径。
    pub selected: Option<PathBuf>,
    /// 是否正在加载。
    pub is_loading: bool,
}

impl Default for FileManager {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        Self {
            current_path: home,
            files: Vec::new(),
            selected: None,
            is_loading: false,
        }
    }
}

impl FileManager {
    /// 读取当前目录内容。
    pub fn load_files(&mut self) -> Vec<FileItem> {
        self.is_loading = true;
        let items = fetch_files(&self.current_path);
        self.files = items.clone();
        self.is_loading = false;
        items
    }

    /// 进入指定目录。
    pub fn navigate_to(&mut self, path: PathBuf) {
        self.current_path = path;
        self.selected = None;
        self.load_files();
    }

    /// 返回上级目录。
    pub fn go_up(&mut self) -> bool {
        let Some(parent) = self.current_path.parent() else {
            return false;
        };
        if parent == self.current_path {
            return false;
        }
        self.navigate_to(parent.to_path_buf());
        true
    }

    /// 新建文件夹。
    pub fn create_folder(&mut self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let target = self.current_path.join(name);
        if std::fs::create_dir(&target).is_err() {
            return false;
        }
        self.load_files();
        true
    }

    /// 新建空文件。
    pub fn create_file(&mut self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let target = self.current_path.join(name);
        if std::fs::write(&target, "").is_err() {
            return false;
        }
        self.load_files();
        true
    }

    /// 删除文件（移到废纸篓）。
    pub fn delete_file(&mut self, path: &Path) -> bool {
        if trash::delete(path).is_err() {
            return false;
        }
        self.load_files();
        true
    }

    /// 重命名。
    pub fn rename_file(&mut self, path: &Path, new_name: &str) -> bool {
        if new_name.is_empty() {
            return false;
        }
        let Some(parent) = path.parent() else {
            return false;
        };
        let target = parent.join(new_name);
        if std::fs::rename(path, &target).is_err() {
            return false;
        }
        self.load_files();
        true
    }
}

/// 读取目录内容，跳过隐藏文件，文件夹优先并按名称排序。
fn fetch_files(dir: &Path) -> Vec<FileItem> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut items: Vec<FileItem> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        // 与 Swift 的 .skipsHiddenFiles 保持一致。
        if name.starts_with('.') {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        items.push(FileItem {
            path: entry.path(),
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len() as i64,
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }

    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    items
}
