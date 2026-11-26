import { describe, expect, it } from 'vitest';

import { formatBytes, formatPercent, formatRate, formatTimestamp } from './format';

describe('formatBytes', () => {
  it('小于 1KB 显示原始字节', () => {
    expect(formatBytes(0)).toBe('0 bytes');
    expect(formatBytes(512)).toBe('512 bytes');
  });

  it('按 1024 进位', () => {
    expect(formatBytes(2048)).toBe('2.0 KB');
    expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MB');
    expect(formatBytes(3 * 1024 ** 3)).toBe('3.0 GB');
  });
});

describe('formatTimestamp', () => {
  it('空值显示占位符', () => {
    expect(formatTimestamp(0)).toBe('—');
  });

  it('输出 MM-DD HH:mm', () => {
    expect(formatTimestamp(1700000000)).toMatch(/^\d{2}-\d{2} \d{2}:\d{2}$/);
  });
});

describe('数值格式', () => {
  it('百分比保留一位小数', () => {
    expect(formatPercent(12.34)).toBe('12.3%');
  });

  it('速率保留两位小数', () => {
    expect(formatRate(1.005)).toBe('1.00 MB/s');
  });
});
