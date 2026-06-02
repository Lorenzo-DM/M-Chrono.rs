use crate::error::{AppError, AppResult};
use crate::models::{Athlete, Course, PendingFinish, Timing, TimingStatus};
use crate::timer::clock::{ClockProvider, SystemClock};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

pub struct Repo {
    pub conn: Arc<Mutex<Connection>>,
    pub clock: Arc<dyn ClockProvider>,
}

impl Repo {
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
    fn _unused_ok() { let _ = AppError::Db("x".into()); }
}
