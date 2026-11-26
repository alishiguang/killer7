import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  root: 'frontend',
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Rust 侧改动由 cargo 自己监听，不要让 Vite 扫描 target/ 造成风暴。
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'safari15',
    sourcemap: false,
    chunkSizeWarningLimit: 900,
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    // Vitest 默认 css:false，会把所有 .css 模块替换成空字符串；
    // tokens.test.ts 需要用 `?raw` 读到 CSS 原文，这里只为 ?raw 放行，其余 CSS 仍然打桩。
    css: { include: [/\?raw\b/] },
  },
});
