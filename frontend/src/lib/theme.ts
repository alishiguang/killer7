import type { Accent, ThemeMode } from './types';

/** 把「跟随系统」解析为具体的深/浅色。 */
export function resolveTheme(mode: ThemeMode): 'dark' | 'light' {
  if (mode === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return mode;
}

/** 把主题与强调色写到 <html> 的 data 属性上，CSS 变量随之生效。 */
export function applyTheme(mode: ThemeMode, accent: Accent): void {
  const root = document.documentElement;
  root.dataset.theme = resolveTheme(mode);
  root.dataset.accent = accent;
}
