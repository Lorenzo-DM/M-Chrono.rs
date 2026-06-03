use crate::error::{AppError, AppResult};
use crate::models::{Athlete, Course, PendingFinish, Timing, TimingStatus};
use crate::timer::clock::{ClockProvider, SystemClock};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncCursor { pub last_seen_remote_id: i64, pub last_pull_at_ms: i64 }

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultRow {
    pub timing_id: i64,
    pub athlete_id: Option<i64>,
    pub bib_number: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub course_id: i64,
    pub course_name: String,
    pub start_timestamp_ms: Option<i64>,
    pub finish_timestamp_ms: Option<i64>,
    pub total_time_ms: Option<i64>,
    pub status: String,
    pub operator_id: String,
    pub duplicate_flagged: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DuplicateGroup {
    pub group_id: String,
    pub athlete_id: Option<i64>,
    pub bib_number: Option<i64>,
    pub timings: Vec<Timing>,
    pub delta_ms: i64,
}

pub struct Repo {
    pub conn: Arc<Mutex<Connection>>,
    pub clock: Arc<dyn ClockProvider>,
}

impl Repo {
    #[allow(dead_code)]
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn, clock: Arc::new(SystemClock) }
    }
    pub fn with_clock(conn: Arc<Mutex<Connection>>, clock: Arc<dyn ClockProvider>) -> Self {
        Self { conn, clock }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("poisoned db lock")
    }

    pub fn upsert_course(&self, c: &Course) -> AppResult<()> {
        let now = self.clock.now_ms();
        self.lock().execute(
            "INSERT INTO courses(id, name, distance_m, started_at_ms, scheduled_at_ms,
                                 created_at_ms, updated_at_ms)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 distance_m = excluded.distance_m,
                 updated_at_ms = excluded.updated_at_ms",
            params![c.id, c.name, c.distance_m, c.started_at_ms, c.scheduled_at_ms, now],
        )?;
        Ok(())
    }

    pub fn upsert_athlete(&self, a: &Athlete) -> AppResult<()> {
        let now = self.clock.now_ms();
        self.lock().execute(
            "INSERT INTO athletes(id, bib_number, first_name, last_name, course_id,
                                  created_at_ms, updated_at_ms)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 bib_number = excluded.bib_number,
                 first_name = excluded.first_name,
                 last_name = excluded.last_name,
                 course_id = excluded.course_id,
                 updated_at_ms = excluded.updated_at_ms",
            params![a.id, a.bib_number, a.first_name, a.last_name, a.course_id, now],
        )?;
        Ok(())
    }

    pub fn list_courses(&self) -> AppResult<Vec<Course>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, distance_m, started_at_ms, scheduled_at_ms FROM courses ORDER BY id"
        )?;
        let rows = stmt.query_map([], |r| Ok(Course {
            id: r.get(0)?, name: r.get(1)?, distance_m: r.get(2)?,
            started_at_ms: r.get(3)?, scheduled_at_ms: r.get(4)?,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn list_athletes(&self) -> AppResult<Vec<Athlete>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, bib_number, first_name, last_name, course_id FROM athletes ORDER BY bib_number"
        )?;
        let rows = stmt.query_map([], |r| Ok(Athlete {
            id: r.get(0)?, bib_number: r.get(1)?, first_name: r.get(2)?,
            last_name: r.get(3)?, course_id: r.get(4)?,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    #[allow(dead_code)]
    pub fn list_athletes_by_course(&self, course_id: i64) -> AppResult<Vec<Athlete>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, bib_number, first_name, last_name, course_id
             FROM athletes WHERE course_id = ?1 ORDER BY bib_number"
        )?;
        let rows = stmt.query_map(params![course_id], |r| Ok(Athlete {
            id: r.get(0)?, bib_number: r.get(1)?, first_name: r.get(2)?,
            last_name: r.get(3)?, course_id: r.get(4)?,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    #[allow(dead_code)]
    pub fn insert_timing_running(&self, athlete_id: i64, course_id: i64,
                                  start_ms: i64, operator_id: &str) -> AppResult<i64> {
        let now = self.clock.now_ms();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO timings(athlete_id, course_id, start_timestamp_ms, status,
                                 operator_id, created_at_ms, updated_at_ms, synced)
             VALUES(?1, ?2, ?3, 'Running', ?4, ?5, ?5, 0)",
            params![athlete_id, course_id, start_ms, operator_id, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_timings_bulk(&self, course_id: i64, athlete_ids: &[i64],
                                start_ms: i64, operator_id: &str) -> AppResult<Vec<i64>> {
        let now = self.clock.now_ms();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let mut ids = Vec::with_capacity(athlete_ids.len());
        {
            let mut stmt = tx.prepare(
                "INSERT INTO timings(athlete_id, course_id, start_timestamp_ms, status,
                                     operator_id, created_at_ms, updated_at_ms, synced)
                 VALUES(?1, ?2, ?3, 'Running', ?4, ?5, ?5, 0)"
            )?;
            for aid in athlete_ids {
                stmt.execute(params![aid, course_id, start_ms, operator_id, now])?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.execute(
            "UPDATE courses SET started_at_ms = ?1, updated_at_ms = ?1 WHERE id = ?2",
            params![start_ms, course_id],
        )?;
        tx.commit()?;
        Ok(ids)
    }

    pub fn update_finish(&self, timing_id: i64, finish_ms: i64, total_ms: i64) -> AppResult<()> {
        let now = self.clock.now_ms();
        self.lock().execute(
            "UPDATE timings SET finish_timestamp_ms = ?1, total_time_ms = ?2,
                                status = 'Finished', updated_at_ms = ?3, synced = 0
             WHERE id = ?4",
            params![finish_ms, total_ms, now, timing_id],
        )?;
        Ok(())
    }

    pub fn update_status(&self, timing_id: i64, status: TimingStatus) -> AppResult<()> {
        let now = self.clock.now_ms();
        self.lock().execute(
            "UPDATE timings SET status = ?1, updated_at_ms = ?2, synced = 0 WHERE id = ?3",
            params![status.as_str(), now, timing_id],
        )?;
        Ok(())
    }

    pub fn undo_finish(&self, timing_id: i64) -> AppResult<()> {
        let now = self.clock.now_ms();
        self.lock().execute(
            "UPDATE timings SET finish_timestamp_ms = NULL, total_time_ms = NULL,
                                status = 'Running', updated_at_ms = ?1, synced = 0
             WHERE id = ?2",
            params![now, timing_id],
        )?;
        Ok(())
    }

    pub fn find_running_timing_for_athlete(&self, athlete_id: i64, operator_id: &str)
        -> AppResult<Option<Timing>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, athlete_id, course_id, start_timestamp_ms,
                    finish_timestamp_ms, status, total_time_ms, operator_id,
                    duplicate_group_id, duplicate_flagged, synced
             FROM timings
             WHERE athlete_id = ?1 AND operator_id = ?2 AND status = 'Running' LIMIT 1"
        )?;
        let mut rows = stmt.query(params![athlete_id, operator_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_timing(row)?)),
            None => Ok(None),
        }
    }

    pub fn get_timing(&self, id: i64) -> AppResult<Option<Timing>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, athlete_id, course_id, start_timestamp_ms,
                    finish_timestamp_ms, status, total_time_ms, operator_id,
                    duplicate_group_id, duplicate_flagged, synced
             FROM timings WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::map_timing(row)?)),
            None => Ok(None),
        }
    }

    fn map_timing(row: &rusqlite::Row) -> rusqlite::Result<Timing> {
        let status_str: String = row.get(6)?;
        Ok(Timing {
            id: row.get(0)?,
            remote_id: row.get(1)?,
            athlete_id: row.get(2)?,
            course_id: row.get(3)?,
            start_timestamp_ms: row.get(4)?,
            finish_timestamp_ms: row.get(5)?,
            status: TimingStatus::from_str(&status_str).unwrap_or(TimingStatus::Registered),
            total_time_ms: row.get(7)?,
            operator_id: row.get(8)?,
            duplicate_group_id: row.get(9)?,
            duplicate_flagged: row.get::<_, i64>(10)? != 0,
            synced: row.get::<_, i64>(11)? != 0,
        })
    }

    pub fn insert_pending_finish(&self, course_id: i64, ts_ms: i64, operator_id: &str)
        -> AppResult<PendingFinish> {
        let now = self.clock.now_ms();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO pending_finishes(course_id, finish_timestamp_ms, operator_id,
                                          created_at_ms, assigned, synced)
             VALUES(?1, ?2, ?3, ?4, 0, 0)",
            params![course_id, ts_ms, operator_id, now],
        )?;
        Ok(PendingFinish {
            id: conn.last_insert_rowid(),
            remote_id: None,
            course_id,
            finish_timestamp_ms: ts_ms,
            operator_id: operator_id.into(),
            assigned: false,
            synced: false,
        })
    }

    pub fn mark_pending_assigned(&self, pending_id: i64) -> AppResult<()> {
        self.lock().execute(
            "UPDATE pending_finishes SET assigned = 1 WHERE id = ?1",
            params![pending_id],
        )?;
        Ok(())
    }

    pub fn delete_pending_finish(&self, pending_id: i64) -> AppResult<()> {
        let n = self.lock().execute(
            "DELETE FROM pending_finishes WHERE id = ?1 AND assigned = 0",
            params![pending_id],
        )?;
        if n == 0 {
            return Err(AppError::NotFound(format!("pending {}", pending_id)));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn list_pending_open(&self, course_id: i64) -> AppResult<Vec<PendingFinish>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, course_id, finish_timestamp_ms, operator_id, assigned, synced
             FROM pending_finishes WHERE course_id = ?1 AND assigned = 0
             ORDER BY finish_timestamp_ms"
        )?;
        let rows = stmt.query_map(params![course_id], |r| Ok(PendingFinish {
            id: r.get(0)?,
            remote_id: r.get(1)?,
            course_id: r.get(2)?,
            finish_timestamp_ms: r.get(3)?,
            operator_id: r.get(4)?,
            assigned: r.get::<_, i64>(5)? != 0,
            synced: r.get::<_, i64>(6)? != 0,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn fetch_unsynced_timings(&self, limit: i64) -> AppResult<Vec<Timing>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, athlete_id, course_id, start_timestamp_ms,
                    finish_timestamp_ms, status, total_time_ms, operator_id,
                    duplicate_group_id, duplicate_flagged, synced
             FROM timings
             WHERE synced = 0 AND sync_attempts < 5
             ORDER BY id LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit], Self::map_timing)?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn fetch_unsynced_pending(&self, limit: i64) -> AppResult<Vec<PendingFinish>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, course_id, finish_timestamp_ms, operator_id, assigned, synced
             FROM pending_finishes
             WHERE synced = 0 AND sync_attempts < 5
             ORDER BY id LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit], |r| Ok(PendingFinish {
            id: r.get(0)?, remote_id: r.get(1)?, course_id: r.get(2)?,
            finish_timestamp_ms: r.get(3)?, operator_id: r.get(4)?,
            assigned: r.get::<_, i64>(5)? != 0, synced: r.get::<_, i64>(6)? != 0,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn mark_timing_synced(&self, local_id: i64, remote_id: i64) -> AppResult<()> {
        self.lock().execute(
            "UPDATE timings SET synced = 1, remote_id = ?1, last_sync_error = NULL
             WHERE id = ?2",
            params![remote_id, local_id],
        )?;
        Ok(())
    }

    pub fn mark_pending_synced(&self, local_id: i64, remote_id: i64) -> AppResult<()> {
        self.lock().execute(
            "UPDATE pending_finishes SET synced = 1, remote_id = ?1 WHERE id = ?2",
            params![remote_id, local_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn record_sync_error_timing(&self, local_id: i64, error: &str) -> AppResult<()> {
        self.lock().execute(
            "UPDATE timings SET sync_attempts = sync_attempts + 1, last_sync_error = ?1
             WHERE id = ?2",
            params![error, local_id],
        )?;
        Ok(())
    }

    pub fn upsert_remote_timing(&self, t: &Timing) -> AppResult<()> {
        let now = self.clock.now_ms();
        let remote = t.remote_id.ok_or_else(||
            AppError::Db("upsert_remote_timing requires remote_id".into()))?;
        self.lock().execute(
            "INSERT INTO timings(remote_id, athlete_id, course_id, start_timestamp_ms,
                                 finish_timestamp_ms, status, total_time_ms, operator_id,
                                 created_at_ms, updated_at_ms, synced)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, 1)
             ON CONFLICT(remote_id) DO UPDATE SET
                 athlete_id = excluded.athlete_id,
                 finish_timestamp_ms = excluded.finish_timestamp_ms,
                 status = excluded.status,
                 total_time_ms = excluded.total_time_ms,
                 updated_at_ms = excluded.updated_at_ms,
                 synced = 1",
            params![remote, t.athlete_id, t.course_id, t.start_timestamp_ms,
                    t.finish_timestamp_ms, t.status.as_str(), t.total_time_ms,
                    t.operator_id, now],
        )?;
        Ok(())
    }

    pub fn set_duplicate_group(&self, timing_ids: &[i64], group_id: &str, flagged: bool)
        -> AppResult<()> {
        let now = self.clock.now_ms();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "UPDATE timings SET duplicate_group_id = ?1, duplicate_flagged = ?2,
                                    updated_at_ms = ?3 WHERE id = ?4"
            )?;
            for id in timing_ids {
                stmt.execute(params![group_id, flagged as i64, now, id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_finished_timings_for_athlete(&self, athlete_id: i64) -> AppResult<Vec<Timing>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, athlete_id, course_id, start_timestamp_ms,
                    finish_timestamp_ms, status, total_time_ms, operator_id,
                    duplicate_group_id, duplicate_flagged, synced
             FROM timings
             WHERE athlete_id = ?1 AND status = 'Finished'
             ORDER BY finish_timestamp_ms"
        )?;
        let rows = stmt.query_map(params![athlete_id], Self::map_timing)?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn get_sync_cursor(&self, resource: &str) -> AppResult<SyncCursor> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT last_seen_remote_id, last_pull_at_ms FROM sync_cursor WHERE resource = ?1"
        )?;
        let mut rows = stmt.query(params![resource])?;
        if let Some(row) = rows.next()? {
            Ok(SyncCursor { last_seen_remote_id: row.get(0)?, last_pull_at_ms: row.get(1)? })
        } else {
            Ok(SyncCursor { last_seen_remote_id: 0, last_pull_at_ms: 0 })
        }
    }

    pub fn update_sync_cursor(&self, resource: &str, last_id: i64, now_ms: i64) -> AppResult<()> {
        self.lock().execute(
            "INSERT INTO sync_cursor(resource, last_seen_remote_id, last_pull_at_ms)
             VALUES(?1, ?2, ?3)
             ON CONFLICT(resource) DO UPDATE SET
                 last_seen_remote_id = excluded.last_seen_remote_id,
                 last_pull_at_ms = excluded.last_pull_at_ms",
            params![resource, last_id, now_ms],
        )?;
        Ok(())
    }

    pub fn list_results_by_course(&self, course_id: i64) -> AppResult<Vec<ResultRow>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.athlete_id, a.bib_number, a.first_name, a.last_name,
                    t.course_id, c.name,
                    t.start_timestamp_ms, t.finish_timestamp_ms, t.total_time_ms,
                    t.status, t.operator_id, t.duplicate_flagged
             FROM timings t
             LEFT JOIN athletes a ON a.id = t.athlete_id
             JOIN courses c ON c.id = t.course_id
             WHERE t.course_id = ?1
             ORDER BY CASE WHEN t.total_time_ms IS NULL THEN 1 ELSE 0 END,
                      t.total_time_ms ASC"
        )?;
        let rows = stmt.query_map(params![course_id], |r| Ok(ResultRow {
            timing_id: r.get(0)?, athlete_id: r.get(1)?, bib_number: r.get(2)?,
            first_name: r.get(3)?, last_name: r.get(4)?,
            course_id: r.get(5)?, course_name: r.get(6)?,
            start_timestamp_ms: r.get(7)?, finish_timestamp_ms: r.get(8)?,
            total_time_ms: r.get(9)?, status: r.get(10)?, operator_id: r.get(11)?,
            duplicate_flagged: r.get::<_, i64>(12)? != 0,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn list_all_timings(&self) -> AppResult<Vec<Timing>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, athlete_id, course_id, start_timestamp_ms,
                    finish_timestamp_ms, status, total_time_ms, operator_id,
                    duplicate_group_id, duplicate_flagged, synced
             FROM timings"
        )?;
        let rows = stmt.query_map([], Self::map_timing)?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn list_duplicate_groups(&self) -> AppResult<Vec<DuplicateGroup>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT t.duplicate_group_id, t.athlete_id, a.bib_number, t.id, t.remote_id,
                    t.course_id, t.start_timestamp_ms, t.finish_timestamp_ms,
                    t.status, t.total_time_ms, t.operator_id,
                    t.duplicate_flagged, t.synced
             FROM timings t LEFT JOIN athletes a ON a.id = t.athlete_id
             WHERE t.duplicate_group_id IS NOT NULL AND t.duplicate_group_id != ''
               AND t.duplicate_flagged = 1
             ORDER BY t.duplicate_group_id, t.finish_timestamp_ms",
        )?;
        let mut groups: std::collections::BTreeMap<String, DuplicateGroup> = Default::default();
        let rows = stmt.query_map([], |r| {
            let gid: String = r.get(0)?;
            let aid: Option<i64> = r.get(1)?;
            let bib: Option<i64> = r.get(2)?;
            let status_str: String = r.get(8)?;
            let t = Timing {
                id: r.get(3)?,
                remote_id: r.get(4)?,
                athlete_id: aid,
                course_id: r.get(5)?,
                start_timestamp_ms: r.get(6)?,
                finish_timestamp_ms: r.get(7)?,
                status: TimingStatus::from_str(&status_str).unwrap_or(TimingStatus::Finished),
                total_time_ms: r.get(9)?,
                operator_id: r.get(10)?,
                duplicate_group_id: Some(gid.clone()),
                duplicate_flagged: r.get::<_, i64>(11)? != 0,
                synced: r.get::<_, i64>(12)? != 0,
            };
            Ok((gid, aid, bib, t))
        })?;
        for row in rows {
            let (gid, aid, bib, t) = row?;
            let g = groups.entry(gid.clone()).or_insert(DuplicateGroup {
                group_id: gid,
                athlete_id: aid,
                bib_number: bib,
                timings: Vec::new(),
                delta_ms: 0,
            });
            g.timings.push(t);
        }
        for g in groups.values_mut() {
            let ts: Vec<i64> = g.timings.iter().filter_map(|t| t.finish_timestamp_ms).collect();
            if let (Some(min), Some(max)) = (ts.iter().min(), ts.iter().max()) {
                g.delta_ms = max - min;
            }
        }
        Ok(groups.into_values().collect())
    }

    pub fn list_all_pending_open(&self) -> AppResult<Vec<PendingFinish>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, remote_id, course_id, finish_timestamp_ms, operator_id, assigned, synced
             FROM pending_finishes WHERE assigned = 0"
        )?;
        let rows = stmt.query_map([], |r| Ok(PendingFinish {
            id: r.get(0)?, remote_id: r.get(1)?, course_id: r.get(2)?,
            finish_timestamp_ms: r.get(3)?, operator_id: r.get(4)?,
            assigned: r.get::<_, i64>(5)? != 0, synced: r.get::<_, i64>(6)? != 0,
        }))?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};

    fn fresh() -> Repo {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        Repo::new(db.conn.clone())
    }

    #[test]
    fn upsert_course_inserts_then_updates() {
        let r = fresh();
        let mut c = Course { id: 1, name: "21K".into(), distance_m: Some(21_000),
                             started_at_ms: None, scheduled_at_ms: None };
        r.upsert_course(&c).unwrap();
        c.name = "21K Trail".into();
        r.upsert_course(&c).unwrap();
        let list = r.list_courses().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "21K Trail");
    }

    #[test]
    fn upsert_athlete_then_list_by_course() {
        let r = fresh();
        r.upsert_course(&Course {
            id: 1, name: "21K".into(), distance_m: None,
            started_at_ms: None, scheduled_at_ms: None,
        }).unwrap();
        r.upsert_athlete(&Athlete {
            id: 100, bib_number: 7, first_name: "Mario".into(),
            last_name: "Rossi".into(), course_id: 1,
        }).unwrap();
        let list = r.list_athletes_by_course(1).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].bib_number, 7);
    }

    #[test]
    fn insert_running_and_finish_updates_total() {
        let r = fresh();
        r.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                   started_at_ms: None, scheduled_at_ms: None }).unwrap();
        r.upsert_athlete(&Athlete { id: 1, bib_number: 1, first_name: "a".into(),
                                     last_name: "b".into(), course_id: 1 }).unwrap();
        let tid = r.insert_timing_running(1, 1, 1_000, "PC-A").unwrap();
        r.update_finish(tid, 1_500, 500).unwrap();
        let t = r.get_timing(tid).unwrap().unwrap();
        assert_eq!(t.total_time_ms, Some(500));
        assert_eq!(t.status, TimingStatus::Finished);
        assert!(!t.synced);
    }

    #[test]
    fn pending_finish_lifecycle() {
        let r = fresh();
        r.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                   started_at_ms: None, scheduled_at_ms: None }).unwrap();
        let p = r.insert_pending_finish(1, 12345, "PC-A").unwrap();
        assert!(r.list_pending_open(1).unwrap().len() == 1);
        r.mark_pending_assigned(p.id).unwrap();
        assert!(r.list_pending_open(1).unwrap().is_empty());
    }

    #[test]
    fn find_running_timing_returns_correct_athlete() {
        let r = fresh();
        r.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                   started_at_ms: None, scheduled_at_ms: None }).unwrap();
        r.upsert_athlete(&Athlete { id: 1, bib_number: 1, first_name: "a".into(),
                                     last_name: "b".into(), course_id: 1 }).unwrap();
        r.insert_timing_running(1, 1, 1_000, "PC-A").unwrap();
        let t = r.find_running_timing_for_athlete(1, "PC-A").unwrap().unwrap();
        assert_eq!(t.athlete_id, Some(1));
        assert!(r.find_running_timing_for_athlete(1, "PC-B").unwrap().is_none());
    }

    #[test]
    fn unsynced_query_excludes_synced_rows() {
        let r = fresh();
        r.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                   started_at_ms: None, scheduled_at_ms: None }).unwrap();
        r.upsert_athlete(&Athlete { id: 1, bib_number: 1, first_name: "a".into(),
                                     last_name: "b".into(), course_id: 1 }).unwrap();
        let t1 = r.insert_timing_running(1, 1, 100, "PC-A").unwrap();
        r.update_finish(t1, 200, 100).unwrap();
        assert_eq!(r.fetch_unsynced_timings(10).unwrap().len(), 1);
        r.mark_timing_synced(t1, 999).unwrap();
        assert_eq!(r.fetch_unsynced_timings(10).unwrap().len(), 0);
    }

    #[test]
    fn sync_cursor_default_zero_and_update() {
        let r = fresh();
        let c = r.get_sync_cursor("timings").unwrap();
        assert_eq!(c.last_seen_remote_id, 0);
        r.update_sync_cursor("timings", 42, 1000).unwrap();
        let c2 = r.get_sync_cursor("timings").unwrap();
        assert_eq!(c2.last_seen_remote_id, 42);
    }

    #[test]
    fn _unused_ok() { let _ = AppError::Db("x".into()); }
}
