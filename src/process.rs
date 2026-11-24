//! 进程管理，对应 Swift 版的 `ProcessManager.swift`。
//!
//! Swift 版调用 `/bin/ps -eo pid,comm` 获取进程，再用 `lsof` 解析监听端口并缓存
//! 30 秒；这里保持同样的数据来源与缓存策略，只把字符串解析挪到 Rust 侧完成。

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::events::AppEvent;

/// 进程监听的端口。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortInfo {
    /// 绑定地址，例如 `*`、`127.0.0.1`。
    pub address: String,
    /// 端口号。
    pub port: u16,
}

impl PortInfo {
    /// `地址:端口` 形式的展示文本。
    pub fn display(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

/// 一个进程条目。
#[derive(Clone, Debug)]
pub struct ProcessInfo {
    /// 进程 PID。
    pub pid: i32,
    /// 可执行文件完整路径。
    pub full_path: String,
    /// 展示名称（路径的最后一段）。
    pub display_name: String,
    /// 该进程正在监听的端口。
    pub ports: Vec<PortInfo>,
}

impl ProcessInfo {
    /// 最小监听端口，没有端口时返回 `None`。
    pub fn min_port(&self) -> Option<u16> {
        self.ports.iter().map(|p| p.port).min()
    }
}

/// 进程管理器。
#[derive(Default)]
pub struct ProcessManager {
    /// 当前进程列表。
    pub processes: Vec<ProcessInfo>,
    /// 是否正在加载。
    pub is_loading: bool,
    /// 端口信息缓存。
    port_cache: HashMap<i32, Vec<PortInfo>>,
    /// 上次抓取端口的时间。
    last_port_fetch: Option<Instant>,
}

impl ProcessManager {
    /// 端口缓存有效期。
    const PORT_CACHE_TTL: Duration = Duration::from_secs(30);

    /// 同步抓取一次进程列表。
    ///
    /// `ps` 与 `lsof` 都在几十毫秒内完成，启动时同步跑一次，窗口打开就能看到
    /// 进程列表，不必等第一轮异步刷新。
    pub fn load_blocking(&mut self, limit: usize) {
        let port_map = fetch_listening_ports();
        self.processes = fetch_all_processes(&port_map, limit);
        self.port_cache = port_map;
        self.last_port_fetch = Some(Instant::now());
    }

    /// 触发一次异步刷新。
    ///
    /// `limit` 为 0 表示不限制进程数量。
    pub fn refresh(&mut self, limit: usize, tx: &Sender<AppEvent>) {
        if self.is_loading {
            return;
        }
        self.is_loading = true;
        let _ = tx.send(AppEvent::ProcessLoading(true));

        // 端口缓存未过期时直接复用，避免重复调用 lsof。
        let cached = match self.last_port_fetch {
            Some(ts) if ts.elapsed() < Self::PORT_CACHE_TTL => Some(self.port_cache.clone()),
            _ => None,
        };

        let tx = tx.clone();
        std::thread::spawn(move || {
            let (port_map, refreshed) = match cached {
                Some(map) => (map, false),
                None => (fetch_listening_ports(), true),
            };

            let list = fetch_all_processes(&port_map, limit);
            let _ = tx.send(AppEvent::Processes(list));
            if refreshed {
                let _ = tx.send(AppEvent::PortCacheUpdated(port_map));
            }
            let _ = tx.send(AppEvent::ProcessLoading(false));
        });
    }

    /// 更新缓存的端口信息。
    pub fn set_port_cache(&mut self, map: HashMap<i32, Vec<PortInfo>>) {
        self.port_cache = map;
        self.last_port_fetch = Some(Instant::now());
    }

    /// 结束指定进程，返回是否成功。
    pub fn kill_process(pid: i32) -> bool {
        let status = Command::new("/bin/kill")
            .arg("-9")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        matches!(status, Ok(status) if status.success())
    }
}

/// 解析 `lsof` 输出，得到 PID -> 监听端口列表。
///
/// 输出形如：
/// `COMMAND  PID USER  FD  TYPE DEVICE SIZE/OFF NODE NAME`，NAME 为 `127.0.0.1:8080`。
fn fetch_listening_ports() -> HashMap<i32, Vec<PortInfo>> {
    let output = match Command::new("/usr/sbin/lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(output) => output,
        Err(_) => return HashMap::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut result: HashMap<i32, Vec<PortInfo>> = HashMap::new();

    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 9 {
            continue;
        }

        let Ok(pid) = fields[1].parse::<i32>() else {
            continue;
        };
        let Some((address, port)) = split_endpoint(fields[8]) else {
            continue;
        };

        let info = PortInfo { address, port };
        let entry = result.entry(pid).or_default();
        if !entry.contains(&info) {
            entry.push(info);
        }
    }

    for ports in result.values_mut() {
        ports.sort_by(|a, b| a.port.cmp(&b.port).then_with(|| a.address.cmp(&b.address)));
    }

    result
}

/// 把 `host:port` 拆成地址与端口，`*` 表示监听所有网卡。
fn split_endpoint(endpoint: &str) -> Option<(String, u16)> {
    let (address, port) = endpoint.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    if port == 0 {
        return None;
    }
    let address = if address.is_empty() { "*" } else { address };
    Some((address.to_string(), port))
}

/// 通过 `ps` 抓取进程列表。
///
/// `limit` 为 0 表示不限制。Swift 版只显示「有监听端口」或「PID > 100」的进程，
/// 这里保持相同过滤规则。
fn fetch_all_processes(port_map: &HashMap<i32, Vec<PortInfo>>, limit: usize) -> Vec<ProcessInfo> {
    let output = match Command::new("/bin/ps")
        .args(["-eo", "pid=,comm="])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut list: Vec<ProcessInfo> = Vec::new();

    for line in text.lines() {
        if limit > 0 && list.len() >= limit {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some((pid_text, rest)) = trimmed.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<i32>() else {
            continue;
        };
        if pid <= 0 {
            continue;
        }

        let full_path = rest.trim().to_string();
        let display_name = full_path
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or(&full_path)
            .to_string();

        let ports = port_map.get(&pid).cloned().unwrap_or_default();

        // 过滤掉没有端口且 PID 很小的内核进程。
        if ports.is_empty() && pid <= 100 {
            continue;
        }

        list.push(ProcessInfo {
            pid,
            full_path,
            display_name,
            ports,
        });
    }

    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_endpoint() {
        assert_eq!(
            split_endpoint("127.0.0.1:8080"),
            Some(("127.0.0.1".to_string(), 8080))
        );
        assert_eq!(split_endpoint("*:443"), Some(("*".to_string(), 443)));
        assert_eq!(split_endpoint("noport"), None);
    }

    #[test]
    fn min_port_reports_smallest() {
        let p = ProcessInfo {
            pid: 1,
            full_path: "/bin/x".into(),
            display_name: "x".into(),
            ports: vec![
                PortInfo {
                    address: "*".into(),
                    port: 9000,
                },
                PortInfo {
                    address: "*".into(),
                    port: 300,
                },
            ],
        };
        assert_eq!(p.min_port(), Some(300));
    }
}
