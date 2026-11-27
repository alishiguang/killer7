import type { TabId } from '../lib/types';

export interface TabDef {
  id: TabId;
  /** 标签文案 key。 */
  labelKey: string;
}

/**
 * 三个标签页的定义，顺序即标签条顺序。
 *
 * 设置**不在**标签条里：它不是并列的视图，而是从工具栏角落的齿轮按钮打开的
 * 覆盖视图（见 `store.ts` 的 `settingsOpen`）。标签条因此只剩一组，不需要分组分隔线。
 */
export const TABS: TabDef[] = [
  { id: 'processes', labelKey: 'nav.process' },
  { id: 'files', labelKey: 'nav.file' },
  { id: 'clipboard', labelKey: 'nav.clipboard' },
];
