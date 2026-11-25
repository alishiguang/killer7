import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: { systemInfo: 'system-info' },
  api: {
    systemSnapshot: vi.fn().mockResolvedValue({
      cpuUsage: 12.34,
      memoryUsage: 56.78,
      diskUsage: 90.12,
      networkUpload: 1.5,
      networkDownload: 2.25,
      cpuHistory: [1, 2, 3],
      memoryHistory: [4, 5, 6],
    }),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'monitor.title': '系统监控',
      'monitor.cpu': 'CPU使用率',
      'monitor.memory': '内存使用率',
      'monitor.disk': '磁盘使用率',
      'monitor.upload': '上传',
      'monitor.download': '下载',
      'monitor.cpu.history': 'CPU 采样',
      'monitor.memory.history': '内存采样',
    },
    activeTab: 'monitor',
    toast: null,
    locale: 'zh-Hans',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Monitor } from './Monitor';

describe('Monitor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染五个指标标签', async () => {
    render(<Monitor />);
    expect(await screen.findByText('CPU使用率')).toBeInTheDocument();
    expect(await screen.findByText('内存使用率')).toBeInTheDocument();
    expect(await screen.findByText('磁盘使用率')).toBeInTheDocument();
    expect(await screen.findByText('上传')).toBeInTheDocument();
    expect(await screen.findByText('下载')).toBeInTheDocument();
  });

  it('按固定小数位展示数值', async () => {
    render(<Monitor />);
    expect(await screen.findByText('12.3%')).toBeInTheDocument();
    expect(await screen.findByText('56.8%')).toBeInTheDocument();
    expect(await screen.findByText('1.50 MB/s')).toBeInTheDocument();
    expect(await screen.findByText('2.25 MB/s')).toBeInTheDocument();
  });
});
