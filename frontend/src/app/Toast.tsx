import { useAppStore } from '../lib/store';

export function Toast() {
  const [state] = useAppStore();
  if (!state.toast) {
    return null;
  }
  return (
    <div className={`toast toast--${state.toast.kind}`} role="status">
      {state.toast.message}
    </div>
  );
}
