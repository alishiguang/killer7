import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Modal } from './Modal';

describe('Modal', () => {
  it('关闭时不渲染', () => {
    render(
      <Modal open={false} title="确认" onClose={() => {}}>
        内容
      </Modal>,
    );
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('点遮罩触发 onClose', () => {
    const onClose = vi.fn();
    render(
      <Modal open title="确认" onClose={onClose}>
        内容
      </Modal>,
    );
    fireEvent.click(screen.getByRole('presentation'));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Esc 触发 onClose', () => {
    const onClose = vi.fn();
    render(
      <Modal open title="确认" onClose={onClose}>
        内容
      </Modal>,
    );
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
