import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

import { Card, Kicker, Modal, SearchField, Segmented, VirtualList } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast } from '../lib/store';
import type { ProcessRow } from '../lib/types';

type SortKey = 'pid' | 'name' | 'port';

export function Processes() {
  const { t } = useI18n();
  const [rows, setRows] = useState<ProcessRow[]>([]);
  const [search, setSearch] = useState('');
  const [sortKey, setSortKey] = useState<SortKey>('pid');
  const [pendingKill, setPendingKill] = useState<ProcessRow | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);
  const [viewportHeight, setViewportHeight] = useState(420);

  // 首次拉取 + 订阅后端推送。
  useEffect(() => {
    let disposed = false;
    void api.listProcesses().then((list) => {
      if (!disposed) {
        setRows(list);
      }
    });
    void onEvent<ProcessRow[]>(EVENT.processes, (list) => setRows(list));
    return () => {
      disposed = true;
    };
  }, []);

  // 列表视口高度随窗口变化，虚拟列表按实际高度计算可见行数。
  useEffect(() => {
    const element = listRef.current;
    if (!element) {
      return;
    }
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry && entry.contentRect.height > 0) {
        setViewportHeight(entry.contentRect.height);
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const filtered = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return rows;
    }
    return rows.filter((row) => {
      if (row.displayName.toLowerCase().includes(keyword)) {
        return true;
      }
      if (String(row.pid).includes(keyword)) {
        return true;
      }
      return row.ports.some((port) => String(port.port).includes(keyword));
    });
  }, [rows, search]);

  const sorted = useMemo(() => {
    const copy = [...filtered];
    copy.sort((a, b) => {
      if (sortKey === 'name') {
        return a.displayName.localeCompare(b.displayName);
      }
      if (sortKey === 'port') {
        return (a.ports[0]?.port ?? 65535) - (b.ports[0]?.port ?? 65535);
      }
      return a.pid - b.pid;
    });
    return copy;
  }, [filtered, sortKey]);

  const onRefresh = useCallback(async () => {
    const list = await api.refreshProcesses();
    setRows(list);
  }, []);

  const confirmKill = useCallback(async () => {
    if (!pendingKill) {
      return;
    }
    const target = pendingKill;
    setPendingKill(null);
    try {
      await api.killProcess(target.pid);
      pushToast('info', t('process.kill.done', [target.displayName]));
      await onRefresh();
    } catch (error) {
      pushToast('error', String(error));
    }
  }, [onRefresh, pendingKill, t]);

  return (
    <div className="page">
      <div className="page__head">
        <Kicker index="01">{t('process.title')}</Kicker>
        <div className="page__tools">
          <SearchField
            value={search}
            onChange={setSearch}
            placeholder={t('process.search.placeholder')}
          />
          <button type="button" className="native-button" onClick={() => void onRefresh()}>
            {t('process.refresh')}
          </button>
        </div>
      </div>

      <Card
        title={t('process.count', [String(sorted.length)])}
        className="page__card"
        actions={
          <div className="page__tools">
            <Segmented<SortKey>
              value={sortKey}
              onChange={setSortKey}
              options={[
                { value: 'pid', label: t('process.pid') },
                { value: 'name', label: t('process.name') },
                { value: 'port', label: t('process.port') },
              ]}
            />
          </div>
        }
      >
        <div className="vtable" ref={listRef}>
          <div className="vrow vrow--head">
            <span className="vrow__pid">{t('process.pid')}</span>
            <span className="vrow__name">{t('process.name')}</span>
            <span className="vrow__port">{t('process.port')}</span>
          </div>
          <VirtualList
            items={sorted}
            height={viewportHeight}
            rowHeight={26}
            overscan={8}
            getKey={(row) => String(row.pid)}
            renderRow={(row) => (
              <div
                className="vrow"
                onClick={() => setPendingKill(row)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  void api.popupContextMenu('process', {
                    pid: row.pid,
                    displayName: row.displayName,
                    fullPath: row.fullPath,
                  });
                }}
              >
                <span className="mono vrow__pid">{row.pid}</span>
                <span className="vrow__name" title={row.fullPath}>
                  {row.displayName}
                </span>
                <span className="mono vrow__port">
                  {row.ports.length === 0
                    ? '—'
                    : row.ports.map((port) => `${port.address}:${port.port}`).join('  ')}
                </span>
              </div>
            )}
            empty={t('process.empty')}
          />
        </div>
      </Card>

      <Modal
        open={pendingKill !== null}
        title={t('process.kill.title')}
        onClose={() => setPendingKill(null)}
        footer={
          <>
            <button
              type="button"
              className="native-button"
              onClick={() => setPendingKill(null)}
            >
              {t('common.cancel')}
            </button>
            <button
              type="button"
              className="native-button native-button--accent"
              onClick={() => void confirmKill()}
            >
              {t('common.confirm')}
            </button>
          </>
        }
      >
        {pendingKill
          ? t('process.kill.confirm', [pendingKill.displayName, String(pendingKill.pid)])
          : ''}
      </Modal>
    </div>
  );
}
