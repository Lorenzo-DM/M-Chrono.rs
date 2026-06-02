use crate::error::{AppError, AppResult};
use crate::models::{Athlete, Course};
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
    fn _unused_ok() { let _ = AppError::Db("x".into()); }
}
