//! 单实例保护。
//!
//! ## 为什么需要
//!
//! 开机自启走 LaunchAgent：plist 的 `ProgramArguments` 直接指向 `.app` 包内的二进制，
//! 由 launchd 在登录时拉起 —— **这条路径不经过 LaunchServices**，所以 LaunchServices
//! 并不知道这个进程的存在。之后用户从 Finder 双击同一个 `Ki6.app`，系统会**再起一个
//! 进程**：两个托盘图标、两套菜单栏项、两份后台采样线程，还可能并发写同一个
//! `settings.json`（后者是真正会损坏数据的那种）。
//!
//! ## 做法
//!
//! 分两层：
//!
//! 1. **文件锁**做原子仲裁（`File::try_lock`）。进程正常退出或崩溃时，操作系统都会
//!    释放锁，不会留下需要人工清理的死锁文件。这也是不用 `tauri-plugin-single-instance`
//!    的原因 —— 它不在本项目的依赖白名单里。
//! 2. **唤起请求**把「再点一次图标」的意图转给已在跑的那个实例：后启动的实例往
//!    唤起目录里丢一个请求文件，首个实例用 `notify` 盯着这个目录，收到请求就把主窗口
//!    显示出来。这一步用文件而不是 AppKit，是因为「仅菜单栏模式」下主窗口本来就是
//!    隐藏的，只有先启动的实例自己知道该怎么把它叫出来。
//!
//! ## 身份键
//!
//! 用**可执行文件的规范化路径**做身份键：打包态是
//! `/Applications/Ki6.app/Contents/MacOS/chuaker7`，开发态是
//! `target/release/chuaker7`。两者天然分开 —— 开发时跑二进制不会把正在运行的正式版
//! 顶掉。也因此不需要读 `Info.plist` 拿 bundle id（`NSBundle` 会把
//! `objc2-foundation` 拖成直接依赖，不在白名单里）。

use std::fs::{File, OpenOptions, TryLockError};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use notify::{EventKind, RecursiveMode, Watcher};
use tauri::{AppHandle, Runtime};

use crate::core::settings::data_dir;

/// 持锁文件 —— 进程活着就一直持有，退出时由操作系统释放。
///
/// 放进 `static` 是必须的：`File` 一析构，fd 就关了，锁随之释放。
static HELD_LOCK: Mutex<Option<File>> = Mutex::new(None);

/// 单实例仲裁结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arbitration {
    /// 本进程是唯一实例，正常启动。
    Primary,
    /// 已有实例在运行：本进程应当退出。
    Secondary,
}

/// 当前实例的身份键（可执行路径的 FNV-1a 哈希）。
///
/// 取哈希而不是直接用路径做文件名：路径里有 `/` 与空格，转义成文件名很难看。
fn identity_key() -> String {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("unknown"));
    format!("{:016x}", fnv1a(exe.to_string_lossy().as_bytes()))
}

/// FNV-1a 64 位哈希（只为把路径压成短文件名，不需要抗碰撞强度）。
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// 锁文件路径。
fn lock_path() -> PathBuf {
    data_dir().join(format!("single-instance-{}.lock", identity_key()))
}

/// 唤起请求目录。
///
/// 只放瞬时请求文件，首个实例消费后立即删除 —— 目录里长期为空，
/// 监听器不会因为别的事件被频繁唤醒。
fn activation_dir() -> PathBuf {
    data_dir().join(format!("single-instance-{}.activate", identity_key()))
}

/// 尝试抢占单实例所有权。
///
/// 返回 `Secondary` 表示**已经有实例在跑**，调用方应当立刻退出。
/// 拿不到锁文件、或加锁本身出错时一律放行（`Primary`）——
/// 单实例是锦上添花的保护，不该因为文件系统出怪问题就让用户打不开应用。
pub fn acquire() -> Arbitration {
    let path = lock_path();
    if let Some(parent) = path.parent()
        && let Err(err) = std::fs::create_dir_all(parent)
    {
        eprintln!("[single-instance] 创建数据目录失败，放行启动: {err}");
        return Arbitration::Primary;
    }

    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
    {
        Ok(file) => file,
        Err(err) => {
            eprintln!("[single-instance] 打开锁文件失败，放行启动: {err}");
            return Arbitration::Primary;
        }
    };

    match file.try_lock() {
        Ok(()) => {
            *HELD_LOCK.lock().unwrap_or_else(|err| err.into_inner()) = Some(file);
            Arbitration::Primary
        }
        Err(TryLockError::WouldBlock) => {
            // 锁被别的进程占着 —— 那就是已经有一个实例在跑了。
            // 这里必须让 `file` 析构（fd 关闭），否则本进程也会持有一份锁。
            Arbitration::Secondary
        }
        Err(TryLockError::Error(err)) => {
            eprintln!("[single-instance] 加锁失败，放行启动: {err}");
            Arbitration::Primary
        }
    }
}

