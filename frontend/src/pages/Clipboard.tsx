import { useCallback, useDeferredValue, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Kicker, type Column } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
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

  // 行右键回调保持稳定引用，否则 DataTable 里的 memo 行组件每次渲染都重绘。
  const handleRowContextMenu = useCallback((row: HistoryRow) => {
    void api.popupContextMenu('clipboard', { id: row.id, content: row.content });
  }, []);

  const columns = useMemo<Column<HistoryRow>[]>(
    () => [
      {
        key: 'content',
        title: t('clipboard.history'),
        width: 'remainder',
        render: (row) => <span title={row.content}>{row.content}</span>,
      },
      {
        key: 'kind',
        title: t('clipboard.column.type'),
        width: 90,
        render: (row) => <span className="text-muted">{t(KIND_KEYS[row.kind] ?? 'clipboard.type.text')}</span>,
      },
      {
        key: 'time',
        title: t('clipboard.column.time'),
        width: 130,
        render: (row) => <span className="mono">{formatTime(row.timestamp)}</span>,
      },
      {
        key: 'action',
        title: t('file.column.action'),
        width: 140,
        render: (row) => (
          <div className="row-actions">
            <button
              type="button"
              className="native-button"
              onClick={async (event) => {
                event.stopPropagation();
                await api.copyText(row.content);
                pushToast('info', t('clipboard.copied'));
              }}
            >
              {t('clipboard.copy')}
            </button>
            <button
              type="button"
              className="native-button"
              onClick={async (event) => {
                event.stopPropagation();
                try {
                  await api.removeHistoryItem(row.id);
                  await reload();
                } catch (error) {
                  pushToast('error', String(error));
                }
              }}
            >
              {t('clipboard.delete')}
            </button>
          </div>
        ),
      },
    ],
    [reload, t],
  );

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="03">{t('clipboard.title')}</Kicker>
        <div className="page__tools">
          <button
            type="button"
            className="native-button"
            onClick={async () => {
              try {
                await api.clearHistory();
                await reload();
              } catch (error) {
                pushToast('error', String(error));
              }
            }}
          >
            {t('clipboard.clearHistory')}
          </button>
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
          onRowContextMenu={handleRowContextMenu}
          empty={t('clipboard.empty')}
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
