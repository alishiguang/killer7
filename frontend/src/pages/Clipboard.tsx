import { useCallback, useEffect, useMemo, useState } from 'react';

import { Card, DataTable, Kicker, SearchField, type Column } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { HistoryRow } from '../lib/types';

export function Clipboard() {
  const { t } = useI18n();
  const [current, setCurrent] = useState('');
  const [history, setHistory] = useState<HistoryRow[]>([]);
  const [search, setSearch] = useState('');

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

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return history;
    }
    return history.filter((row) => row.content.toLowerCase().includes(keyword));
  }, [history, search]);

  const reload = useCallback(async () => {
    setHistory(await api.clipboardHistory());
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
        title: t('clipboard.current'),
        width: 90,
        render: (row) => <span className="text-muted">{row.kind}</span>,
      },
      {
        key: 'time',
        title: t('common.more'),
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
          <SearchField
            value={search}
            onChange={setSearch}
            placeholder={t('clipboard.search.placeholder')}
          />
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
          onRowContextMenu={(row) => {
            void api.popupContextMenu('clipboard', { id: row.id, content: row.content });
          }}
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
