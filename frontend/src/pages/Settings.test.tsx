import { fireEvent, render, screen, waitFor } from '@testing-library/react';
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
      locale: 'zh-hans',
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
      locale: 'zh-hans',
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
      'settings.menuBar.hint': '启用后应用不再占用 Dock 图标位置，只在菜单栏常驻',
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
    locale: 'zh-hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Settings } from './Settings';
import { api } from '../lib/bridge';

const updateSettings = api.updateSettings as unknown as ReturnType<typeof vi.fn>;
const setAutoLaunch = api.setAutoLaunch as unknown as ReturnType<typeof vi.fn>;

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

  /// 启动项卡片的三个开关必须各自接到正确的后端通道：
  /// 菜单栏模式与全局快捷键走设置补丁，开机自启先落 LaunchAgent 再写设置。
  /// 这三条链路一旦接错，开关会「看着变了、实际没生效」。
  it('启动项三个开关各自提交正确的设置补丁', async () => {
    render(<Settings />);

    await fireEvent.click(await screen.findByRole('checkbox', { name: '启用菜单栏模式' }));
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith({ showMenuBar: true }),
    );

    await fireEvent.click(await screen.findByRole('checkbox', { name: '全局快捷键' }));
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith({ globalShortcutEnabled: false }),
    );
  });

  it('开机自启先落 LaunchAgent，成功后才写设置', async () => {
    render(<Settings />);

    await fireEvent.click(await screen.findByRole('checkbox', { name: '开机自启动' }));
    await waitFor(() => expect(setAutoLaunch).toHaveBeenCalledWith(true));
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith({ launchAtLogin: true }),
    );
  });

  /// 自启落盘失败时不能把开关留在「已开启」的状态。
  it('开机自启失败时不再提交设置补丁', async () => {
    setAutoLaunch.mockRejectedValueOnce(new Error('写入 LaunchAgent 失败'));
    render(<Settings />);

    await fireEvent.click(await screen.findByRole('checkbox', { name: '开机自启动' }));
    await waitFor(() => expect(setAutoLaunch).toHaveBeenCalledWith(true));
    expect(updateSettings).not.toHaveBeenCalled();
  });

  /// 当前语言必须在分段控件里显示为选中。
  /// 前端曾用 `zh-Hans`、Rust 侧是 `zh-hans`，两边对不上：
  /// 界面上永远看不到「简体中文」被选中，点它还会被 serde 拒收。
  it('当前语言在分段控件里显示为选中', async () => {
    render(<Settings />);
    expect(await screen.findByRole('button', { name: '简体中文' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('语言切换提交 Rust 侧规范写法 zh-hans', async () => {
    render(<Settings />);
    await fireEvent.click(await screen.findByRole('button', { name: '简体中文' }));
    await waitFor(() =>
      expect(updateSettings).toHaveBeenCalledWith({ locale: 'zh-hans' }),
    );
  });
});
