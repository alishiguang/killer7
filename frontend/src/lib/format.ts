const UNITS = ['bytes', 'KB', 'MB', 'GB', 'TB', 'PB'] as const;

/** 字节数格式化，与 Rust 侧 format_bytes 口径一致。 */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} bytes`;
  }
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(1)} ${UNITS[unit]}`;
}

/** 秒级 UNIX 时间 → `MM-DD HH:mm`。 */
export function formatTimestamp(seconds: number): string {
  if (!seconds) {
    return '—';
  }
  const date = new Date(seconds * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(
    date.getMinutes(),
  )}`;
}

/** 百分比。 */
export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

/** 网络速率。 */
export function formatRate(value: number): string {
  return `${value.toFixed(2)} MB/s`;
}
