import { describe, expect, it } from 'vitest';

import { interpolate, translate } from './i18n';

describe('interpolate', () => {
  it('按出现顺序替换 %@', () => {
    expect(interpolate('结束进程 %@ (%@)?', ['Finder', '42'])).toBe('结束进程 Finder (42)?');
  });

  it('参数不足时保留占位符', () => {
    expect(interpolate('结束进程 %@', [])).toBe('结束进程 %@');
  });
});

describe('translate', () => {
  it('命中 key 返回文案', () => {
    expect(translate({ 'tab.process': '进程管理' }, 'tab.process')).toBe('进程管理');
  });

  it('缺失 key 返回 key 本身', () => {
    expect(translate({}, 'missing.key')).toBe('missing.key');
  });

  it('带参数时自动插值', () => {
    expect(translate({ 'process.count': '共 %@ 个进程' }, 'process.count', ['12'])).toBe('共 12 个进程');
  });
});
