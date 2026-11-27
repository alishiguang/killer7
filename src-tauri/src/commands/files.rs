//! 文件管理命令。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::commands::dto::FileRow;
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

/// 返回上级目录，根目录返回 None。
#[tauri::command]
pub fn parent_dir(path: String) -> Option<String> {
    Path::new(&path)
        .parent()
        .filter(|parent| *parent != Path::new(&path))
        // 相对路径的 parent 是空串，空串不是有效目录，要一并挡掉。
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_string_lossy().to_string())
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
    // 监听器的创建与挂载统一在 system::tasks::watch_path 里，
    // 失败原因由它打到 stderr —— 静默失败会让前端以为已经切到新目录。
    crate::system::tasks::watch_path(&app, state.inner(), &target);
}
