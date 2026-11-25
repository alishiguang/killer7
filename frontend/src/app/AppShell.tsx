import { useEffect } from 'react';

import { EVENT, api, onEvent } from '../lib/bridge';
import { applyTheme } from '../lib/theme';
import { pushToast, setState, useAppStore } from '../lib/store';
import type { TabId } from '../lib/types';
import { Sidebar } from './Sidebar';
import { Toast } from './Toast';
import { TopBar } from './TopBar';
import { Clipboard } from '../pages/Clipboard';
import { Files } from '../pages/Files';
import { Monitor } from '../pages/Monitor';
import { Plugins } from '../pages/Plugins';
import { Processes } from '../pages/Processes';
import { Settings } from '../pages/Settings';

const PAGES = {
  processes: Processes,
  files: Files,
  clipboard: Clipboard,
  monitor: Monitor,
  plugins: Plugins,
  settings: Settings,
} as const;

export function AppShell() {
  const [state] = useAppStore();

  // 启动引导：拉设置与文案，并应用主题。
  useEffect(() => {
    let disposed = false;

    void (async () => {
      const settings = await api.getSettings();
      const strings = await api.getStrings(settings.locale);
      if (disposed) {
        return;
      }
      applyTheme(settings.themeMode, settings.accent);
      setState({ settings, strings, locale: settings.locale });

      // 支持用 `?page=processes` 直接定位标签页：截图与自动化脚本需要能直达某一页，
      // 而不必先点侧栏。参数非法时忽略，留在默认页。
      const requested = new URLSearchParams(window.location.search).get('page');
      if (requested && requested in PAGES) {
        setState({ activeTab: requested as TabId });
      }
    })();

    return () => {
      disposed = true;
    };
  }, []);

  // 订阅后端事件。onEvent 返回 Promise<UnlistenFn>，把取消函数收集起来在卸载时统一调用。
  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    const track = (promise: Promise<() => void>) => {
      void promise.then((unlisten) => unlisteners.push(unlisten));
    };

    track(
      onEvent<string>(EVENT.menuAction, (action) => {
        if (action.startsWith('nav:')) {
          setState({ activeTab: action.slice(4) as TabId });
        } else if (action === 'app:settings') {
          // 应用菜单里的「设置…」发的是 app:settings，不是 nav: 前缀。
          setState({ activeTab: 'settings' });
        }
      }),
    );

    track(
      onEvent<{ kind: 'info' | 'error'; message: string }>(EVENT.notice, (notice) => {
        pushToast(notice.kind, notice.message);
      }),
    );

    track(onEvent(EVENT.trayOpen, () => setState({})));

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  const Page = PAGES[state.activeTab] ?? Processes;

  return (
    <div className="app-shell">
      <Sidebar />
      <div className="app-shell__main">
        <TopBar />
        <main className="app-shell__content">
          <Page />
        </main>
      </div>
      <Toast />
    </div>
  );
}
