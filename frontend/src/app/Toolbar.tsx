import { Icon, SearchField } from '../components';
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import { TABS } from './navigation';

/**
 * 工具栏：左侧品牌与当前页，中间横向标签条，右侧搜索框与设置按钮。
 *
 * 结构参考 macOS「活动监视器」——它没有左侧栏，用工具栏里的分段标签切换视图，
 * 搜索框也放在工具栏右侧。搜索词因此是外壳级共享状态（见 `store.ts` 的 `search`）。
 *
 * 设置不占标签位：它只是一个角落里的齿轮按钮，避免为一个低频视图在主导航上
 * 占一个长期位置（见 `navigation.ts` 的说明）。
 */
export function Toolbar() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();
  const current = TABS.find((tab) => tab.id === state.activeTab);
  // 品牌副标题跟着当前视图走：设置是覆盖视图，打开时副标题显示「设置」。
  const brandTag = state.settingsOpen ? t('settings.title') : current ? t(current.labelKey) : '';

  return (
    <header className="toolbar">
      <div className="toolbar__brand" data-tauri-drag-region>
        <span className="toolbar__brand-name">{t('app.brand')}</span>
        <span className="toolbar__brand-tag">{brandTag}</span>
      </div>

      <nav className="tabstrip" role="tablist" aria-label={t('app.brand')}>
        {TABS.map((tab) => (
          <span key={tab.id} className="tabstrip__slot">
            <button
              type="button"
              role="tab"
              className="tabstrip__item"
              aria-selected={state.activeTab === tab.id}
              onClick={() => setState({ activeTab: tab.id, settingsOpen: false })}
            >
              {t(tab.labelKey)}
            </button>
          </span>
        ))}
      </nav>

      <div className="toolbar__spacer" data-tauri-drag-region />

      {state.settingsOpen ? null : (
        <div className="toolbar__search">
          <SearchField
            value={state.search}
            onChange={(value) => setState({ search: value })}
            placeholder={t('common.search')}
          />
        </div>
      )}

      <button
        type="button"
        className="native-button native-button--icon toolbar__settings"
        title={t('settings.title')}
        aria-label={t('settings.title')}
        aria-pressed={state.settingsOpen}
        onClick={() => setState({ settingsOpen: !state.settingsOpen, search: '' })}
      >
        <Icon name="gear" />
      </button>
    </header>
  );
}
