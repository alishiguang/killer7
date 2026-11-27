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
      locale: 'zh-hans',
    }),
    getStrings: vi.fn().mockResolvedValue({
      'nav.process': '进程管理',
      'nav.file': '文件管理',
      'nav.clipboard': '剪贴板',
      'settings.title': '设置',
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
      'settings.title': '设置',
      'app.brand': 'KI6',
      'app.tagline': 'MACOS 系统工具箱',
      'common.search': '搜索',
      'bottom.cpu': 'CPU',
      'bottom.memory': '内存',
      'bottom.disk': '磁盘',
      'bottom.cpuLoad': 'CPU 负载',
      'bottom.processes': '进程',
      'bottom.network': '网络',
    },
    activeTab: 'processes',
    settingsOpen: false,
    toast: null,
    locale: 'zh-hans',
    search: '',
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

  it('渲染品牌与三个标签页', async () => {
    render(<AppShell />);
    expect(await screen.findByText('KI6')).toBeInTheDocument();

    // 三个标签在工具栏的标签条里（活动监视器式布局，没有左侧栏）。
    // 设置不占标签位，它由工具栏角落的齿轮按钮进入；系统监控页已整体下线。
    const tabs = screen.getByRole('tablist');
    for (const label of ['进程管理', '文件管理', '剪贴板']) {
      expect(await within(tabs).findByRole('tab', { name: label })).toBeInTheDocument();
    }
    expect(within(tabs).queryByRole('tab', { name: '设置' })).not.toBeInTheDocument();
    expect(within(tabs).queryByRole('tab', { name: '系统监控' })).not.toBeInTheDocument();
  });

  it('搜索框在工具栏下面单独一行，右侧有设置按钮，底部有汇总栏', async () => {
    render(<AppShell />);
    expect(await screen.findByPlaceholderText('搜索')).toBeInTheDocument();

    // 设置入口是工具栏角落里的一个齿轮图标按钮。
    expect(await screen.findByRole('button', { name: '设置' })).toBeInTheDocument();

    // 底部汇总栏的三栏：CPU / 内存 / 磁盘 + CPU 负载 + 进程数 + 网络。
    expect(await screen.findByText('CPU 负载')).toBeInTheDocument();
    expect(await screen.findByText('内存')).toBeInTheDocument();
    expect(await screen.findByText('磁盘')).toBeInTheDocument();
  });
});
