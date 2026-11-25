import { render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: {
    processes: 'processes',
    systemInfo: 'system-info',
    clipboard: 'clipboard',
    filesChanged: 'files-changed',
    trayOpen: 'tray-open',
    trayQuit: 'tray-quit',
    notice: 'notice',
    menuAction: 'menu-action',
  },
  api: {
    getSettings: vi.fn().mockResolvedValue({
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
    getStrings: vi.fn().mockResolvedValue({
      'nav.process': '进程管理',
      'nav.file': '文件管理',
      'nav.clipboard': '剪贴板',
      'nav.monitor': '系统监控',
      'nav.plugins': '插件',
      'nav.settings': '设置',
      'nav.section.tools': '工具',
      'nav.section.system': '系统',
      'app.brand': 'KI6',
      'app.tagline': 'MACOS 系统工具箱',
    }),
    // 默认激活的是进程页，AppShell 会把它挂载起来，
    // 页面在 useEffect 里调用的每个 api 方法都必须有桩，否则测试会抛
    // "api.xxx is not a function"。
    listProcesses: vi.fn().mockResolvedValue([]),
    refreshProcesses: vi.fn().mockResolvedValue([]),
    killProcess: vi.fn().mockResolvedValue(true),
    popupContextMenu: vi.fn().mockResolvedValue(undefined),
    listDir: vi.fn().mockResolvedValue([]),
    homeDir: vi.fn().mockResolvedValue('/tmp'),
    watchDir: vi.fn().mockResolvedValue(undefined),
    pickDirectory: vi.fn().mockResolvedValue(null),
    clipboardCurrent: vi.fn().mockResolvedValue(''),
    clipboardHistory: vi.fn().mockResolvedValue([]),
    systemSnapshot: vi.fn().mockResolvedValue({
      cpuUsage: 0,
      memoryUsage: 0,
      diskUsage: 0,
      networkUpload: 0,
      networkDownload: 0,
      cpuHistory: [],
      memoryHistory: [],
    }),
    listPlugins: vi.fn().mockResolvedValue([]),
    pluginsDir: vi.fn().mockResolvedValue('/tmp/plugins'),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'nav.process': '进程管理',
      'nav.file': '文件管理',
      'nav.clipboard': '剪贴板',
      'nav.monitor': '系统监控',
      'nav.plugins': '插件',
      'nav.settings': '设置',
      'nav.section.tools': '工具',
      'nav.section.system': '系统',
      'app.brand': 'KI6',
      'app.tagline': 'MACOS 系统工具箱',
    },
    activeTab: 'processes',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { AppShell } from './AppShell';

describe('AppShell', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染品牌与六个标签页', async () => {
    render(<AppShell />);
    expect(await screen.findByText('KI6')).toBeInTheDocument();

    // 当前激活页的标题（Kicker）与侧栏导航项同名，
    // findByText 会命中多个元素，必须限定在侧栏导航区内断言。
    const nav = screen.getByRole('navigation');
    for (const label of ['进程管理', '文件管理', '剪贴板', '系统监控', '插件', '设置']) {
      expect(await within(nav).findByText(label)).toBeInTheDocument();
    }
  });
});
