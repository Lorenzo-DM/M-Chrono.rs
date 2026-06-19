import { listen } from '@tauri-apps/api/event';
import type { Timing, PendingFinish, SyncStatus } from './types';

export type AppEvents = {
  'course:started': { course_id: number; started_at_ms: number };
  'course:ended': { course_id: number; ended_at_ms: number };
  'course:reset': { course_id: number };
  'athlete:finished': Timing;
  'pending:captured': PendingFinish;
  'split:recorded': any;
  'data:changed': null;
  'sync:status': SyncStatus;
  'network:status': { online: boolean };
  'auth:device_code': any;
  'auth:success': any;
  'auth:failed': { reason: string };
  'auth:required': null;
  'auth:logged_out': null;
  'duplicate:detected': {
    athlete_id: number;
    group_id: string;
    delta_ms: number;
    flagged: boolean;
  };
};

export function on<E extends keyof AppEvents>(
  event: E,
  handler: (payload: AppEvents[E]) => void,
) {
  return listen<AppEvents[E]>(event, e => handler(e.payload));
}
