//! 系统监控，对应 Swift 版的 `SystemMonitor.swift`。
//!
//! Swift 版直接调用 `host_processor_info` / `host_statistics64` / `getifaddrs` 读取
//! 指标；Rust 版改用 `sysinfo` 提供的跨平台封装，指标口径保持一致：
//! CPU 使用率、内存使用率、磁盘使用率、网络上传 / 下载速度。

use std::path::PathBuf;
use std::time::Instant;

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

/// 网络采样器：记住上次刷新网络计数器的时刻。
///
/// `sysinfo` 的 `NetworkData::received()` 返回的是「**自上次 refresh 起**的增量」。
/// 关闭网络监控时我们不刷新计数器，重新打开后的第一次刷新拿到的增量
/// 覆盖了**整个停用区间**。若仍用标称采样间隔做除数，速率会被放大到
/// 「停用时长 ÷ 采样间隔」倍（关 1 分钟、间隔 5s → 虚报 12 倍速率）。
///
/// 所以除数必须用两次刷新之间的**真实经过时间**。
#[derive(Debug, Default)]
pub struct NetworkSampler {
    /// 上次刷新网络计数器的时刻。
    last_refresh: Option<Instant>,
}

impl NetworkSampler {
    /// 新建一个采样器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 采样一次系统指标，网络速率按真实间隔计算。
    pub fn sample(
        &mut self,
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
            // 除数取「自上次刷新网络计数器起」的真实秒数。
            let divisor = self.divisor(interval);
            let usage = network_usage(networks, divisor);
            // 刷新已经发生，把基线推到此刻。
            self.last_refresh = Some(Instant::now());
            usage
        } else {
            // 关闭期间不刷新计数器，也不更新基线 ——
            // 基线必须停在「最后一次真实刷新」上，重新打开时才算得出真实间隔。
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

    /// 本次采样应使用的除数（秒）。
    ///
    /// 首次采样用标称间隔，之后用两次刷新之间的真实间隔。
    /// 下限 0.5 秒，避免刷新过快时除数趋零导致速率爆炸。
    fn divisor(&self, fallback: f64) -> f64 {
        let elapsed = match self.last_refresh {
            Some(ts) => ts.elapsed().as_secs_f64(),
            None => fallback,
        };
        elapsed.max(0.5)
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
///
/// `divisor` 是本次增量对应的**真实**间隔（秒），由 [`NetworkSampler`] 给出；
/// 标称采样间隔不能直接拿来用 —— 关闭网络监控期间不刷新计数器，
/// 重新打开时的增量覆盖整个停用区间。
fn network_usage(networks: &mut sysinfo::Networks, divisor: f64) -> (f64, f64) {
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

    let divisor = divisor.max(0.5) * 1024.0 * 1024.0;
    (transmitted as f64 / divisor, received as f64 / divisor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn divisor_falls_back_to_nominal_interval_on_first_sample() {
        let sampler = NetworkSampler::new();
        assert!((sampler.divisor(5.0) - 5.0).abs() < 1e-9);
    }

    /// 关闭网络监控 30 秒后重新打开：除数必须是 30 秒而不是标称的 5 秒，
    /// 否则速率会被虚报 6 倍。
    #[test]
    fn divisor_uses_real_elapsed_time_after_a_gap() {
        let mut sampler = NetworkSampler::new();
        sampler.last_refresh = Some(Instant::now() - Duration::from_secs(30));
        let divisor = sampler.divisor(5.0);
        assert!(
            (29.0..=31.0).contains(&divisor),
            "真实间隔 30s，除数应接近 30，实际 {divisor}"
        );
    }

    #[test]
    fn divisor_never_drops_below_half_a_second() {
        let mut sampler = NetworkSampler::new();
        sampler.last_refresh = Some(Instant::now());
        assert!((sampler.divisor(5.0) - 0.5).abs() < 1e-9);
    }
}
