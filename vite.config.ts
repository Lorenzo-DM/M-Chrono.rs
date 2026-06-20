import { defineConfig, createLogger } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

const host = process.env.TAURI_DEV_HOST;

// Tauri desktop-only app — suppress Vite's localhost URL from dev output
const logger = createLogger();
const originalInfo = logger.info.bind(logger);
logger.info = (msg, opts) => {
  if (msg.includes('localhost') || msg.includes('Local') || msg.includes('Network')) return;
  originalInfo(msg, opts);
};

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  clearScreen: false,
  customLogger: logger,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
});
