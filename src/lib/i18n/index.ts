import { derived, writable } from 'svelte/store';
import { en } from './locales/en';
import { it } from './locales/it';
import type { Translations } from './types';

export type { Translations } from './types';

export type Locale = 'en' | 'it';

export const SUPPORTED_LOCALES: { code: Locale; nativeName: string }[] = [
  { code: 'en', nativeName: 'English' },
  { code: 'it', nativeName: 'Italiano' },
];

const STORAGE_KEY = 'locale';

const translations: Record<Locale, Translations> = { en, it };

function systemLocale(): Locale {
  if (typeof navigator === 'undefined') return 'it';
  const lang = (navigator.language ?? '').toLowerCase();
  if (lang.startsWith('en')) return 'en';
  if (lang.startsWith('it')) return 'it';
  return 'it';
}

function readStoredLocale(): Locale {
  if (typeof localStorage === 'undefined') return systemLocale();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw === 'en' || raw === 'it' ? raw : systemLocale();
  } catch {
    return systemLocale();
  }
}

export const locale = writable<Locale>(readStoredLocale());

if (typeof localStorage !== 'undefined') {
  locale.subscribe((l) => {
    try {
      localStorage.setItem(STORAGE_KEY, l);
    } catch {
      // ignore storage failures
    }
  });
}

export const t = derived(locale, ($locale): Translations => translations[$locale]);

/**
 * Interpolate named placeholders: i('Hello {name}', { name: 'World' }) → 'Hello World'
 */
export function i(template: string, vars: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (_, k) => String(vars[k] ?? ''));
}
