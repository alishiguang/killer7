import { useEffect, type ReactNode } from 'react';

export interface ModalProps {
  open: boolean;
  title: string;
  children: ReactNode;
  /** 底部按钮。 */
  footer?: ReactNode;
  onClose: () => void;
}

/** 半透明遮罩 + 圆角卡片，Esc 关闭。 */
export function Modal({ open, title, children, footer, onClose }: ModalProps) {
  useEffect(() => {
    if (!open) {
      return;
    }
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={onClose} role="presentation">
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(event) => event.stopPropagation()}
      >
        <header className="modal__head">{title}</header>
        <div className="modal__body">{children}</div>
        {footer ? <footer className="modal__foot">{footer}</footer> : null}
      </div>
    </div>
  );
}
