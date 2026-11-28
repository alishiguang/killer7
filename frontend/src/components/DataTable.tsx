import { memo, useMemo, type ReactElement, type ReactNode } from 'react';

import { compareValues, type SortDir } from '../lib/sort';
import { SortButton } from './SortButton';

export interface Column<T> {
  /** 列 key。 */
  key: string;
  /** 表头文案。 */
  title: string;
  /** 列宽，`remainder` 表示占满剩余空间。 */
  width?: number | 'remainder';
  /** 单元格内容。 */
  render: (item: T, index: number) => ReactNode;
  /** 右对齐。 */
  align?: 'left' | 'right';
  /** 排序取值。给了这一列，表头才能点。 */
  sortValue?: (item: T) => string | number;
}

export interface DataTableProps<T> {
  /** 列定义。 */
  columns: Column<T>[];
  /** 数据源。 */
  items: T[];
  /** 行 key。 */
  getKey: (item: T, index: number) => string;
  /** 行点击（可选）：选中该行。 */
  onRowClick?: (item: T, index: number) => void;
  /** 行双击（可选）：执行该行的主操作。 */
  onRowDoubleClick?: (item: T, index: number) => void;
  /** 行右键（可选，Task 25 接入原生菜单）。 */
  onRowContextMenu?: (item: T, index: number, event: React.MouseEvent) => void;
  /** 选中行的 key。 */
  selectedKey?: string | null;
  /** 空态内容。 */
  empty?: ReactNode;
  /** 当前排序列的 key（对应 Column.key）。 */
  sortKey?: string | null;
  /** 当前排序方向。 */
  sortDir?: SortDir;
  /** 点击表头切换排序。 */
  onSort?: (key: string) => void;
}

/** 表格行的 props。 */
interface TableRowProps<T> {
  columns: Column<T>[];
  item: T;
  index: number;
  selected: boolean;
  onRowClick?: (item: T, index: number) => void;
  onRowDoubleClick?: (item: T, index: number) => void;
  onRowContextMenu?: (item: T, index: number, event: React.MouseEvent) => void;
}

/**
 * 表格行。
 *
 * 单独抽出来并 memo 化：数据源刷新时（例如后台每轮推送进程列表），
 * 只有内容真的变了的那几行需要重绘，其余行直接复用上次的 DOM。
 *
 * memo 生效的前提是 props 稳定 —— `columns` 与两个回调都得是稳定引用，
 * 页面侧要用 `useCallback` / `useMemo` 保证这一点。
 *
 * `memo` 会把泛型擦成 `unknown`，所以这里显式断言回泛型函数类型，
 * 否则 `Column<T>[]` 赋不上 `Column<unknown>[]`。
 */
const TableRow = memo(function TableRowInner<T>({
  columns,
  item,
  index,
  selected,
  onRowClick,
  onRowDoubleClick,
  onRowContextMenu,
}: TableRowProps<T>) {
  return (
    <tr
      data-selected={selected ? 'true' : undefined}
      onClick={onRowClick ? () => onRowClick(item, index) : undefined}
      onDoubleClick={
        onRowDoubleClick ? () => onRowDoubleClick(item, index) : undefined
      }
      onContextMenu={
        onRowContextMenu
          ? (event) => {
              event.preventDefault();
              onRowContextMenu(item, index, event);
            }
          : undefined
      }
    >
      {columns.map((column) => (
        <td key={column.key} style={{ textAlign: column.align ?? 'left' }}>
          {column.render(item, index)}
        </td>
      ))}
    </tr>
  );
}) as unknown as <T>(props: TableRowProps<T>) => ReactElement;

/** 发丝描边的数据表格。 */
export function DataTable<T>({
  columns,
  items,
  getKey,
  onRowClick,
  onRowDoubleClick,
  onRowContextMenu,
  selectedKey,
  empty,
  sortKey,
  sortDir = 'asc',
  onSort,
}: DataTableProps<T>) {
  // 排序放在组件内部：页面只管给 sortKey / sortDir / onSort，
  // 少一处「忘了排序」的可能。
  const sorted = useMemo(() => {
    if (!sortKey) {
      return items;
    }
    const column = columns.find((item) => item.key === sortKey);
    if (!column?.sortValue) {
      return items;
    }
    const factor = sortDir === 'desc' ? -1 : 1;
    return [...items].sort((a, b) => factor * compareValues(column.sortValue!(a), column.sortValue!(b)));
  }, [columns, items, sortDir, sortKey]);

  return (
    <table className="data-table">
      <colgroup>
        {columns.map((column) => (
          <col
            key={column.key}
            style={
              column.width === 'remainder'
                ? undefined
                : { width: `${column.width ?? 100}px`, flex: '0 0 auto' }
            }
          />
        ))}
      </colgroup>
      <thead>
        <tr>
          {columns.map((column) => (
            <th key={column.key} style={{ textAlign: column.align ?? 'left' }}>
              {column.sortValue && onSort ? (
                <SortButton
                  label={column.title}
                  className={column.align === 'right' ? 'th-sort--right' : undefined}
                  active={sortKey === column.key}
                  dir={sortDir}
                  onClick={() => onSort(column.key)}
                />
              ) : (
                column.title
              )}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {sorted.length === 0 ? (
          <tr>
            <td colSpan={columns.length} className="empty-state">
              {empty ?? '暂无数据'}
            </td>
          </tr>
        ) : (
          sorted.map((item, index) => {
            const key = getKey(item, index);
            return (
              <TableRow
                key={key}
                columns={columns}
                item={item}
                index={index}
                selected={selectedKey === key}
                onRowClick={onRowClick}
                onRowDoubleClick={onRowDoubleClick}
                onRowContextMenu={onRowContextMenu}
              />
            );
          })
        )}
      </tbody>
    </table>
  );
}
