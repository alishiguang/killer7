import { render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: { processes: 'processes' },
  api: {
    listProcesses: vi.fn().mockResolvedValue([
      { pid: 12, displayName: 'Finder', fullPath: '/System/Finder', ports: [] },
      { pid: 340, displayName: 'node', fullPath: '/usr/local/bin/node', ports: [{ address: '*', port: 8080 }] },
    ]),
    refreshProcesses: vi.fn().mockResolvedValue([]),
    killProcess: vi.fn().mockResolvedValue(true),
    popupContextMenu: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'process.title': '进程管理',
      'process.pid': 'PID',
      'process.name': '进程名称',
      'process.port': '监听端口',
      'process.refresh': '刷新',
      'process.search.placeholder': '搜索进程名称、PID 或端口号',
      'process.count': '共 %@ 个进程',
      'process.empty': '没有匹配的进程',
      'process.kill.title': '确认结束进程',
      'process.kill.confirm': '确定要结束进程 %@ (PID: %@) 吗？',
      'process.kill.done': '已结束进程 %@',
      'common.cancel': '取消',
      'common.confirm': '确认',
    },
    activeTab: 'processes',
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

import { Processes } from './Processes';

describe('Processes', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('渲染进程表头与行', async () => {
    const { container } = render(<Processes />);
    expect(await screen.findByText('Finder')).toBeInTheDocument();
    expect(await screen.findByText('*:8080')).toBeInTheDocument();

    // 「PID」同时出现在排序分段控件与虚拟表表头里，
    // findByText('PID') 会命中多个元素，必须限定在表头行内断言。
    const head = container.querySelector('.vrow--head');
    expect(head).not.toBeNull();
    expect(within(head as HTMLElement).getByText('PID')).toBeInTheDocument();
  });

  it('显示进程总数', async () => {
    render(<Processes />);
    expect(await screen.findByText('共 2 个进程')).toBeInTheDocument();
  });
});