/// 通知已在运行的实例把主窗口叫出来。
///
/// 写一个带唯一名字的请求文件即可：首个实例按「目录里有新文件」消费，
/// 不需要读文件内容，也就不存在两个后启动实例互相覆盖请求的问题。
pub fn request_activation() {
    let dir = activation_dir();
    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!("[single-instance] 创建唤起目录失败: {err}");
        return;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file = dir.join(format!("{}-{stamp}.request", std::process::id()));
    if let Err(err) = std::fs::write(&file, b"activate") {
        eprintln!("[single-instance] 写入唤起请求失败: {err}");
    }
}

/// 启动唤起请求监听（只由首个实例执行）。
///
/// 收到请求后清空目录并把主窗口显示出来 —— 与点击托盘图标的效果一致。
pub fn spawn_activation_watcher<R: Runtime>(app: AppHandle<R>) {
    let dir = activation_dir();
    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!("[single-instance] 创建唤起目录失败: {err}");
        return;
    }

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
            let _ = tx.send(result);
        });
        let Ok(mut watcher) = watcher else {
            eprintln!("[single-instance] 无法创建唤起监听器");
            return;
        };
        if let Err(err) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
            eprintln!("[single-instance] 监听 {} 失败: {err}", dir.display());
            return;
        }

        while let Ok(result) = rx.recv() {
            let Ok(event) = result else { continue };
            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                continue;
            }
            // 先清目录再唤起：请求是一次性的，残留文件会让下一次启动白唤起一次。
            clear_requests(&dir);
            crate::system::tray::show_main_window(&app);
        }
    });
}

/// 清空唤起请求目录。
fn clear_requests(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let _ = std::fs::remove_file(entry.path());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_is_stable_and_distinguishes_inputs() {
        assert_eq!(fnv1a(b"abc"), fnv1a(b"abc"));
        assert_ne!(fnv1a(b"abc"), fnv1a(b"abd"));
        // FNV-1a 64 位的标准测试向量。
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"abc"), 0xe71f_a219_0541_574b);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn identity_key_is_hex_and_stable() {
        let key = identity_key();
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(key, identity_key());
    }

    #[test]
    fn lock_and_activation_paths_share_the_same_key() {
        let lock = lock_path();
        let activation = activation_dir();
        let key = identity_key();
        assert!(lock.to_string_lossy().contains(&key));
        assert!(activation.to_string_lossy().contains(&key));
        assert_ne!(lock, activation);
    }

    /// 同一进程内对同一把锁连续加锁：第一次成功，**第二次必须失败**。
    ///
    /// 这是单实例保护的核心不变量 —— 锁不住就说明「已经有别人了」。
    /// 这里用两个独立的 `File` 句柄模拟两个进程争抢同一把锁。
    #[test]
    fn second_lock_attempt_is_refused() {
        let path = std::env::temp_dir().join(format!("ki6-lock-test-{}.lock", std::process::id()));
        let first = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .expect("打开锁文件");
        first.try_lock().expect("第一次加锁应成功");

        let second = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .expect("打开锁文件");
        let err = second
            .try_lock()
            .expect_err("同一把锁上的第二次加锁必须失败");
        assert!(
            matches!(err, TryLockError::WouldBlock),
            "应当是「锁被占着」，而不是别的 I/O 错误"
        );

        // 清理：文件本身可以留下，锁已经随句柄释放。
        let _ = std::fs::remove_file(&path);
    }

    /// 唤起请求必须真的落到磁盘上，且能被监听方看到。
    #[test]
    fn request_activation_writes_a_file() {
        let dir = activation_dir();
        // 先清空：上一次运行若被中断，可能留下残留文件，断言「恰好一个」会误判。
        clear_requests(&dir);
        request_activation();
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .expect("读取唤起目录")
            .flatten()
            .map(|e| e.path())
            .collect();
        assert_eq!(entries.len(), 1, "应当恰好写入一个请求文件");
        assert!(entries[0].extension().is_some_and(|ext| ext == "request"));

        clear_requests(&dir);
        let left = std::fs::read_dir(&dir).expect("读取唤起目录").count();
        assert_eq!(left, 0, "消费后目录应当被清空");
    }
}
