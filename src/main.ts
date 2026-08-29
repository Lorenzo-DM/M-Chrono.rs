import { mount } from 'svelte';
import './app.css';
import App from './App.svelte';
import { initTheme } from './lib/theme';
import { initPalette } from './lib/palette';

initTheme();
initPalette();

const app = mount(App, { target: document.getElementById('app')! });

export default app;
