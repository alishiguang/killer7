import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../lib/bridge', () => ({
  EVENT: { clipboard: 'clipboard' },
  api: {
    clipboardCurrent: vi.fn().mockResolvedValue('hello chuaker7'),
    clipboardHistory: vi.fn().mockResolvedValue([
      { id: 1, content: '第一条历史', timestamp: 1700000000, kind: 'text' },
    ]),
    clearHistory: vi.fn().mockResolvedValue(undefined),
  },
  onEvent: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'clipboard.title': '剪贴板管理',
      'clipboard.current': '当前剪贴板',
      'clipboard.history': '历史记录',
      'clipboard.history.count': '共 %@ 条历史',
      'clipboard.search.placeholder': '搜索历史记录',
      'clipboard.clearHistory': '清空历史',
      'clipboard.copy': '复制',
      'clipboard.delete': '删除',
      'clipboard.empty': '（空）',
      'clipboard.copied': '已复制到剪贴板',
      'common.more': '更多',
    },
    activeTab: 'clipboard',
    toast: null,
    locale: 'zh-Hans',
    search: '',
  };
  return {
    ...actual,
    useAppState: () => state,
    useAppStore: () => [state, () => {}],
  };
});

import { Clipboard } from './Clipboard';

describe('Clipboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('展示当前剪贴板内容', async () => {
    render(<Clipboard />);
    expect(await screen.findByText('hello chuaker7')).toBeInTheDocument();
  });

  it('展示历史条目与计数', async () => {
    render(<Clipboard />);
    expect(await screen.findByText('第一条历史')).toBeInTheDocument();
    expect(await screen.findByText('共 1 条历史')).toBeInTheDocument();
  });

  it('渲染清空历史按钮', async () => {
    render(<Clipboard />);
    // 按钮改成图标后，文字只存在于 aria-label 里，按无障碍名称断言。
    expect(await screen.findByRole('button', { name: '清空历史' })).toBeInTheDocument();
  });
});
