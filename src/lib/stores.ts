import { writable } from 'svelte/store';
import type { Course, SyncStatus, AppConfig } from './types';

export const courses = writable<Course[]>([]);
export const syncStatus = writable<SyncStatus>({
  pending_count: 0,
  last_success_at_ms: null,
  last_error: null,
  is_online: false,
});
export const isAuthenticated = writable<boolean>(false);
export const config = writable<AppConfig | null>(null);
