//! 系统监控，对应 Swift 版的 `SystemMonitor.swift`。
//!
//! Swift 版直接调用 `host_processor_info` / `host_statistics64` / `getifaddrs` 读取
//! 指标；Rust 版改用 `sysinfo` 提供的跨平台封装，指标口径保持一致：
//! CPU 使用率、内存使用率、磁盘使用率、网络上传 / 下载速度。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sysinfo::System;

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

/// 系统监控器。
#[derive(Clone, Debug, Default)]
pub struct SystemMonitor {
    /// 当前指标。
    pub system_info: SystemInfo,
    /// CPU 使用率采样历史，用于绘制曲线。
    pub cpu_history: Vec<f64>,
}

impl SystemMonitor {
    /// 记录一次采样。
    pub fn push_sample(&mut self, info: SystemInfo) {
        self.system_info = info;
        self.cpu_history.push(info.cpu_usage);

        // 只保留最近 120 个采样点，避免内存无限增长。
        const MAX_SAMPLES: usize = 120;
        if self.cpu_history.len() > MAX_SAMPLES {
            self.cpu_history.remove(0);
        }
    }
}

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

/// 所有核心的平均使用率。
fn average_cpu_usage(system: &System) -> f64 {
    let cpus = system.cpus();
    if cpus.is_empty() {
        return 0.0;
    }
    let total: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    (total / cpus.len() as f32) as f64
}

/// 内存使用率（%）。
fn memory_usage(system: &System) -> f64 {
    let total = system.total_memory();
    if total == 0 {
        return 0.0;
    }
    system.used_memory() as f64 / total as f64 * 100.0
}

/// 磁盘使用率（%），取主目录所在卷。
fn disk_usage() -> f64 {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let list = disks.list();
    if list.is_empty() {
        return 0.0;
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

    let disk = list
        .iter()
        .find(|disk| home.starts_with(disk.mount_point()))
        .or_else(|| list.first());

    let Some(disk) = disk else {
        return 0.0;
    };

    let total = disk.total_space();
    if total == 0 {
        return 0.0;
    }
    let used = total.saturating_sub(disk.available_space());
    used as f64 / total as f64 * 100.0
}

/// 网络上传 / 下载速度（MB/s）。
///
/// `sysinfo` 在刷新时给出的 `received()` / `transmitted()` 是本次刷新周期的增量，
/// 除以采样间隔即可得到速度。计数器必须复用同一个 `Networks` 实例，否则每次
/// 都是新建基线，速率恒为 0。
fn network_usage(networks: &mut sysinfo::Networks, interval: f64) -> (f64, f64) {
    networks.refresh(true);

    let mut received: u64 = 0;
    let mut transmitted: u64 = 0;

    for (name, data) in networks.list() {
        // 与 Swift 版一致，只统计常见网卡。
        if !(name.starts_with("en") || name.starts_with("pdp_ip") || name.starts_with("awdl")) {
            continue;
        }
        received += data.received();
        transmitted += data.transmitted();
    }

    let divisor = interval.max(0.5) * 1024.0 * 1024.0;
    (transmitted as f64 / divisor, received as f64 / divisor)
}
