import { useCallback, useDeferredValue, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Kicker, Modal, SearchField, type Column } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { FileRow } from '../lib/types';

export function Files() {
  const { t } = useI18n();
  const [cwd, setCwd] = useState<string>('');
  const [rows, setRows] = useState<FileRow[]>([]);
  const [search, setSearch] = useState('');
  const [renaming, setRenaming] = useState<FileRow | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [creating, setCreating] = useState<'folder' | 'file' | null>(null);
  const [createValue, setCreateValue] = useState('');
  /** 原生右键菜单当前对应的行（「重命名 / 删除」需要它才能继续）。 */
  const [contextRow, setContextRow] = useState<FileRow | null>(null);

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
        render: (row) => <span className="mono">{row.sizeText}</span>,
      },
      {
        key: 'modified',
        title: t('file.column.modified'),
        width: 130,
        render: (row) => <span className="mono">{formatTime(row.modified)}</span>,
      },
      {
        key: 'action',
        title: t('file.column.action'),
        // 三个操作按钮（进入文件夹 / 重命名 / 删除）挤在 180px 里会溢出卡片右边界。
        width: 230,
        render: (row) => (
          <div className="row-actions">
            {row.isDir ? (
              <button
                type="button"
                className="native-button"
                onClick={(event) => {
                  event.stopPropagation();
                  void enter(row);
                }}
              >
                {t('file.openFolder')}
              </button>
            ) : (
              <button
                type="button"
                className="native-button"
                onClick={(event) => {
                  event.stopPropagation();
                  void api.revealInFinder(row.path);
                }}
              >
                {t('file.reveal')}
              </button>
            )}
            <button
              type="button"
              className="native-button"
              onClick={(event) => {
                event.stopPropagation();
                setRenaming(row);
                setRenameValue(row.name);
              }}
            >
              {t('file.rename')}
            </button>
            <button
              type="button"
              className="native-button"
              onClick={(event) => {
                event.stopPropagation();
                void remove(row);
              }}
            >
              {t('file.delete')}
            </button>
          </div>
        ),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [t, cwd],
  );

  const enter = useCallback(
    async (row: FileRow) => {
      setCwd(row.path);
      await load(row.path);
      await api.watchDir(row.path);
    },
    [load],
  );

  // 行点击与行右键的回调保持稳定引用，否则 DataTable 里的 memo 行组件
  // 每次渲染都会因为 props 变了而全部重绘。
  const handleRowClick = useCallback(
    (row: FileRow) => {
      if (row.isDir) {
        void enter(row);
      }
    },
    [enter],
  );

  const handleRowContextMenu = useCallback((row: FileRow) => {
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
    setCwd(parent);
    await load(parent);
    await api.watchDir(parent);
  }, [cwd, load]);

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
  }, [contextRow, remove]);

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
        <div className="page__tools">
          <button
            type="button"
            className="native-button"
            onClick={async () => {
              const picked = await api.pickDirectory();
              if (!picked) {
                return;
              }
              setCwd(picked);
              await load(picked);
              await api.watchDir(picked);
            }}
          >
            {t('file.openFolder')}
          </button>
          <button type="button" className="native-button" onClick={() => void goUp()}>
            {t('common.back')}
          </button>
          <button
            type="button"
            className="native-button"
            onClick={() => {
              setCreating('file');
              setCreateValue('');
            }}
          >
            {t('file.newFile')}
          </button>
          <button
            type="button"
            className="native-button"
            onClick={() => {
              setCreating('folder');
              setCreateValue('');
            }}
          >
            {t('file.newFolder')}
          </button>
          <SearchField value={search} onChange={setSearch} placeholder={t('file.search.placeholder')} />
        </div>
      </div>

      <Card title={cwd} className="page__card">
        <DataTable
          columns={columns}
          items={filtered}
          getKey={(row) => row.path}
          onRowClick={handleRowClick}
          onRowContextMenu={handleRowContextMenu}
          empty={t('common.empty')}
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
