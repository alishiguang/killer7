import { useEffect } from 'react';

import { EVENT, api, onEvent } from '../lib/bridge';
import { applyTheme } from '../lib/theme';
import { pushToast, setState, useAppStore } from '../lib/store';
import type { TabId } from '../lib/types';
import { BottomBar } from './BottomBar';
import { Toast } from './Toast';
import { Toolbar } from './Toolbar';
import { Clipboard } from '../pages/Clipboard';
import { Files } from '../pages/Files';
import { Monitor } from '../pages/Monitor';
import { Processes } from '../pages/Processes';
import { Settings } from '../pages/Settings';

const PAGES = {
  processes: Processes,
  files: Files,
  clipboard: Clipboard,
  monitor: Monitor,
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
      // 而不必先点标签条。`?page=settings` 直达设置视图。参数非法时忽略，留在默认页。
      const requested = new URLSearchParams(window.location.search).get('page');
      if (requested === 'settings') {
        setState({ settingsOpen: true });
      } else if (requested && requested in PAGES) {
        setState({ activeTab: requested as TabId });
      }
    })();

    return () => {
      disposed = true;
    };
  }, []);

  // 切页时清空搜索词：搜索框是外壳级的共享状态，跨页沿用会让上一个页面的
  // 过滤条件莫名其妙地作用到下一个页面（活动监视器切标签也是清空的）。
  useEffect(() => {
    setState({ search: '' });
  }, [state.activeTab]);

  // 订阅后端事件。onEvent 返回 Promise<UnlistenFn>，把取消函数收集起来在卸载时统一调用。
  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    const track = (promise: Promise<() => void>) => {
      void promise.then((unlisten) => unlisteners.push(unlisten));
    };

    track(
      onEvent<string>(EVENT.menuAction, (action) => {
        if (action.startsWith('nav:')) {
          setState({ activeTab: action.slice(4) as TabId, settingsOpen: false });
        } else if (action === 'app:settings') {
          // 应用菜单里的「设置…」发的是 app:settings，不是 nav: 前缀。
          setState({ settingsOpen: true });
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

  // 底部汇总栏是「工具页的全局资源摘要」。系统监控页顶部已经有 CPU/内存/磁盘的
  // KPI 卡与两张采样曲线，底栏与之重复；设置页也不需要它。两处都隐藏。
  const showBottomBar = !state.settingsOpen && state.activeTab !== 'monitor';

  return (
    <div className="app-shell">
      <Toolbar />
      <main className="app-shell__content">
        {state.settingsOpen ? <Settings /> : <Page />}
      </main>
      {showBottomBar ? <BottomBar /> : null}
      <Toast />
    </div>
  );
}
