import { SearchField } from '../components';
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import type { TabId } from '../lib/types';
import { TABS } from './navigation';

/**
 * 工具栏：左侧品牌与当前页，中间横向标签条，右侧搜索框。
 *
 * 结构参考 macOS「活动监视器」——它没有左侧栏，用工具栏里的分段标签切换视图，
 * 搜索框也放在工具栏右侧。搜索词因此是外壳级共享状态（见 `store.ts` 的 `search`）。
 */
export function Toolbar() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();
  const current = TABS.find((tab) => tab.id === state.activeTab);

  // 标签条：工具组与系统组之间有一道细分隔，保留原来的信息分组。
  let previousGroup: string | null = null;

  return (
    <header className="toolbar">
      <div className="toolbar__brand" data-tauri-drag-region>
        <span className="toolbar__brand-name">{t('app.brand')}</span>
        <span className="toolbar__brand-tag">{current ? t(current.labelKey) : ''}</span>
      </div>

      <nav className="tabstrip" role="tablist" aria-label={t('app.brand')}>
        {TABS.map((tab) => {
          const needsDivider = previousGroup !== null && previousGroup !== tab.group;
          previousGroup = tab.group;
          return (
            <span key={tab.id} className="tabstrip__slot">
              {needsDivider ? <span className="tabstrip__divider" aria-hidden="true" /> : null}
              <button
                type="button"
                role="tab"
                className="tabstrip__item"
                aria-selected={state.activeTab === tab.id}
                onClick={() => setState({ activeTab: tab.id as TabId })}
              >
                {t(tab.labelKey)}
              </button>
            </span>
          );
        })}
      </nav>

      <div className="toolbar__spacer" data-tauri-drag-region />

      <div className="toolbar__search">
        <SearchField
          value={state.search}
          onChange={(value) => setState({ search: value })}
          placeholder={t('common.search')}
        />
      </div>
    </header>
  );
}
