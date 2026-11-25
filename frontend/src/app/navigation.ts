import type { TabId } from '../lib/types';

export interface TabDef {
  id: TabId;
  /** 侧栏文案 key。 */
  labelKey: string;
  /** 侧栏图标（几何符号，呼应设计语言的 ◆）。 */
  glyph: string;
  /** 分组：工具 / 系统。 */
  group: 'tools' | 'system';
}

/** 六个标签页的定义，顺序即侧栏顺序。 */
export const TABS: TabDef[] = [
  { id: 'processes', labelKey: 'nav.process', glyph: '◆', group: 'tools' },
  { id: 'files', labelKey: 'nav.file', glyph: '◆', group: 'tools' },
  { id: 'clipboard', labelKey: 'nav.clipboard', glyph: '◆', group: 'tools' },
  { id: 'monitor', labelKey: 'nav.monitor', glyph: '◆', group: 'tools' },
  { id: 'plugins', labelKey: 'nav.plugins', glyph: '◆', group: 'system' },
  { id: 'settings', labelKey: 'nav.settings', glyph: '◆', group: 'system' },
];

/** 标签页 id 列表。 */
export const TAB_IDS: TabId[] = TABS.map((tab) => tab.id);
