import { SearchField } from '../components';
import { useI18n } from '../lib/i18n';
import { useAppStore } from '../lib/store';

/**
 * 搜索框：贴在**页头右侧**，与该页的操作按钮同一行。
 *
 * 之前它独占工具栏下面一整行，把表格往下挤了一截；收进页头后，
 * 搜索与「翻页 / 新建」这类操作同属一层，视线不用在两行之间来回跳。
 *
 * 搜索词是外壳级的共享状态（见 `store.ts` 的 `search`），切页清空。
 * 设置视图没有列表可筛，页头里不渲染这一项。
 */
export function PageSearch() {
  const { t } = useI18n();
  const [state, setState] = useAppStore();

  return (
    <SearchField
      value={state.search}
      onChange={(value) => setState({ search: value })}
      placeholder={t('common.search')}
    />
  );
}
