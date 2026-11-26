import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const { popupContextMenu, deleteToTrash, menuActionHandlers } = vi.hoisted(() => ({
  popupContextMenu: vi.fn(),
  deleteToTrash: vi.fn(),
  menuActionHandlers: [] as Array<(payload: string) => void>,
}));

vi.mock('../lib/bridge', () => ({
  EVENT: { filesChanged: 'files-changed', menuAction: 'menu-action' },
  api: {
    listDir: vi.fn().mockResolvedValue([
      { path: '/tmp/a.txt', name: 'a.txt', isDir: false, size: 12, sizeText: '12 bytes', modified: 0 },
      { path: '/tmp/sub', name: 'sub', isDir: true, size: 0, sizeText: '-', modified: 0 },
    ]),
    watchDir: vi.fn().mockResolvedValue(undefined),
    homeDir: vi.fn().mockResolvedValue('/tmp'),
    pickDirectory: vi.fn().mockResolvedValue(null),
    parentDir: vi.fn().mockResolvedValue('/tmp'),
    revealInFinder: vi.fn().mockResolvedValue(undefined),
    renamePath: vi.fn().mockResolvedValue(undefined),
    deleteToTrash,
    createFolder: vi.fn().mockResolvedValue(undefined),
    createFile: vi.fn().mockResolvedValue(undefined),
    popupContextMenu,
  },
  onEvent: vi.fn((name: string, handler: (payload: unknown) => void) => {
    if (name === 'menu-action') {
      menuActionHandlers.push(handler as (payload: string) => void);
    }
    return Promise.resolve(() => {});
  }),
}));

vi.mock('../lib/store', async () => {
  const actual = await vi.importActual<typeof import('../lib/store')>('../lib/store');
  const state = {
    settings: null,
    strings: {
      'file.title': '文件管理',
      'file.column.name': '名称',
      'file.column.size': '大小',
      'file.column.modified': '修改时间',
      'file.column.action': '操作',
      'file.newFile': '新建文件',
      'file.newFolder': '新建文件夹',
      'file.openFolder': '进入文件夹',
      'file.rename': '重命名',
      'file.delete': '删除',
      'file.reveal': '在 Finder 中显示',
      'file.search.placeholder': '搜索文件',
      'common.back': '返回',
      'common.empty': '暂无数据',
    },
    activeTab: 'files',
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

import { Files } from './Files';

/** 触发一次原生右键菜单。 */
function rightClick(name: string): void {
  const cell = screen.getByText(name);
  const row = cell.closest('tr');
  if (!row) {
    throw new Error(`找不到 ${name} 所在行`);
  }
  fireEvent.contextMenu(row);
}

describe('Files', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    menuActionHandlers.length = 0;
  });

  it('渲染文件表头', async () => {
    render(<Files />);
    expect(await screen.findByText('名称')).toBeInTheDocument();
    expect(await screen.findByText('大小')).toBeInTheDocument();
  });

  it('显示新建文件夹与新建文件按钮', async () => {
    render(<Files />);
    expect(await screen.findByText('新建文件夹')).toBeInTheDocument();
    expect(await screen.findByText('新建文件')).toBeInTheDocument();
  });

  it('右键一行时弹出原生菜单并带上行数据', async () => {
    render(<Files />);
    await screen.findByText('a.txt');
    rightClick('a.txt');
    expect(popupContextMenu).toHaveBeenCalledWith('file', {
      path: '/tmp/a.txt',
      name: 'a.txt',
      isDir: false,
    });
  });

  it('收到 file:delete 事件时删除被右键的那一行', async () => {
    render(<Files />);
    await screen.findByText('a.txt');
    rightClick('a.txt');
    expect(menuActionHandlers.length).toBeGreaterThan(0);
    for (const handler of menuActionHandlers) {
      handler('file:delete');
    }
    await waitFor(() => expect(deleteToTrash).toHaveBeenCalledWith('/tmp/a.txt'));
  });

  it('收到 file:rename 事件时打开重命名弹窗', async () => {
    render(<Files />);
    await screen.findByText('a.txt');
    rightClick('a.txt');
    for (const handler of menuActionHandlers) {
      handler('file:rename');
    }
    expect(await screen.findByDisplayValue('a.txt')).toBeInTheDocument();
  });
});
