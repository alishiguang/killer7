import type { TabId } from '../lib/types';

export interface TabDef {
  id: TabId;
  /** 标签文案 key。 */
  labelKey: string;
  /** 分组：工具 / 系统。工具组与系统组之间在标签条上有一道细分隔。 */
  group: 'tools' | 'system';
}

/** 五个标签页的定义，顺序即标签条顺序。 */
export const TABS: TabDef[] = [
  { id: 'processes', labelKey: 'nav.process', group: 'tools' },
  { id: 'files', labelKey: 'nav.file', group: 'tools' },
  { id: 'clipboard', labelKey: 'nav.clipboard', group: 'tools' },
  { id: 'monitor', labelKey: 'nav.monitor', group: 'tools' },
  { id: 'settings', labelKey: 'nav.settings', group: 'system' },
];

/** 标签页 id 列表。 */
export const TAB_IDS: TabId[] = TABS.map((tab) => tab.id);
