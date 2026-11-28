//! 进程管理。
//!
//! 数据来源按平台分派：
//!
//! - Unix（macOS / Linux）：`ps -eo pid=,comm=` 取进程，`lsof -nP -iTCP -sTCP:LISTEN`
//!   取监听端口，`kill -9` 结束进程；
//! - Windows：没有 `ps` / `lsof`，改用 `sysinfo` 取进程、`netstat -ano -p TCP`
//!   取监听端口、`taskkill /PID <pid> /F` 结束进程。
//!
//! 两边的对外接口完全一致，端口缓存策略也相同（30 秒 TTL）。

use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    /// 采集本身在几十到几百毫秒内完成，启动时同步跑一次，窗口打开就能看到
    /// 进程列表，不必等第一轮异步刷新。
    pub fn load_blocking(&mut self, limit: usize) {
        let port_map = backend::fetch_listening_ports();
        self.processes = backend::fetch_all_processes(&port_map, limit);
        self.port_cache = port_map;
        self.last_port_fetch = Some(Instant::now());
    }

    /// 采集一次进程列表（同步）。
    ///
    /// 端口缓存未过期时直接复用，避免重复调用系统命令。
    /// 线程调度由 `system::tasks` 负责，本方法保持纯同步以便单测。
    pub fn collect(&mut self, limit: usize) -> Vec<ProcessInfo> {
        let cached = match self.last_port_fetch {
            Some(ts) if ts.elapsed() < Self::PORT_CACHE_TTL => Some(self.port_cache.clone()),
            _ => None,
        };

        let (port_map, refreshed) = match cached {
            Some(map) => (map, false),
            None => (backend::fetch_listening_ports(), true),
        };

        if refreshed {
            self.port_cache = port_map.clone();
            self.last_port_fetch = Some(Instant::now());
        }

        backend::fetch_all_processes(&port_map, limit)
    }

    /// 采集一次进程列表并写回缓存。
    ///
    /// `collect()` 只更新端口缓存、**不写** `self.processes`；而 `list_processes`
    /// 命令读的正是 `self.processes`。后台循环与手动刷新命令都需要「采集 → 回写」
    /// 成对出现，收口到这里，避免两处各写一份而漏掉回写。
    pub fn collect_and_store(&mut self, limit: usize) {
        let list = self.collect(limit);
        self.processes = list;
    }

    /// 更新缓存的端口信息。
    pub fn set_port_cache(&mut self, map: HashMap<i32, Vec<PortInfo>>) {
        self.port_cache = map;
        self.last_port_fetch = Some(Instant::now());
    }

    /// 结束指定进程，返回是否成功。
    ///
    /// 非正 pid 一律拒绝：`kill(1)` 把 `0` 解释为「调用者所在进程组」、把 `-1`
    /// 解释为「该用户的所有进程」，直接透传会造成误杀。
    pub fn kill_process(pid: i32) -> bool {
        backend::kill_process(pid)
    }
}

/// Unix（macOS / Linux）实现：`ps` + `lsof` + `kill`。
#[cfg(not(target_os = "windows"))]
mod backend {
    use std::collections::HashMap;
    use std::process::{Command, Stdio};

    use super::{PortInfo, ProcessInfo};

    /// 解析 `lsof` 输出，得到 PID -> 监听端口列表。
    ///
    /// 输出形如：
    /// `COMMAND  PID USER  FD  TYPE DEVICE SIZE/OFF NODE NAME`，NAME 为 `127.0.0.1:8080`。
    pub fn fetch_listening_ports() -> HashMap<i32, Vec<PortInfo>> {
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
            let Some((address, port)) = super::split_endpoint(fields[8]) else {
                continue;
            };

            let info = PortInfo { address, port };
            let entry = result.entry(pid).or_default();
            if !entry.contains(&info) {
                entry.push(info);
            }
        }

        sort_ports(&mut result);
        result
    }

    /// 通过 `ps` 抓取进程列表。
    ///
    /// `limit` 为 0 表示不限制。只显示「有监听端口」或「PID > 100」的进程 ——
    /// 过滤掉没有端口且 PID 很小的内核进程。
    pub fn fetch_all_processes(
        port_map: &HashMap<i32, Vec<PortInfo>>,
        limit: usize,
    ) -> Vec<ProcessInfo> {
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

    /// 用 `kill -9` 结束进程。
    pub fn kill_process(pid: i32) -> bool {
        if pid <= 0 {
            return false;
        }

        let status = Command::new("/bin/kill")
            .arg("-9")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        matches!(status, Ok(status) if status.success())
    }

    /// 端口按端口号、地址排序，保证同一份数据每次展示顺序一致。
    fn sort_ports(result: &mut HashMap<i32, Vec<PortInfo>>) {
        for ports in result.values_mut() {
            ports.sort_by(|a, b| a.port.cmp(&b.port).then_with(|| a.address.cmp(&b.address)));
        }
    }
}

/// Windows 实现：`sysinfo` + `netstat` + `taskkill`。
#[cfg(target_os = "windows")]
mod backend {
    use std::collections::HashMap;
    use std::process::{Command, Stdio};

    use super::{PortInfo, ProcessInfo};

