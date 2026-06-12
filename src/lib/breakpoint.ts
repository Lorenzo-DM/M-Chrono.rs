import { derived, writable } from 'svelte/store';

export type Breakpoint = 'mobile' | 'tablet' | 'desktop' | 'wide';

const BREAKPOINTS = {
  tablet: 700,
  desktop: 1100,
  wide: 1440,
} as const;

function computeBreakpoint(width: number): Breakpoint {
  if (width >= BREAKPOINTS.wide) return 'wide';
  if (width >= BREAKPOINTS.desktop) return 'desktop';
  if (width >= BREAKPOINTS.tablet) return 'tablet';
  return 'mobile';
}

function readWidth(): number {
  if (typeof window === 'undefined') return 1280;
  return window.innerWidth;
}

export const breakpoint = writable<Breakpoint>(computeBreakpoint(readWidth()));

if (typeof window !== 'undefined') {
  const update = () => breakpoint.set(computeBreakpoint(window.innerWidth));
  window.addEventListener('resize', update, { passive: true });
}

export const isMobile = derived(breakpoint, ($breakpoint) => $breakpoint === 'mobile');
export const isTablet = derived(breakpoint, ($breakpoint) => $breakpoint === 'tablet');
export const isDesktop = derived(
  breakpoint,
  ($breakpoint) => $breakpoint === 'desktop' || $breakpoint === 'wide',
);
