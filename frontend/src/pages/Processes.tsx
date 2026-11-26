import {
  memo,
  useCallback,
  useDeferredValue,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import { Card, Icon, Kicker, Modal, Segmented, VirtualList } from '../components';
import { EVENT, api, onEvent } from '../lib/bridge';
import { useI18n } from '../lib/i18n';
import { pushToast, useAppStore } from '../lib/store';
import type { ProcessRow } from '../lib/types';

type SortKey = 'pid' | 'name' | 'port';

/**
 * 进程列表的一行。
 *
 * 单独抽出来并 memo 化：后台每轮推送进程列表时，只有内容真的变了的行
 * 需要重绘。`row` 是数据引用 —— `rows` 数组不变时行引用稳定，memo 才生效。
 */
const ProcessRowView = memo(function ProcessRowView({
  row,
  onSelect,
}: {
  row: ProcessRow;
  onSelect: (row: ProcessRow) => void;
}) {
  return (
    <div
      className="vrow"
      onClick={() => onSelect(row)}
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
  );
});

export function Processes() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const [rows, setRows] = useState<ProcessRow[]>([]);
  const [sortKey, setSortKey] = useState<SortKey>('pid');
  const [pendingKill, setPendingKill] = useState<ProcessRow | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);
  const [viewportHeight, setViewportHeight] = useState(420);

  // 搜索框在工具栏上（与活动监视器一致），搜索词是外壳级的共享状态。
  const search = state.search;

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
  //
  // 测量要可靠，不能只靠 ResizeObserver：它的首次回调可能早于外层布局定稿，
  // 拿到的是中间态高度；而列表自身的内联高度又不影响 `.vtable__body` 的尺寸，
  // 于是「尺寸不再变化 → 观察器不再回调 → 高度永远停在中间值」。
  // 所以用 useLayoutEffect 在每次布局提交后同步量一次（getBoundingClientRect
  // 会强制刷新布局，读到的是定稿值），再用 ResizeObserver 兜住后续的窗口变化。
  const measure = useCallback(() => {
    const element = listRef.current;
    if (!element) {
      return;
    }
    const next = Math.floor(element.getBoundingClientRect().height);
    if (next > 0) {
      setViewportHeight(next);
    }
  }, []);

  useLayoutEffect(() => {
    measure();
  }, [measure]);

  useEffect(() => {
    const element = listRef.current;
    if (!element) {
      return;
    }
    const observer = new ResizeObserver(() => measure());
    observer.observe(element);
    return () => observer.disconnect();
  }, [measure]);

  // 搜索词走 useDeferredValue：打字时先更新输入框，筛选结果滞后一拍，
  // 长列表下不会因为每个按键都重算 500 行的过滤而掉帧。
  const deferredSearch = useDeferredValue(search);

  const filtered = useMemo(() => {
    const keyword = deferredSearch.trim().toLowerCase();
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
  }, [rows, deferredSearch]);

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
          <button
            type="button"
            className="native-button native-button--icon"
            title={t('process.refresh')}
            aria-label={t('process.refresh')}
            onClick={() => void onRefresh()}
          >
            <Icon name="refresh" />
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
        <div className="vtable">
          <div className="vrow vrow--head">
            <span className="vrow__pid">{t('process.pid')}</span>
            <span className="vrow__name">{t('process.name')}</span>
            <span className="vrow__port">{t('process.port')}</span>
          </div>
          <div className="vtable__body" ref={listRef}>
            <VirtualList
              items={sorted}
              height={viewportHeight}
              rowHeight={26}
              overscan={8}
              getKey={(row) => String(row.pid)}
              renderRow={(row) => (
                <ProcessRowView row={row} onSelect={setPendingKill} />
              )}
              empty={t('process.empty')}
            />
          </div>
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
