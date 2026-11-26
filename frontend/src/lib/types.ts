// 与 src-tauri/src/commands/dto.rs 一一对应。

export interface PortRow {
  address: string;
  port: number;
}

export interface ProcessRow {
  pid: number;
  displayName: string;
  fullPath: string;
  ports: PortRow[];
}

export interface FileRow {
  path: string;
  name: string;
  isDir: boolean;
  size: number;
  sizeText: string;
  modified: number;
}

export interface HistoryRow {
  id: number;
  content: string;
  timestamp: number;
  kind: string;
}

export interface SystemInfoDto {
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
  networkUpload: number;
  networkDownload: number;
  cpuHistory: number[];
  memoryHistory: number[];
  /** 当前进程总数（取自进程缓存）。 */
  processCount: number;
}

export interface PluginRow {
  id: string;
  name: string;
  version: string;
  author: string | null;
  description: string | null;
  minimumAppVersion: string | null;
  permissions: string[];
  main: string | null;
  dir: string;
}

export type ThemeMode = 'system' | 'light' | 'dark';
export type Accent = 'signal' | 'haze' | 'moss' | 'amber';
export type Locale = 'zh-Hans' | 'en';

export interface Settings {
  showMenuBar: boolean;
  autoRefreshInterval: number;
  processListLimit: number;
  enableNetworkMonitor: boolean;
  themeMode: ThemeMode;
  accent: Accent;
  launchAtLogin: boolean;
  globalShortcutEnabled: boolean;
  locale: Locale;
}

export type SettingsPatch = Partial<Settings>;

export interface Notice {
  kind: 'info' | 'error';
  message: string;
}

export type TabId = 'processes' | 'files' | 'clipboard' | 'monitor' | 'plugins' | 'settings';
