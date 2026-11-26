//! Tauri command 门面。
//!
//! 这一层只做「编解码 + 转发」：参数来自前端，调用 core 或 system，
//! 返回可序列化的结果。业务逻辑不写在这里。

pub mod clipboard;
pub mod dto;
pub mod files;
pub mod monitor;
pub mod plugins;
pub mod processes;
pub mod settings;
pub mod system;

/// 汇总所有 command。
pub fn handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        processes::list_processes,
        processes::refresh_processes,
        processes::kill_process,
        files::list_dir,
        files::create_folder,
        files::create_file,
        files::rename_path,
        files::delete_to_trash,
        files::reveal_in_finder,
        files::parent_dir,
        files::copy_text,
        files::home_dir,
        files::watch_dir,
        clipboard::clipboard_current,
        clipboard::clipboard_history,
        clipboard::remove_history_item,
        clipboard::clear_history,
        monitor::system_snapshot,
        plugins::list_plugins,
        plugins::reload_plugins,
        plugins::plugins_dir,
        settings::get_settings,
        settings::update_settings,
        settings::get_strings,
        system::hide_main_window,
        system::show_main_window,
        system::set_menu_bar_mode,
        system::set_shortcut_enabled,
        system::set_auto_launch,
        system::popup_context_menu,
    ]
}
