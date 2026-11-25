import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import { TABS } from './navigation';

export function TopBar() {
  const { t } = useI18n();
  const [state] = useAppStore();
  const current = TABS.find((tab) => tab.id === state.activeTab);

  return (
    <header className="topbar" data-tauri-drag-region>
      <div className="topbar__spacer" data-tauri-drag-region />
      <div className="topbar__title">
        <span className="kicker">{current ? t(current.labelKey) : ''}</span>
      </div>
      <div className="topbar__right" data-tauri-drag-region />
    </header>
  );
}
