import { mount } from 'svelte';
import './app.css';
import App from './App.svelte';
import { initTheme } from './lib/theme';

initTheme();

const app = mount(App, { target: document.getElementById('app')! });

if (
  typeof window !== 'undefined' &&
  'serviceWorker' in navigator &&
  !('__TAURI__' in window)
) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {
      // ignore registration failures
    });
  });
}

export default app;
