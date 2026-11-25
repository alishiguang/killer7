import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import { TABS } from './navigation';

export function Sidebar() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();

  const tools = TABS.filter((tab) => tab.group === 'tools');
  const system = TABS.filter((tab) => tab.group === 'system');

  const renderGroup = (labelKey: string, items: typeof TABS) => (
    <div className="sidebar__group">
      <div className="sidebar__group-label">{t(labelKey)}</div>
      {items.map((tab) => (
        <button
          key={tab.id}
          type="button"
          className="sidebar__item"
          data-active={state.activeTab === tab.id ? 'true' : undefined}
          onClick={() => setState({ activeTab: tab.id })}
        >
          <span className="sidebar__glyph" aria-hidden="true">
            {tab.glyph}
          </span>
          <span>{t(tab.labelKey)}</span>
        </button>
      ))}
    </div>
  );

  return (
    <nav className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__brand-name">{t('app.brand')}</div>
        <div className="sidebar__brand-tag">{t('app.tagline')}</div>
      </div>
      {renderGroup('nav.section.tools', tools)}
      {renderGroup('nav.section.system', system)}
    </nav>
  );
}
