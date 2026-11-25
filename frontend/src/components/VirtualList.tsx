import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

export interface VirtualListProps<T> {
  /** 数据源。 */
  items: T[];
  /** 行高（px），固定值。 */
  rowHeight?: number;
  /** 视口外额外渲染的行数。 */
  overscan?: number;
  /** 容器高度（px）。 */
  height: number;
  /** 行渲染函数。 */
  renderRow: (item: T, index: number) => ReactNode;
  /** 行 key。 */
  getKey: (item: T, index: number) => string;
  /** 空态内容。 */
  empty?: ReactNode;
}

/**
 * 固定行高的虚拟列表。
 *
 * 500 行只渲染约 30 个 DOM 节点（26px 行高 + overscan 8），
 * 保证滚动不掉帧。
 */
export function VirtualList<T>({
  items,
  rowHeight = 26,
  overscan = 8,
  height,
  renderRow,
  getKey,
  empty,
}: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const frameRef = useRef<number | null>(null);

  const onScroll = useCallback((event: React.UIEvent<HTMLDivElement>) => {
    const top = event.currentTarget.scrollTop;
    if (frameRef.current !== null) {
      cancelAnimationFrame(frameRef.current);
    }
    frameRef.current = requestAnimationFrame(() => {
      setScrollTop(top);
    });
  }, []);

  useEffect(
    () => () => {
      if (frameRef.current !== null) {
        cancelAnimationFrame(frameRef.current);
      }
    },
    [],
  );

  const { start, end, offsetY, totalHeight } = useMemo(() => {
    const visible = Math.ceil(height / rowHeight);
    const first = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
    const last = Math.min(items.length, first + visible + overscan * 2);
    return {
      start: first,
      end: last,
      offsetY: first * rowHeight,
      totalHeight: items.length * rowHeight,
    };
  }, [height, items.length, overscan, rowHeight, scrollTop]);

  if (items.length === 0) {
    return (
      <div className="scrollable" style={{ height, overflowY: 'auto' }}>
        <div className="empty-state">{empty ?? '暂无数据'}</div>
      </div>
    );
  }

  const slice = items.slice(start, end);

  return (
    <div className="scrollable" style={{ height, overflowY: 'auto' }} onScroll={onScroll}>
      <div style={{ height: totalHeight, position: 'relative' }}>
        <div style={{ position: 'absolute', top: offsetY, left: 0, right: 0 }}>
          {slice.map((item, index) => {
            const realIndex = start + index;
            return (
              <div key={getKey(item, realIndex)} style={{ height: rowHeight }}>
                {renderRow(item, realIndex)}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
