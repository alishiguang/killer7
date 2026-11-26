// 前端插值：与 Rust 侧 format_placeholders 口径一致，按顺序替换 %@。

import { useCallback } from 'react';

import { useAppState } from './store';

/** 把模板中的 `%@` 按出现顺序替换为给定参数。 */
export function interpolate(template: string, args: string[]): string {
  let out = '';
  let rest = template;
  let index = 0;

  for (;;) {
    const pos = rest.indexOf('%@');
    if (pos < 0) {
      break;
    }
    out += rest.slice(0, pos);
    out += args[index] ?? '%@';
    index += 1;
    rest = rest.slice(pos + 2);
  }

  return out + rest;
}

/** 取文案；缺失时返回 key 本身（与 NSLocalizedString 行为一致）。 */
export function translate(
  strings: Record<string, string>,
  key: string,
  args: string[] = [],
): string {
  const template = strings[key] ?? key;
  return args.length > 0 ? interpolate(template, args) : template;
}

/** 在组件里取文案函数。 */
export function useI18n(): {
  t: (key: string, args?: string[]) => string;
  strings: Record<string, string>;
} {
  const { strings } = useAppState();
  // t 的身份必须稳定：页面里的 useCallback / useMemo 多以它为依赖，
  // 每次渲染都换身份会让这些缓存全部失效、监听反复重订阅。
  const t = useCallback(
    (key: string, args: string[] = []) => translate(strings, key, args),
    [strings],
  );
  return { strings, t };
}
