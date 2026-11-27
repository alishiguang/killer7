// 表头排序的共享状态机。
//
// 全应用只有一套排序交互：点表头切换该列的升/降序，箭头指示当前方向。
// 页面上不再放「按 PID / 按名称 / 按端口」这类分段切换控件。

export type SortDir = 'asc' | 'desc';

export interface SortState {
  /** 当前排序列的 key（对应 Column.key）；null 表示不排序。 */
  key: string | null;
  /** 当前排序方向。 */
  dir: SortDir;
}

/** 点表头后的下一状态：同一列来回切换方向，换列则从升序重新开始。 */
export function nextSort(current: SortState, key: string): SortState {
  if (current.key === key) {
    return { key, dir: current.dir === 'asc' ? 'desc' : 'asc' };
  }
  return { key, dir: 'asc' };
}

/** 比较两个排序取值：两边都是数字就按数值比，否则按字符串比。 */
export function compareValues(a: string | number, b: string | number): number {
  if (typeof a === 'number' && typeof b === 'number') {
    return a - b;
  }
  return String(a).localeCompare(String(b));
}
