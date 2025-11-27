import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@tauri-apps/api/tauri': path.resolve(__dirname, 'tests/mocks/tauri.ts'),
      '@tauri-apps/api/core': path.resolve(__dirname, 'tests/mocks/tauri.ts'),
      '@tauri-apps/api/event': path.resolve(__dirname, 'tests/mocks/tauri-event.ts'),
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['tests/setup.ts'],
    exclude: [
      'node_modules/**',
      'tests/accessibility.test.ts',
      'tests/drawings.test.ts',
      'tests/tutorial-autostart.test.ts',
      'tests/tutorial-store.test.ts',
      'tests/v0-trading.test.ts',
      'tests/wallet-store.test.ts',
      'tests/phantom-connect.test.tsx',
      'tests/stream-hooks.test.tsx',
      'tests/e2e/**',
      'e2e/**',
      'mobile-tests/**',
    ],
    coverage: {
      reporter: ['text', 'html'],
    },
  },
});
