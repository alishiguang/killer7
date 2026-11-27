import { Icon } from './Icon';
import type { SortDir } from '../lib/sort';

export interface SortButtonProps {
  /** 表头文案。 */
  label: string;
  /** 该列是否是当前排序列。 */
  active: boolean;
  /** 当前排序方向（仅当 active 时生效）。 */
  dir: SortDir;
  /** 点击回调。 */
  onClick: () => void;
  /** 附加类名（复用所在列的布局类）。 */
  className?: string;
}

/**
 * 可排序的表头。
 *
 * 全应用只有这一种排序入口：点表头切换该列的升/降序。
 * 非当前排序列的箭头平时隐去，悬停时淡淡地露出来提示「这列能排」。
 */
export function SortButton({ label, active, dir, onClick, className }: SortButtonProps) {
  return (
    <button
      type="button"
      className={className ? `th-sort ${className}` : 'th-sort'}
      data-active={active ? 'true' : undefined}
      onClick={onClick}
    >
      <span>{label}</span>
      <Icon
        name={active && dir === 'desc' ? 'sort-desc' : 'sort-asc'}
        className="th-sort__arrow"
        size={11}
      />
    </button>
  );
}