    /// 解析 `netstat -ano -p TCP` 输出，得到 PID -> 监听端口列表。
    ///
    /// 输出形如：
    /// `Proto  Local Address  Foreign Address  State  PID`，监听态的 State 列是
    /// `LISTENING`，Local Address 形如 `0.0.0.0:135` 或 `[::]:445`。
    pub fn fetch_listening_ports() -> HashMap<i32, Vec<PortInfo>> {
        let output = match Command::new("netstat")
            .args(["-ano", "-p", "TCP"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => output,
            Err(_) => return HashMap::new(),
        };

        let text = String::from_utf8_lossy(&output.stdout);
        let mut result: HashMap<i32, Vec<PortInfo>> = HashMap::new();

        for line in text.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            // 表头是 `Proto Local Address ...`，数据行至少 5 列。
            if fields.len() < 5 || !fields[0].eq_ignore_ascii_case("TCP") {
                continue;
            }
            if !fields[3].eq_ignore_ascii_case("LISTENING") {
                continue;
            }
            let Ok(pid) = fields[4].parse::<i32>() else {
                continue;
            };
            let Some((address, port)) = split_endpoint(fields[1]) else {
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

    /// 通过 `sysinfo` 抓取进程列表。
    ///
    /// `sysinfo` 直接给 PID / 名称 / 可执行路径，比解析 `tasklist` 稳
    /// （`tasklist` 只给名称，给不出完整路径）。
    pub fn fetch_all_processes(
        port_map: &HashMap<i32, Vec<PortInfo>>,
        limit: usize,
    ) -> Vec<ProcessInfo> {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

        let mut system = System::new();
        // 只要 PID / 名称 / 可执行路径：CPU 与内存不显示，刷新它们纯属浪费。
        // `exe` 的默认刷新策略是 `UpdateKind::Never`，不显式打开就永远拿不到完整路径。
        let kind = ProcessRefreshKind::default().with_exe(UpdateKind::OnlyIfNotSet);
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);

        // `sysinfo` 的内部是 HashMap，迭代顺序不确定 —— 先按 PID 排序再截断，
        // 否则同一个 limit 每次拿到的进程集合都不一样。
        let mut pids: Vec<i32> = system
            .processes()
            .keys()
            .map(|pid| pid.as_u32() as i32)
            .collect();
        pids.sort_unstable();

        let mut list: Vec<ProcessInfo> = Vec::new();
        for pid in pids {
            if limit > 0 && list.len() >= limit {
                break;
            }
            if pid <= 0 {
                continue;
            }

            let Some(process) = system.process(sysinfo::Pid::from_u32(pid as u32)) else {
                continue;
            };

            let display_name = process.name().to_string_lossy().to_string();
            // 受保护进程拿不到可执行路径，退回进程名。
            let full_path = process
                .exe()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| display_name.clone());

            let ports = port_map.get(&pid).cloned().unwrap_or_default();
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

    /// 用 `taskkill` 结束进程。
    pub fn kill_process(pid: i32) -> bool {
        if pid <= 0 {
            return false;
        }

        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        matches!(status, Ok(status) if status.success())
    }

    /// 把 Windows 的监听地址拆成地址与端口。
    ///
    /// `netstat` 的 Local Address 形如 `0.0.0.0:135`、`[::]:445`。
    /// 通配地址统一显示成 `*`，与 Unix 侧保持一致。
    pub(super) fn split_endpoint(endpoint: &str) -> Option<(String, u16)> {
        let (address, port) = endpoint.rsplit_once(':')?;
        let port = port.parse::<u16>().ok()?;
        if port == 0 {
            return None;
        }
        let address = address.trim_start_matches('[').trim_end_matches(']');
        let address = match address {
            "" | "0.0.0.0" | "::" => "*".to_string(),
            other => other.to_string(),
        };
        Some((address.to_string(), port))
    }
}

/// 把 `host:port` 拆成地址与端口，`*` 表示监听所有网卡。
#[cfg(not(target_os = "windows"))]
fn split_endpoint(endpoint: &str) -> Option<(String, u16)> {
    let (address, port) = endpoint.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    if port == 0 {
        return None;
    }
    let address = if address.is_empty() { "*" } else { address };
    Some((address.to_string(), port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn splits_endpoint() {
        assert_eq!(
            split_endpoint("127.0.0.1:8080"),
            Some(("127.0.0.1".to_string(), 8080))
        );
        assert_eq!(split_endpoint("*:443"), Some(("*".to_string(), 443)));
        assert_eq!(split_endpoint("noport"), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn splits_endpoint() {
        assert_eq!(
            backend::split_endpoint("127.0.0.1:8080"),
            Some(("127.0.0.1".to_string(), 8080))
        );
        assert_eq!(
            backend::split_endpoint("0.0.0.0:135"),
            Some(("*".to_string(), 135))
        );
        assert_eq!(
            backend::split_endpoint("[::]:445"),
            Some(("*".to_string(), 445))
        );
        assert_eq!(
            backend::split_endpoint("[::1]:8080"),
            Some(("::1".to_string(), 8080))
        );
        assert_eq!(backend::split_endpoint("noport"), None);
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

    #[test]
    fn collect_alone_does_not_fill_cache() {
        let mut manager = ProcessManager::default();
        let _ = manager.collect(4);
        assert!(
            manager.processes.is_empty(),
            "collect() 不得写缓存，缓存只由 load_blocking / collect_and_store 写入"
        );
    }

    /// `collect_and_store` 必须把采集结果写回缓存。
    ///
    /// 右键「结束进程」走的就是这条路径：它只调用 `collect` 时，
    /// 被杀掉的进程会继续留在缓存里，底栏进程总数与 `list_processes`
    /// 都停在杀之前的快照。
    #[test]
    fn collect_and_store_fills_cache() {
        let mut manager = ProcessManager::default();
        manager.collect_and_store(4);
        assert!(
            !manager.processes.is_empty(),
            "collect_and_store() 应把采集结果写回缓存"
        );
        assert!(manager.processes.len() <= 4);
    }

    #[test]
    fn kill_process_rejects_non_positive_pid() {
        assert!(!ProcessManager::kill_process(0));
        assert!(!ProcessManager::kill_process(-1));
    }
}
