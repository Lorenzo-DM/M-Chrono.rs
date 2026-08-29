import { get, writable } from 'svelte/store';

export type Palette = 'stone' | 'slate' | 'nord';

const STORAGE_KEY = 'color-palette';

export const PALETTES: ReadonlyArray<{ value: Palette; label: string }> = [
  { value: 'stone', label: 'Stone' },
  { value: 'slate', label: 'Slate' },
  { value: 'nord', label: 'Nord' },
];

const DEFAULT_PALETTE: Palette = 'stone';

function isPalette(value: unknown): value is Palette {
  return PALETTES.some((p) => p.value === value);
}

function readStoredPalette(): Palette {
  if (typeof localStorage === 'undefined') return DEFAULT_PALETTE;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return isPalette(raw) ? raw : DEFAULT_PALETTE;
  } catch {
    return DEFAULT_PALETTE;
  }
}

export const palette = writable<Palette>(readStoredPalette());

if (typeof localStorage !== 'undefined') {
  palette.subscribe((value) => {
    try {
      localStorage.setItem(STORAGE_KEY, value);
    } catch {
      // ignore storage failures
    }
  });
}

if (typeof document !== 'undefined') {
  palette.subscribe((value) => {
    document.documentElement.dataset.palette = value;
  });
}

export function initPalette() {
  if (typeof document === 'undefined') return;
  document.documentElement.dataset.palette = get(palette);
}

export function setPalette(value: Palette) {
  palette.set(value);
}
