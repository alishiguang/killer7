import { describe, expect, it } from 'vitest';
import tokens from './tokens.css?raw';

describe('cindy.cn 设计令牌', () => {
  it('保留 ink / paper 两套底色', () => {
    expect(tokens).toContain('#0f0e15');
    expect(tokens).toContain('#faf6ee');
  });

  it('保留奶油色正文与单一信号红强调', () => {
    expect(tokens).toContain('#f3ede2');
    expect(tokens).toContain('#f43d3f');
  });

  it('提供四档强调色与三档主题', () => {
    for (const accent of ['signal', 'haze', 'moss', 'amber']) {
      expect(tokens).toContain(`[data-accent='${accent}']`);
    }
    for (const theme of ['dark', 'light']) {
      expect(tokens).toContain(`[data-theme='${theme}']`);
    }
  });
});
