import { derived, get, writable } from 'svelte/store';

export type ThemeMode = 'auto' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';

const STORAGE_KEY = 'theme-mode';

function readStoredMode(): ThemeMode {
  if (typeof localStorage === 'undefined') return 'auto';
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw === 'light' || raw === 'dark' || raw === 'auto' ? raw : 'auto';
  } catch {
    return 'auto';
  }
}

function readSystemTheme(): ResolvedTheme {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'light';
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export const themeMode = writable<ThemeMode>(readStoredMode());

const systemTheme = writable<ResolvedTheme>(readSystemTheme());

if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  const update = (matches: boolean) => systemTheme.set(matches ? 'dark' : 'light');

  update(mq.matches);

  if (typeof mq.addEventListener === 'function') {
    mq.addEventListener('change', (event) => update(event.matches));
  } else if (typeof mq.addListener === 'function') {
    mq.addListener((event: MediaQueryListEvent) => update(event.matches));
  }
}

export const resolvedTheme = derived(
  [themeMode, systemTheme],
  ([$themeMode, $systemTheme]): ResolvedTheme => {
    if ($themeMode === 'auto') return $systemTheme;
    return $themeMode;
  },
);

if (typeof localStorage !== 'undefined') {
  themeMode.subscribe((mode) => {
    try {
      localStorage.setItem(STORAGE_KEY, mode);
    } catch {
      // ignore storage failures
    }
  });
}

if (typeof document !== 'undefined') {
  resolvedTheme.subscribe((theme) => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  });
}

export function initTheme() {
  if (typeof document === 'undefined') return;
  const theme = get(resolvedTheme);
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
}

export function setThemeMode(mode: ThemeMode) {
  themeMode.set(mode);
}

export function cycleQuickToggle() {
  const current = get(resolvedTheme);
  themeMode.set(current === 'dark' ? 'light' : 'dark');
}
