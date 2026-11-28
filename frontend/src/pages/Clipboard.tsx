import { useCallback, useDeferredValue, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Icon, Kicker, type Column } from '../components';
import { PageSearch } from '../app/PageSearch';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { nextSort, type SortState } from '../lib/sort';
import { pushToast, useAppStore } from '../lib/store';
import type { HistoryRow } from '../lib/types';

/** 剪贴板类型 → 文案 key。后端只给 `text` / `image` / `url` 三种原始值。 */
const KIND_KEYS: Record<string, string> = {
  text: 'clipboard.type.text',
  image: 'clipboard.type.image',
  url: 'clipboard.type.url',
};

export function Clipboard() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const [current, setCurrent] = useState('');
  const [history, setHistory] = useState<HistoryRow[]>([]);
  /** 排序：默认按时间倒序（新的在前），点表头切换方向。 */
  const [sort, setSort] = useState<SortState>({ key: 'time', dir: 'desc' });
  /** 当前选中的历史条目（单击选中、双击/右键作用于它）。 */
  const [selectedId, setSelectedId] = useState<number | null>(null);

  // 搜索框在工具栏上（与活动监视器一致），搜索词是外壳级的共享状态。
  const search = state.search;

  useEffect(() => {
    let disposed = false;
    void api.clipboardCurrent().then((text) => {
      if (!disposed) {
        setCurrent(text);
      }
    });
    void api.clipboardHistory().then((list) => {
      if (!disposed) {
        setHistory(list);
      }
    });
    void onEvent<string>(EVENT.clipboard, (text) => setCurrent(text));
    return () => {
      disposed = true;
    };
  }, []);

  // 搜索词走 useDeferredValue：打字时先更新输入框，筛选结果滞后一拍。
  const deferredSearch = useDeferredValue(search);

  const filtered = useMemo(() => {
    const keyword = deferredSearch.trim().toLowerCase();
    if (!keyword) {
      return history;
    }
    return history.filter((row) => row.content.toLowerCase().includes(keyword));
  }, [history, deferredSearch]);

  const reload = useCallback(async () => {
    setHistory(await api.clipboardHistory());
  }, []);

  // 行点击与行右键的回调保持稳定引用，否则 DataTable 里的 memo 行组件每次渲染都重绘。
  //
  // 单击只选中；右键同时把选中态挪到目标行 —— 菜单动作作用在「当前选中行」上，
  // 选中态与菜单目标必须一致。
  const handleRowClick = useCallback((row: HistoryRow) => {
    setSelectedId(row.id);
  }, []);

  /** 双击：主操作 —— 复制到系统剪贴板（与右键菜单里的「复制」同一条路径）。 */
  const handleRowDoubleClick = useCallback(async (row: HistoryRow) => {
    try {
      const ok = await api.copyToClipboard(row.content);
      pushToast(ok ? 'info' : 'error', t(ok ? 'clipboard.copied' : 'clipboard.copy.failed'));
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [t]);

  const handleRowContextMenu = useCallback((row: HistoryRow) => {
    setSelectedId(row.id);
    void api.popupContextMenu('clipboard', { id: row.id, content: row.content });
  }, []);

  const columns = useMemo<Column<HistoryRow>[]>(
    () => [
      {
        key: 'content',
        title: t('clipboard.history'),
        width: 'remainder',
        sortValue: (row) => row.content,
        render: (row) => <span title={row.content}>{row.content}</span>,
      },
      {
        key: 'kind',
        title: t('clipboard.column.type'),
        width: 90,
        sortValue: (row) => row.kind,
        render: (row) => <span className="text-muted">{t(KIND_KEYS[row.kind] ?? 'clipboard.type.text')}</span>,
      },
      {
        key: 'time',
        title: t('clipboard.column.time'),
        width: 130,
        sortValue: (row) => row.timestamp,
        render: (row) => <span className="mono">{formatTime(row.timestamp)}</span>,
      },
    ],
    [t],
  );

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="03">{t('clipboard.title')}</Kicker>
        <div className="page__head-actions">
          <PageSearch />
          <div className="page__tools">
            <button
              type="button"
              className="native-button native-button--icon"
              title={t('clipboard.clearHistory')}
              aria-label={t('clipboard.clearHistory')}
              onClick={async () => {
                try {
                  await api.clearHistory();
                  await reload();
                } catch (error) {
                  pushToast('error', String(error));
                }
              }}
            >
              <Icon name="x-circle" />
            </button>
          </div>
        </div>
      </div>

      <Card title={t('clipboard.current')} className="clipboard-current">
        <div className="clipboard-current__text">
          {current.length > 0 ? current : t('clipboard.empty')}
        </div>
      </Card>

      <Card title={t('clipboard.history.count', [String(filtered.length)])} className="page__card">
        <DataTable
          columns={columns}
          items={filtered}
          getKey={(row) => String(row.id)}
          selectedKey={selectedId === null ? null : String(selectedId)}
          onRowClick={handleRowClick}
          onRowDoubleClick={handleRowDoubleClick}
          onRowContextMenu={handleRowContextMenu}
          empty={t('clipboard.empty')}
          sortKey={sort.key}
          sortDir={sort.dir}
          onSort={(key) => setSort((prev) => nextSort(prev, key))}
        />
      </Card>
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
