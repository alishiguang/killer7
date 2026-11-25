// 约 60 行的轻量状态容器：useSyncExternalStore + 不可变更新。

import { useSyncExternalStore } from 'react';

import type { Notice, Settings, TabId } from './types';

export interface AppState {
  /** 完整设置，未加载时为 null。 */
  settings: Settings | null;
  /** 文案表。 */
  strings: Record<string, string>;
  /** 当前标签页。 */
  activeTab: TabId;
  /** 顶部提示。 */
  toast: (Notice & { id: number }) | null;
  /** 界面语言。 */
  locale: 'zh-Hans' | 'en';
}

const initialState: AppState = {
  settings: null,
  strings: {},
  activeTab: 'processes',
  toast: null,
  locale: 'zh-Hans',
};

let state: AppState = initialState;
const listeners = new Set<() => void>();

function getSnapshot(): AppState {
  return state;
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** 合并式更新全局状态。 */
export function setState(patch: Partial<AppState> | ((prev: AppState) => Partial<AppState>)): void {
  const delta = typeof patch === 'function' ? patch(state) : patch;
  state = { ...state, ...delta };
  listeners.forEach((listener) => listener());
}

/** 读取全局状态（订阅式）。 */
export function useAppState(): AppState {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

/** 读取并更新全局状态。 */
export function useAppStore(): [AppState, typeof setState] {
  return [useAppState(), setState];
}

/** 弹出顶部提示；3 秒后自动消失。 */
export function pushToast(kind: Notice['kind'], message: string): void {
  const id = Date.now();
  setState(() => ({ toast: { kind, message, id } }));
  window.setTimeout(() => {
    setState((prev) => (prev.toast?.id === id ? { toast: null } : {}));
  }, 3000);
}
