//! 前后端契约的数据结构。
//!
//! 所有字段用 camelCase，与 `frontend/src/lib/types.ts` 一一对应。
//! core 内部类型保持自己的形状，转换只在这一层发生。

use serde::{Deserialize, Serialize};

use crate::core::clipboard::HistoryEntry;
use crate::core::files::FileItem;
use crate::core::monitor::SystemInfo;
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
    /// 当前进程总数（取自进程缓存，不额外采集）。
    pub process_count: usize,
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
///
/// `rename_all` 在 enum 上只改**变体名**，`rename_all_fields` 才改**变体内部字段**。
/// 前端发的是 camelCase 的 payload（`{ pid, displayName, fullPath }`），
/// Task 25 会用 `serde_json::from_value::<ContextTarget>` 反序列化它——
/// 少了 `rename_all_fields` 会解析失败（`missing field 'display_name'`），
/// 右键菜单永远弹不出来。
///
/// `rename_all_fields` 自 serde 1.0.177 起可用；本项目依赖下限取 1.0.177，锁定 serde 1.0.229。
#[derive(Clone, Debug, Deserialize)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
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
            process_count: 0,
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
