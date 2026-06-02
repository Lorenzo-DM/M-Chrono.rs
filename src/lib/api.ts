import { invoke } from '@tauri-apps/api/core';
import type {
  Course, Timing, PendingFinish, AthleteRow, DisplaySnapshot,
  AppConfig, DeviceCodeResponse,
} from './types';

export const api = {
  getCourses: () => invoke<Course[]>('get_courses'),
  pollDisplay: () => invoke<DisplaySnapshot>('poll_display'),
  getAthletesByCourse: (courseId: number) =>
    invoke<AthleteRow[]>('get_athletes_by_course', { courseId }),
  getPendingFinishes: (courseId: number) =>
    invoke<PendingFinish[]>('get_pending_finishes', { courseId }),
  getConfig: () => invoke<AppConfig>('get_config'),
  startCourse: (courseId: number) => invoke<number>('start_course', { courseId }),
  finishByBib: (bib: number) => invoke<Timing>('finish_by_bib', { bib }),
  finishByAthleteId: (athleteId: number) =>
    invoke<Timing>('finish_by_athlete_id', { athleteId }),
  capturePending: (courseId: number) =>
    invoke<PendingFinish>('capture_pending_finish', { courseId }),
  assignPending: (pendingId: number, bib: number) =>
    invoke<Timing>('assign_pending', { pendingId, bib }),
  withdrawAthlete: (bib: number) => invoke<void>('withdraw_athlete', { bib }),
  undoFinish: (timingId: number) => invoke<void>('undo_finish', { timingId }),
  updateOperatorId: (id: string) => invoke<void>('update_operator_id', { id }),
  startDeviceLogin: () => invoke<DeviceCodeResponse>('start_device_login'),
  isAuthenticated: () => invoke<boolean>('is_authenticated'),
  logout: () => invoke<void>('logout'),
};
