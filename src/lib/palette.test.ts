import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

describe('palette', () => {
  beforeEach(() => {
    localStorage.clear();
    delete document.documentElement.dataset.palette;
    vi.resetModules();
  });

  it('defaults to stone when storage is empty', async () => {
    const { palette } = await import('./palette');
    expect(get(palette)).toBe('stone');
  });

  it('loads a persisted palette from storage', async () => {
    localStorage.setItem('color-palette', 'nord');
    const { palette } = await import('./palette');
    expect(get(palette)).toBe('nord');
  });

  it('falls back to stone for an unknown persisted value', async () => {
    localStorage.setItem('color-palette', 'chartreuse');
    const { palette } = await import('./palette');
    expect(get(palette)).toBe('stone');
  });

  it('persists palette changes', async () => {
    const { palette } = await import('./palette');
    palette.set('slate');
    expect(localStorage.getItem('color-palette')).toBe('slate');
  });

  it('updates the document palette when it changes', async () => {
    const { palette } = await import('./palette');
    palette.set('nord');
    expect(document.documentElement.dataset.palette).toBe('nord');
    palette.set('slate');
    expect(document.documentElement.dataset.palette).toBe('slate');
  });

  it('applies the stored palette to the document on init', async () => {
    localStorage.setItem('color-palette', 'nord');
    const { initPalette } = await import('./palette');
    delete document.documentElement.dataset.palette;
    initPalette();
    expect(document.documentElement.dataset.palette).toBe('nord');
  });

  it('exposes every palette as a selectable option', async () => {
    const { PALETTES } = await import('./palette');
    expect(PALETTES.map((p) => p.value)).toEqual(['stone', 'slate', 'nord']);
  });
});
