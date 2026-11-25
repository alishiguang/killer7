//! 文件管理命令。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, State};

use crate::commands::dto::FileRow;
use crate::core::platform;
use crate::events::EV_FILES_CHANGED;
use crate::state::AppState;

/// 列出指定目录的内容。
#[tauri::command]
pub fn list_dir(path: String) -> Vec<FileRow> {
    let dir = PathBuf::from(path);
    let mut manager = crate::core::files::FileManager {
        current_path: dir,
        files: Vec::new(),
        selected: None,
        is_loading: false,
    };
    manager.load_files();
    manager.files.iter().map(FileRow::from).collect()
}

/// 新建文件夹。
#[tauri::command]
pub fn create_folder(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    dir: String,
    name: String,
) -> Result<(), String> {
    let created = {
        let mut files = state.files.lock().expect("文件锁中毒");
        files.navigate_to(PathBuf::from(dir));
        files.create_folder(&name)
    };

    if created {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err(format!("创建文件夹「{name}」失败"))
    }
}

/// 新建空文件。
#[tauri::command]
pub fn create_file(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    dir: String,
    name: String,
) -> Result<(), String> {
    let created = {
        let mut files = state.files.lock().expect("文件锁中毒");
        files.navigate_to(PathBuf::from(dir));
        files.create_file(&name)
    };

    if created {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err(format!("创建文件「{name}」失败"))
    }
}

/// 重命名文件或文件夹。
#[tauri::command]
pub fn rename_path(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
    new_name: String,
) -> Result<(), String> {
    let renamed = {
        let mut files = state.files.lock().expect("文件锁中毒");
        files.rename_file(Path::new(&path), &new_name)
    };

    if renamed {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err(format!("重命名为「{new_name}」失败"))
    }
}

/// 移到废纸篓。
#[tauri::command]
pub fn delete_to_trash(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    path: String,
) -> Result<(), String> {
    let deleted = {
        let mut files = state.files.lock().expect("文件锁中毒");
        files.delete_file(Path::new(&path))
    };

    if deleted {
        let _ = app.emit(EV_FILES_CHANGED, ());
        Ok(())
    } else {
        Err("移到废纸篓失败".to_string())
    }
}

/// 在 Finder 中定位。
#[tauri::command]
pub fn reveal_in_finder(path: String) {
    platform::reveal_in_finder(Path::new(&path));
}

/// 返回上级目录，根目录返回 None。
#[tauri::command]
pub fn parent_dir(path: String) -> Option<String> {
    Path::new(&path)
        .parent()
        .filter(|parent| *parent != Path::new(&path))
        .map(|parent| parent.to_string_lossy().to_string())
}

/// 复制文本到系统剪贴板。
#[tauri::command]
pub fn copy_text(text: String) {
    platform::copy_to_clipboard(&text);
}

/// 用户主目录（文件页的初始目录）。
#[tauri::command]
pub fn home_dir() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/"))
        .to_string_lossy()
        .to_string()
}

/// 切换监听目录并重新挂载监听器。
#[tauri::command]
pub fn watch_dir(app: AppHandle, state: State<'_, Arc<AppState>>, path: String) {
    let target = PathBuf::from(path);

    let watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if let Ok(event) = result
            && crate::system::tasks::is_content_change(&event.kind)
        {
            let _ = app.emit(EV_FILES_CHANGED, ());
        }
    });

    let Ok(mut watcher) = watcher else {
        return;
    };
    if watcher.watch(&target, RecursiveMode::NonRecursive).is_err() {
        return;
    }

    *state.watcher.lock().expect("监听锁中毒") = Some(watcher);
}
