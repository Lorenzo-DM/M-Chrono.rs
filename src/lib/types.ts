export type TimingStatus = 'Registered' | 'Running' | 'Finished' | 'Withdrawn';

export interface Course {
  id: number;
  name: string;
  distance_m: number | null;
  started_at_ms: number | null;
  scheduled_at_ms: number | null;
  ended_at_ms: number | null;
}

export interface Athlete {
  id: number;
  bib_number: number;
  first_name: string;
  last_name: string;
  course_id: number;
}

export interface Timing {
  id: number;
  remote_id: number | null;
  athlete_id: number | null;
  course_id: number;
  start_timestamp_ms: number | null;
  finish_timestamp_ms: number | null;
  status: TimingStatus;
  total_time_ms: number | null;
  operator_id: string;
  duplicate_group_id: string | null;
  duplicate_flagged: boolean;
  synced: boolean;
}

export interface PendingFinish {
  id: number;
  remote_id: number | null;
  course_id: number;
  finish_timestamp_ms: number;
  operator_id: string;
  assigned: boolean;
  synced: boolean;
}

export interface AthleteRow {
  athlete: Athlete;
  status: TimingStatus;
  finish_ms: number | null;
  total_ms: number | null;
  timing_id: number | null;
}

export interface CourseSnapshot {
  id: number;
  elapsed_ms: number | null;
  finishers_count: number;
  started: boolean;
  ended: boolean;
}

export interface DisplaySnapshot {
  courses: CourseSnapshot[];
  now_ms: number;
}

export interface DeviceCodeResponse {
  user_code: string;
  verification_uri: string;
  verification_uri_complete: string | null;
  expires_in: number;
  interval: number;
}

export interface SyncStatus {
  pending_count: number;
  last_success_at_ms: number | null;
  last_error: string | null;
  is_online: boolean;
}

export interface AppConfig {
  oidc_issuer_url: string;
  oidc_client_id: string;
  oidc_scopes: string;
  api_base_url: string;
  sync_interval_secs: number;
  operator_id: string;
  dedup_window_ms: number;
  dedup_warn_delta_ms: number;
}
