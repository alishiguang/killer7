// Kicker.tsx
export interface KickerProps {
  /** 编号，如 "01"。 */
  index?: string;
  /** 标题。 */
  children: React.ReactNode;
}

export function Kicker({ index, children }: KickerProps) {
  return (
    <div className="section-title">
      {index ? <span className="kicker">{index}</span> : null}
      <span className="kicker">◆</span>
      <span className="kicker">{children}</span>
    </div>
  );
}
