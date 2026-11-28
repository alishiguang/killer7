import { useCallback, useDeferredValue, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Icon, Kicker, Modal, type Column } from '../components';
import { PageSearch } from '../app/PageSearch';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { nextSort, type SortState } from '../lib/sort';
import { pushToast, useAppStore } from '../lib/store';
import type { FileRow } from '../lib/types';

export function Files() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const [cwd, setCwd] = useState<string>('');
  const [rows, setRows] = useState<FileRow[]>([]);
  const [renaming, setRenaming] = useState<FileRow | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [creating, setCreating] = useState<'folder' | 'file' | null>(null);
  const [createValue, setCreateValue] = useState('');
  /** 原生右键菜单当前对应的行（「重命名 / 删除」需要它才能继续）。 */
  const [contextRow, setContextRow] = useState<FileRow | null>(null);
  /** 排序：默认按名称升序，点表头切换方向。 */
  const [sort, setSort] = useState<SortState>({ key: 'name', dir: 'asc' });
  /** 当前选中的行（单击选中、双击/右键作用于它）。 */
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  // 搜索框在工具栏上（与活动监视器一致），搜索词是外壳级的共享状态。
  const search = state.search;

  const load = useCallback(async (path: string) => {
    const list = await api.listDir(path);
    setRows(list);
  }, []);

  // 初始目录：用户主目录。
  useEffect(() => {
    let disposed = false;
    void (async () => {
      const target = await api.homeDir();
      if (disposed || !target) {
        return;
      }
      setCwd(target);
      await load(target);
      await api.watchDir(target);
    })();
    return () => {
      disposed = true;
    };
  }, [load]);

  useEffect(() => {
    void onEvent(EVENT.filesChanged, () => {
      if (cwd) {
        void load(cwd);
      }
    });
  }, [cwd, load]);

  // 搜索词走 useDeferredValue：打字时先更新输入框，筛选结果滞后一拍。
  const deferredSearch = useDeferredValue(search);

  const filtered = useMemo(() => {
    const keyword = deferredSearch.trim().toLowerCase();
    if (!keyword) {
      return rows;
    }
    return rows.filter((row) => row.name.toLowerCase().includes(keyword));
  }, [rows, deferredSearch]);

  const columns = useMemo<Column<FileRow>[]>(
    () => [
      {
        key: 'name',
        title: t('file.column.name'),
        width: 'remainder',
        sortValue: (row) => row.name,
        render: (row) => (
          <span className="file-name">
            <span className="file-name__glyph" aria-hidden="true">
              {row.isDir ? '◆' : '·'}
            </span>
            {row.name}
          </span>
        ),
      },
      {
        key: 'size',
        title: t('file.column.size'),
        width: 100,
        align: 'right',
        sortValue: (row) => row.size,
        render: (row) => <span className="mono">{row.sizeText}</span>,
      },
      {
        key: 'modified',
        title: t('file.column.modified'),
        width: 130,
        sortValue: (row) => row.modified,
        render: (row) => <span className="mono">{formatTime(row.modified)}</span>,
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [t, cwd],
  );

  /** 进入指定目录：切换 cwd、加载内容、重新挂监听。 */
  const openDir = useCallback(
    async (path: string) => {
      setCwd(path);
      // 换目录后旧目录里的选中行不再存在，选中态要清掉。
      setSelectedPath(null);
      await load(path);
      await api.watchDir(path);
    },
    [load],
  );

  // 行点击与行右键的回调保持稳定引用，否则 DataTable 里的 memo 行组件
  // 每次渲染都会因为 props 变了而全部重绘。
  //
  // 单击只选中：选中态要和右键菜单的目标行是同一行，否则会出现
  // 「高亮 A、菜单却操作 B」。
  const handleRowClick = useCallback((row: FileRow) => {
    setSelectedPath(row.path);
  }, []);

  /** 双击：主操作 —— 目录进入，文件用系统默认程序打开。 */
  const handleRowDoubleClick = useCallback(
    async (row: FileRow) => {
      if (row.isDir) {
        await openDir(row.path);
        return;
      }
      try {
        await api.openPath(row.path);
      } catch (error) {
        pushToast('error', String(error));
      }
    },
    [openDir],
  );

  const handleRowContextMenu = useCallback((row: FileRow) => {
    setSelectedPath(row.path);
    setContextRow(row);
    void api.popupContextMenu('file', {
      path: row.path,
      name: row.name,
      isDir: row.isDir,
    });
  }, []);

  const goUp = useCallback(async () => {
    const parent = await api.parentDir(cwd);
    if (!parent) {
      return;
    }
    await openDir(parent);
  }, [cwd, openDir]);

  const remove = useCallback(
    async (row: FileRow) => {
      try {
        await api.deleteToTrash(row.path);
        pushToast('info', t('file.deleted'));
        await load(cwd);
      } catch (error) {
        pushToast('error', String(error));
      }
    },
    [cwd, load, t],
  );

  // 原生右键菜单里的「重命名 / 删除」由 Rust 通过 menu-action 事件回传，
  // 被操作的目标行在 onRowContextMenu 里记下，这里补齐后续动作。
  // 放在 remove 之后声明，避免闭包捕获到旧的 remove。
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void onEvent<string>(EVENT.menuAction, (action) => {
      // 右键菜单里的「进入文件夹」：路径直接带在事件里。
      if (action.startsWith('file:open:')) {
        void openDir(action.slice('file:open:'.length));
        return;
      }
      if (!contextRow) {
        return;
      }
      if (action === 'file:rename') {
        setRenaming(contextRow);
        setRenameValue(contextRow.name);
      } else if (action === 'file:delete') {
        void remove(contextRow);
      }
    }).then((fn) => {
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [contextRow, openDir, remove]);

  const submitRename = useCallback(async () => {
    if (!renaming || !renameValue.trim()) {
      return;
    }
    try {
      await api.renamePath(renaming.path, renameValue.trim());
      pushToast('info', t('file.renamed', [renameValue.trim()]));
      setRenaming(null);
      await load(cwd);
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [cwd, load, renameValue, renaming, t]);

  const submitCreate = useCallback(async () => {
    if (!creating || !createValue.trim()) {
      return;
    }
    try {
      if (creating === 'folder') {
        await api.createFolder(cwd, createValue.trim());
      } else {
        await api.createFile(cwd, createValue.trim());
      }
      pushToast('info', t('file.created', [createValue.trim()]));
      setCreating(null);
      setCreateValue('');
      await load(cwd);
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [createValue, creating, cwd, load, t]);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="02">{t('file.title')}</Kicker>
        <div className="page__head-actions">
          <PageSearch />
          <div className="page__tools">
            <button
              type="button"
              className="native-button native-button--icon"
              title={t('file.openFolder')}
              aria-label={t('file.openFolder')}
              onClick={async () => {
                const picked = await api.pickDirectory();
                if (!picked) {
                  return;
                }
                await openDir(picked);
              }}
            >
              <Icon name="folder" />
            </button>
            <button
              type="button"
              className="native-button native-button--icon"
              title={t('common.back')}
              aria-label={t('common.back')}
              onClick={() => void goUp()}
            >
              <Icon name="arrow-up" />
            </button>
            <button
              type="button"
              className="native-button native-button--icon"
              title={t('file.newFile')}
              aria-label={t('file.newFile')}
              onClick={() => {
                setCreating('file');
                setCreateValue('');
              }}
            >
              <Icon name="file-plus" />
            </button>
            <button
              type="button"
              className="native-button native-button--icon"
              title={t('file.newFolder')}
              aria-label={t('file.newFolder')}
              onClick={() => {
                setCreating('folder');
                setCreateValue('');
              }}
            >
              <Icon name="folder-plus" />
            </button>
          </div>
        </div>
      </div>

      <Card title={cwd} className="page__card">
        <DataTable
          columns={columns}
          items={filtered}
          getKey={(row) => row.path}
          selectedKey={selectedPath}
          onRowClick={handleRowClick}
          onRowDoubleClick={handleRowDoubleClick}
          onRowContextMenu={handleRowContextMenu}
          empty={t('common.empty')}
          sortKey={sort.key}
          sortDir={sort.dir}
          onSort={(key) => setSort((prev) => nextSort(prev, key))}
        />
      </Card>

      <Modal
        open={renaming !== null}
        title={t('file.rename.title')}
        onClose={() => setRenaming(null)}
        footer={
          <>
            <button type="button" className="native-button" onClick={() => setRenaming(null)}>
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="native-button native-button--accent"
              onClick={() => void submitRename()}
            >
              {t('common.save')}
            </button>
          </>
        }
      >
        <input
          className="native-input"
          value={renameValue}
          placeholder={t('file.rename.hint')}
          onChange={(event) => setRenameValue(event.target.value)}
        />
      </Modal>

      <Modal
        open={creating !== null}
        title={creating === 'folder' ? t('file.newFolder.title') : t('file.newFile')}
        onClose={() => setCreating(null)}
        footer={
          <>
            <button type="button" className="native-button" onClick={() => setCreating(null)}>
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="native-button native-button--accent"
              onClick={() => void submitCreate()}
            >
              {t('common.create')}
            </button>
          </>
        }
      >
        <input
          className="native-input"
          value={createValue}
          placeholder={t('file.newFolder.placeholder')}
          onChange={(event) => setCreateValue(event.target.value)}
        />
      </Modal>
    </div>
  );
}

function formatTime(seconds: number): string {
  if (!seconds) {
    return '—';
  }
  const date = new Date(seconds * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(
    date.getMinutes(),
  )}`;
}
