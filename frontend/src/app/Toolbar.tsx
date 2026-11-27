import { Icon } from '../components';
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';
import { TABS } from './navigation';

/**
 * 工具栏：左侧品牌，中间**居中**的标签条，右侧设置齿轮。
 *
 * 标签条用绝对定位居中（见 base.css 的 `.tabstrip`），左右两侧宽度不一样时
 * 也不会把标签推偏 —— 这是 macOS 应用（如 Safari）居中标签栏的标准做法。
 *
 * 品牌块只留一个 wordmark：当前页名已经有两处出处（标签条的高亮项、页面自己的
 * 编号标题），再在角上写一遍只是挤占红绿灯旁边的空间。
 *
 * 设置不占标签位：它只是一个角落里的齿轮按钮，避免为一个低频视图在主导航上
 * 占一个长期位置（见 `navigation.ts` 的说明）。
 */
export function Toolbar() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();

  return (
    <header className="toolbar">
      <div className="toolbar__brand" data-tauri-drag-region>
        <span className="toolbar__brand-name">{t('app.brand')}</span>
      </div>

      <nav className="tabstrip" role="tablist" aria-label={t('app.brand')}>
        {TABS.map((tab) => (
          <span key={tab.id} className="tabstrip__slot">
            <button
              type="button"
              role="tab"
              className="tabstrip__item"
              aria-selected={!state.settingsOpen && state.activeTab === tab.id}
              onClick={() => setState({ activeTab: tab.id, settingsOpen: false })}
            >
              {t(tab.labelKey)}
            </button>
          </span>
        ))}
      </nav>

      <div className="toolbar__spacer" data-tauri-drag-region />

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
