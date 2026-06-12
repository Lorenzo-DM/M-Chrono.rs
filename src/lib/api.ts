import { invoke } from '@tauri-apps/api/core';
import type {
  Course, Timing, PendingFinish, AthleteRow, DisplaySnapshot,
  AppConfig, DeviceCodeResponse, Athlete, AthleteInput, ImportSummary,
  Race, RaceInput, CourseInput,
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
  endCourse: (courseId: number, confirmName: string) =>
    invoke<number>('end_course', { courseId, confirmName }),
  restartCourse: (courseId: number, confirmName: string) =>
    invoke<void>('restart_course', { courseId, confirmName }),
  finishByBib: (bib: number) => invoke<Timing>('finish_by_bib', { bib }),
  finishByAthleteId: (athleteId: number) =>
    invoke<Timing>('finish_by_athlete_id', { athleteId }),
  capturePending: (courseId: number) =>
    invoke<PendingFinish>('capture_pending_finish', { courseId }),
  capturePendingTie: (courseId: number) =>
    invoke<PendingFinish>('capture_pending_tie', { courseId }),
  assignPending: (pendingId: number, bib: number) =>
    invoke<Timing>('assign_pending', { pendingId, bib }),
  withdrawAthlete: (bib: number) => invoke<void>('withdraw_athlete', { bib }),
  undoFinish: (timingId: number) => invoke<void>('undo_finish', { timingId }),
  reassignBib: (timingId: number, newBib: number) =>
    invoke<Timing>('reassign_bib', { timingId, newBib }),
  deletePendingFinish: (pendingId: number) =>
    invoke<void>('delete_pending_finish', { pendingId }),
  updateOperatorId: (id: string) => invoke<void>('update_operator_id', { id }),
  updateConfig: (patch: Partial<AppConfig>) =>
    invoke<AppConfig>('update_config', { patch }),
  startDeviceLogin: () => invoke<DeviceCodeResponse>('start_device_login'),
  isAuthenticated: () => invoke<boolean>('is_authenticated'),
  logout: () => invoke<void>('logout'),
  fetchRemoteData: () => invoke<{ courses_count: number; athletes_count: number }>('fetch_remote_data'),
  importAthletesFile: (path: string) =>
    invoke<ImportSummary>('import_athletes_file', { path }),
  saveAthlete: (id: number | null, input: AthleteInput) =>
    invoke<Athlete>('save_athlete', { id, input }),
  deleteAthlete: (id: number) => invoke<void>('delete_athlete', { id }),
  getAllAthletes: () => invoke<Athlete[]>('get_all_athletes'),
  getRaces: () => invoke<Race[]>('get_races'),
  saveRace: (id: number | null, input: RaceInput) =>
    invoke<Race>('save_race', { id, input }),
  deleteRace: (id: number) => invoke<void>('delete_race', { id }),
  saveCourse: (id: number | null, input: CourseInput) =>
    invoke<Course>('save_course', { id, input }),
  deleteCourse: (id: number) => invoke<void>('delete_course', { id }),
  getDuplicateGroups: () => invoke<any[]>('get_duplicate_groups'),
  exportResultsXlsx: (path: string) =>
    invoke<{ path: string; courses_count: number; athletes_count: number }>(
      'export_results_xlsx', { path }
    ),
};
