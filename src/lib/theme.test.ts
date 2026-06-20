import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

describe('theme', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.dataset.theme = 'light';
    document.documentElement.style.colorScheme = '';
    vi.resetModules();
  });

  it('defaults to auto when storage is empty', async () => {
    const { themeMode } = await import('./theme');
    expect(get(themeMode)).toBe('auto');
  });

  it('loads persisted mode from storage', async () => {
    localStorage.setItem('theme-mode', 'dark');
    const { themeMode } = await import('./theme');
    expect(get(themeMode)).toBe('dark');
  });

  it('persists mode changes', async () => {
    const { themeMode } = await import('./theme');
    themeMode.set('dark');
    expect(localStorage.getItem('theme-mode')).toBe('dark');
  });

  it('resolves auto mode to a concrete theme', async () => {
    const { themeMode, resolvedTheme } = await import('./theme');
    themeMode.set('light');
    expect(get(resolvedTheme)).toBe('light');
    themeMode.set('dark');
    expect(get(resolvedTheme)).toBe('dark');
  });

  it('updates the document theme when the mode changes', async () => {
    const { themeMode } = await import('./theme');
    themeMode.set('dark');
    expect(document.documentElement.dataset.theme).toBe('dark');
    themeMode.set('light');
    expect(document.documentElement.dataset.theme).toBe('light');
  });
});
