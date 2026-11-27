import { SearchField } from '../components';
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';

/**
 * 搜索条：工具栏下面独占一行。
 *
 * 搜索词是外壳级的共享状态（见 `store.ts` 的 `search`），切页清空。
 * 设置视图没有列表可筛，这一行会随之隐藏。
 */
export function SearchBar() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();

  return (
    <div className="searchbar">
      <SearchField
        value={state.search}
        onChange={(value) => setState({ search: value })}
        placeholder={t('common.search')}
      />
    </div>
  );
}
