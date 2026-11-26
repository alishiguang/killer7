import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: {},
  api: {
    updateSettings: vi.fn().mockResolvedValue({
      showMenuBar: false,
      autoRefreshInterval: 5,
      processListLimit: 500,
      enableNetworkMonitor: true,
      themeMode: 'dark',
      accent: 'signal',
      launchAtLogin: false,
      globalShortcutEnabled: true,
      locale: 'zh-Hans',
    }),
    setAutoLaunch: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: {
      showMenuBar: false,
      autoRefreshInterval: 5,
      processListLimit: 500,
      enableNetworkMonitor: true,
      themeMode: 'dark',
      accent: 'signal',
      launchAtLogin: false,
      globalShortcutEnabled: true,
      locale: 'zh-Hans',
    },
    strings: {
      'settings.title': '设置',
      'settings.appearance': '外观',
      'settings.performance': '性能',
      'settings.startup': '启动项',
      'settings.about': '关于',
      'settings.theme.mode': '主题模式',
      'settings.theme.system': '跟随系统',
      'settings.theme.light': '浅色模式',
      'settings.theme.dark': '深色模式',
      'settings.accent': '强调色',
      'settings.accent.signal': '信号红',
      'settings.accent.haze': '雾蓝',
      'settings.accent.moss': '苔绿',
      'settings.accent.amber': '琥珀',
      'settings.locale': '界面语言',
      'settings.refresh.interval': '自动刷新间隔',
      'settings.processLimit': '进程列表上限',
      'settings.noLimit': '无限制',
      'settings.networkMonitor': '网络监控',
      'settings.performance.hint': '降低刷新频率和限制可减少 CPU 占用',
      'settings.enableMenuBar': '启用菜单栏模式',
      'settings.menuBar.hint': '启用后，应用将在菜单栏显示图标',
      'settings.autoLaunch': '开机自启动',
      'settings.autoLaunch.hint': '启用后，应用将在登录时自动启动',
      'settings.globalShortcut': '全局快捷键',
      'settings.globalShortcut.hint': '启用后按 ⌘+L 可快速呼出应用',
      'settings.version': '版本',
      'settings.bundleId': 'Bundle ID',
      'settings.copyright': '版权',
      'common.back': '返回',
    },
    activeTab: 'processes',
    settingsOpen: true,
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Settings } from './Settings';

describe('Settings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染四张设置卡片', async () => {
    render(<Settings />);
    expect(await screen.findByText('外观')).toBeInTheDocument();
    expect(await screen.findByText('性能')).toBeInTheDocument();
    expect(await screen.findByText('启动项')).toBeInTheDocument();
    expect(await screen.findByText('关于')).toBeInTheDocument();
  });

  it('渲染主题分段控件的三个选项', async () => {
    render(<Settings />);
    expect(await screen.findByText('跟随系统')).toBeInTheDocument();
    expect(await screen.findByText('浅色模式')).toBeInTheDocument();
    expect(await screen.findByText('深色模式')).toBeInTheDocument();
  });

  it('渲染关于信息中的 bundle id', async () => {
    render(<Settings />);
    expect(await screen.findByText('com.igigi.chuaker7')).toBeInTheDocument();
  });

  it('页头带一个返回按钮', async () => {
    render(<Settings />);
    expect(await screen.findByRole('button', { name: '返回' })).toBeInTheDocument();
  });
});
