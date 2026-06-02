import { writable } from 'svelte/store';
import type { Course, SyncStatus, AppConfig } from './types';

export type LayoutMode = 'tabs' | 'split' | 'grid';

export const courses = writable<Course[]>([]);
export const syncStatus = writable<SyncStatus>({
  pending_count: 0,
  last_success_at_ms: null,
  last_error: null,
  is_online: false,
});
export const isAuthenticated = writable<boolean>(false);
export const config = writable<AppConfig | null>(null);

export const layoutMode = writable<LayoutMode>('tabs');
export const activeCourseId = writable<number | null>(null);

// Course IDs selected for visible lanes (in split / grid modes)
export const visibleLanes = writable<number[]>([]);

// Recent finishes (cross-lane), most-recent first; capped client-side
export type FinishEvent = {
  timing_id: number;
  course_id: number;
  bib_number: number | null;
  total_ms: number | null;
  operator_id: string;
  at_ms: number;
};
export const recentFinishes = writable<FinishEvent[]>([]);
