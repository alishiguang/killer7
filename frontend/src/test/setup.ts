import '@testing-library/jest-dom/vitest';

// jsdom 不实现 ResizeObserver；进程页用它测量虚拟列表视口高度。
// 桩必须放在共享 setup 里 —— AppShell 测试也会挂载进程页，
// 只在 Processes.test.tsx 里打桩会让 AppShell 测试炸在
// "ResizeObserver is not defined" 上。
globalThis.ResizeObserver = class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

// jsdom 不实现 Canvas 2D 上下文，会往 stderr 刷 "Not implemented"。
// Sparkline 只用到下面这几个方法与属性，给一个空实现桩即可，
// 同时保证测试输出是干净的。
HTMLCanvasElement.prototype.getContext = (() => ({
  setTransform: () => {},
  clearRect: () => {},
  beginPath: () => {},
  moveTo: () => {},
  lineTo: () => {},
  stroke: () => {},
  closePath: () => {},
  fill: () => {},
  strokeStyle: '',
  lineWidth: 0,
  fillStyle: '',
})) as unknown as typeof HTMLCanvasElement.prototype.getContext;
