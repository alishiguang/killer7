import type { ReactNode } from 'react';

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
}

export interface DataTableProps<T> {
  /** 列定义。 */
  columns: Column<T>[];
  /** 数据源。 */
  items: T[];
  /** 行 key。 */
  getKey: (item: T, index: number) => string;
  /** 行点击（可选）。 */
  onRowClick?: (item: T, index: number) => void;
  /** 行右键（可选，Task 25 接入原生菜单）。 */
  onRowContextMenu?: (item: T, index: number, event: React.MouseEvent) => void;
  /** 选中行的 key。 */
  selectedKey?: string | null;
  /** 空态内容。 */
  empty?: ReactNode;
}

/** 发丝描边的数据表格。 */
export function DataTable<T>({
  columns,
  items,
  getKey,
  onRowClick,
  onRowContextMenu,
  selectedKey,
  empty,
}: DataTableProps<T>) {
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
              {column.title}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {items.length === 0 ? (
          <tr>
            <td colSpan={columns.length} className="empty-state">
              {empty ?? '暂无数据'}
            </td>
          </tr>
        ) : (
          items.map((item, index) => {
            const key = getKey(item, index);
            return (
              <tr
                key={key}
                data-selected={selectedKey === key ? 'true' : undefined}
                onClick={onRowClick ? () => onRowClick(item, index) : undefined}
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
          })
        )}
      </tbody>
    </table>
  );
}
