import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { VirtualList } from './VirtualList';

describe('VirtualList', () => {
  const items = Array.from({ length: 500 }, (_, index) => index);

  it('只渲染视口附近的行，而不是全部 500 行', () => {
    render(
      <VirtualList
        items={items}
        height={260}
        rowHeight={26}
        overscan={8}
        getKey={(item) => String(item)}
        renderRow={(item) => <span>行 {item}</span>}
      />,
    );

    // 26px 行高 + 260px 视口 + overscan 8 → 渲染约 28 行。
    expect(screen.getByText('行 0')).toBeInTheDocument();
    expect(screen.queryByText('行 499')).not.toBeInTheDocument();
    expect(screen.getAllByText(/^行 \d+$/).length).toBeLessThan(60);
  });

  it('空数据时展示空态', () => {
    render(
      <VirtualList
        items={[]}
        height={200}
        getKey={(item: number) => String(item)}
        renderRow={(item: number) => <span>{item}</span>}
        empty="没有匹配的进程"
      />,
    );
    expect(screen.getByText('没有匹配的进程')).toBeInTheDocument();
  });
});
