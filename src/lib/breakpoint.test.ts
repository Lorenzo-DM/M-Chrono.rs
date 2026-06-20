import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

function mockInnerWidth(width: number) {
  Object.defineProperty(window, 'innerWidth', {
    configurable: true,
    value: width,
  });
}

describe('breakpoint', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('returns mobile for narrow widths', async () => {
    mockInnerWidth(600);
    const { breakpoint } = await import('./breakpoint');
    expect(get(breakpoint)).toBe('mobile');
  });

  it('returns tablet for mid widths', async () => {
    mockInnerWidth(900);
    const { breakpoint } = await import('./breakpoint');
    expect(get(breakpoint)).toBe('tablet');
  });

  it('returns desktop for laptop widths', async () => {
    mockInnerWidth(1280);
    const { breakpoint } = await import('./breakpoint');
    expect(get(breakpoint)).toBe('desktop');
  });

  it('returns wide for large widths', async () => {
    mockInnerWidth(1920);
    const { breakpoint } = await import('./breakpoint');
    expect(get(breakpoint)).toBe('wide');
  });

  it('updates on resize', async () => {
    mockInnerWidth(1280);
    const { breakpoint } = await import('./breakpoint');
    expect(get(breakpoint)).toBe('desktop');
    mockInnerWidth(600);
    window.dispatchEvent(new Event('resize'));
    expect(get(breakpoint)).toBe('mobile');
  });

  it('exposes derived mobile helper', async () => {
    mockInnerWidth(600);
    const { isMobile } = await import('./breakpoint');
    expect(get(isMobile)).toBe(true);
  });
});
