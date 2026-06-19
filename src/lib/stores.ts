import { writable } from 'svelte/store';
import type { Course, SyncStatus, AppConfig, CourseSnapshot, Athlete, Checkpoint } from './types';
import { api } from './api';

export type LayoutMode = 'tabs' | 'split' | 'grid';
export type NavView = 'timing' | 'results' | 'settings' | 'export';

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

// ---- Shared display poll ---------------------------------------------------
// A single interval (started in App.svelte) polls the backend and writes the
// per-course snapshot here, so every LaneCard reads from one source instead of
// each spawning its own 100ms loop.
export const courseSnapshots = writable<Record<number, CourseSnapshot>>({});
export const nowMs = writable<number>(Date.now());

let pollTimer: ReturnType<typeof setInterval> | null = null;

export function startDisplayPoll(intervalMs = 100): () => void {
  stopDisplayPoll();
  const tick = async () => {
    try {
      const snap = await api.pollDisplay();
      const map: Record<number, CourseSnapshot> = {};
      for (const c of snap.courses) map[c.id] = c;
      courseSnapshots.set(map);
      nowMs.set(snap.now_ms);
    } catch {
      // transient — keep last values
    }
  };
  tick();
  pollTimer = setInterval(tick, intervalMs);
  return stopDisplayPoll;
}

export function stopDisplayPoll(): void {
  if (pollTimer !== null) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

// ---- Shared athlete roster -------------------------------------------------
// Loaded once and refreshed on data changes; LaneCard/ResultsPage read from it
// instead of each issuing its own per-course query.
export const allAthletes = writable<Athlete[]>([]);

export async function refreshAthletes(): Promise<void> {
  try {
    allAthletes.set(await api.getAllAthletes());
  } catch {
    // leave previous roster in place
  }
}

// Checkpoints across all courses; LaneCard filters by course.
export const checkpoints = writable<Checkpoint[]>([]);

export async function refreshCheckpoints(): Promise<void> {
  try {
    checkpoints.set(await api.getCheckpoints());
  } catch {
    // keep previous
  }
}

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
