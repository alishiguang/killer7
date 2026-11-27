// Checkbox.tsx
export interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: React.ReactNode;
  /** 无可见标签时的无障碍名称。 */
  ariaLabel?: string;
  disabled?: boolean;
}

/** macOS 方块复选框。 */
export function Checkbox({ checked, onChange, label, ariaLabel, disabled }: CheckboxProps) {
  return (
    <label className="checkbox">
      <input
        className="native-checkbox"
        type="checkbox"
        checked={checked}
        disabled={disabled}
        aria-label={ariaLabel}
        onChange={(event) => onChange(event.target.checked)}
      />
      {label ? <span className="checkbox__label">{label}</span> : null}
    </label>
  );
}
