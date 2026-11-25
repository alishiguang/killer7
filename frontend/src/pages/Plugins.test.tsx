import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: {},
  api: {
    listPlugins: vi.fn().mockResolvedValue([
      {
        id: 'demo',
        name: '示例插件',
        version: '1.0.0',
        author: 'Igigi',
        description: null,
        minimumAppVersion: null,
        permissions: ['fs'],
        main: null,
        dir: '/tmp/plugins',
      },
    ]),
    reloadPlugins: vi.fn().mockResolvedValue([]),
    pluginsDir: vi.fn().mockResolvedValue('/tmp/plugins'),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'nav.plugins': '插件',
      'settings.plugin.dir': '插件目录',
      'settings.plugin.empty': '尚未安装插件',
      'plugin.manifest': '清单信息',
      'plugin.version': '版本',
      'plugin.author': '作者',
      'plugin.permissions': '权限',
      'plugin.minVersion': '最低应用版本',
      'process.refresh': '刷新',
    },
    activeTab: 'plugins',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Plugins } from './Plugins';

describe('Plugins', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('展示插件目录与清单信息', async () => {
    render(<Plugins />);
    expect(await screen.findByText('/tmp/plugins')).toBeInTheDocument();
    expect(await screen.findByText('示例插件')).toBeInTheDocument();
    expect(await screen.findByText('1.0.0')).toBeInTheDocument();
  });

  it('没有插件时展示空态', async () => {
    const { api } = await import('../lib/bridge');
    vi.mocked(api.listPlugins).mockResolvedValueOnce([]);
    render(<Plugins />);
    expect(await screen.findByText('尚未安装插件')).toBeInTheDocument();
  });
});
