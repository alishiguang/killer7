// Card.tsx
export interface CardProps {
  /** 卡片标题。 */
  title?: React.ReactNode;
  /** 右上角操作区。 */
  actions?: React.ReactNode;
  children?: React.ReactNode;
  className?: string;
}

export function Card({ title, actions, children, className }: CardProps) {
  return (
    <section className={`card ${className ?? ''}`}>
      {title || actions ? (
        <header className="card__head">
          <span className="card__title">{title}</span>
          {actions ? <div className="card__actions">{actions}</div> : null}
        </header>
      ) : null}
      <div className="card__body">{children}</div>
    </section>
  );
}
