// Checkbox.tsx
export interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: React.ReactNode;
  disabled?: boolean;
}

/** macOS 方块复选框。 */
export function Checkbox({ checked, onChange, label, disabled }: CheckboxProps) {
  return (
    <label className="checkbox">
      <input
        className="native-checkbox"
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      {label ? <span className="checkbox__label">{label}</span> : null}
    </label>
  );
}
