// Tauri 桥接层：所有 invoke / listen 收敛在这里，页面不直接接触 @tauri-apps/api。

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

import type {
  FileRow,
  HistoryRow,
  ProcessRow,
  Settings,
  SettingsPatch,
  SystemInfoDto,
} from './types';

/** 事件名常量，与 src-tauri/src/events.rs 一一对应。 */
export const EVENT = {
  processes: 'processes',
  systemInfo: 'system-info',
  clipboard: 'clipboard',
  filesChanged: 'files-changed',
  trayOpen: 'tray-open',
  trayQuit: 'tray-quit',
  notice: 'notice',
  menuAction: 'menu-action',
} as const;

/** 全部 command 的薄封装。 */
export const api = {
  listProcesses: () => invoke<ProcessRow[]>('list_processes'),
  refreshProcesses: () => invoke<ProcessRow[]>('refresh_processes'),
  killProcess: (pid: number) => invoke<boolean>('kill_process', { pid }),

  listDir: (path: string) => invoke<FileRow[]>('list_dir', { path }),
  createFolder: (dir: string, name: string) => invoke<void>('create_folder', { dir, name }),
  createFile: (dir: string, name: string) => invoke<void>('create_file', { dir, name }),
  renamePath: (path: string, newName: string) =>
    invoke<void>('rename_path', { path, newName }),
  deleteToTrash: (path: string) => invoke<void>('delete_to_trash', { path }),
  parentDir: (path: string) => invoke<string | null>('parent_dir', { path }),
  homeDir: () => invoke<string>('home_dir'),
  watchDir: (path: string) => invoke<void>('watch_dir', { path }),

  clipboardCurrent: () => invoke<string>('clipboard_current'),
  clipboardHistory: () => invoke<HistoryRow[]>('clipboard_history'),
  clearHistory: () => invoke<void>('clear_history'),

  systemSnapshot: () => invoke<SystemInfoDto>('system_snapshot'),

  getSettings: () => invoke<Settings>('get_settings'),
  updateSettings: (patch: SettingsPatch) => invoke<Settings>('update_settings', { patch }),
  getStrings: (locale: string) => invoke<Record<string, string>>('get_strings', { locale }),

  hideMainWindow: () => invoke<void>('hide_main_window'),
  showMainWindow: () => invoke<void>('show_main_window'),
  setMenuBarMode: (enabled: boolean) => invoke<void>('set_menu_bar_mode', { enabled }),
  setShortcutEnabled: (enabled: boolean) => invoke<void>('set_shortcut_enabled', { enabled }),
  setAutoLaunch: (enabled: boolean) => invoke<void>('set_auto_launch', { enabled }),
  popupContextMenu: (kind: string, payload: unknown) =>
    invoke<void>('popup_context_menu', { kind, payload }),

  /** 弹出 macOS 原生目录选择框，取消时返回 null。 */
  pickDirectory: async (): Promise<string | null> => {
    const selected = await open({ directory: true, multiple: false });
    return typeof selected === 'string' ? selected : null;
  },
};

/** 订阅一个后端事件，返回取消订阅函数。 */
export function onEvent<T>(name: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(name, (event) => handler(event.payload));
}
