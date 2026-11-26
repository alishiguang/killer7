export interface SearchFieldProps {
  /** 当前搜索词。 */
  value: string;
  /** 输入变化回调。 */
  onChange: (value: string) => void;
  /** 占位文案。 */
  placeholder?: string;
}

/** 搜索框：原生 input，保留系统 IME 与拼写检查。 */
export function SearchField({ value, onChange, placeholder }: SearchFieldProps) {
  return (
    <div className="search-field">
      <span className="search-field__icon" aria-hidden="true">
        ⌕
      </span>
      <input
        className="native-input"
        type="search"
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        spellCheck={false}
      />
    </div>
  );
}
