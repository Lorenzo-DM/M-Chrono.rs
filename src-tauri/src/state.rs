use crate::db::repo::Repo;
use crate::error::AppResult;
use crate::models::{Athlete, Course, PendingFinish, Timing};
use crate::timer::clock::ClockProvider;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct RaceState {
    pub courses: HashMap<i64, Course>,
    pub athletes_by_id: HashMap<i64, Athlete>,
    pub athletes_by_bib: HashMap<i64, Athlete>,
    pub timings: HashMap<i64, Timing>,
    pub timings_by_athlete: HashMap<i64, Vec<i64>>,
    pub pending: Vec<PendingFinish>,
    pub course_clock_origin: HashMap<i64, Instant>,
}

pub type SharedState = Arc<RwLock<RaceState>>;

pub fn new_shared() -> SharedState {
    Arc::new(RwLock::new(RaceState::default()))
}

pub fn bootstrap_from_db(repo: &Repo, clock: &dyn ClockProvider) -> AppResult<RaceState> {
    let mut s = RaceState::default();
    for c in repo.list_courses()? {
        if let Some(started) = c.started_at_ms {
            let now = clock.now_ms();
            let elapsed = (now - started).max(0) as u64;
            let inst = clock.instant_now().checked_sub(Duration::from_millis(elapsed))
                .unwrap_or_else(|| clock.instant_now());
            s.course_clock_origin.insert(c.id, inst);
        }
        s.courses.insert(c.id, c);
    }
    for a in repo.list_athletes()? {
        s.athletes_by_id.insert(a.id, a.clone());
        s.athletes_by_bib.insert(a.bib_number, a);
    }
    for t in repo.list_all_timings()? {
        if let Some(aid) = t.athlete_id {
            s.timings_by_athlete.entry(aid).or_default().push(t.id);
        }
        s.timings.insert(t.id, t);
    }
    s.pending = repo.list_all_pending_open()?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::db::repo::Repo;
    use crate::models::{Athlete, Course};
    use crate::timer::clock::SystemClock;

    #[test]
    fn bootstrap_empty_returns_empty_state() {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let repo = Repo::new(db.conn.clone());
        let s = bootstrap_from_db(&repo, &SystemClock).unwrap();
        assert!(s.courses.is_empty());
        assert!(s.athletes_by_id.is_empty());
    }

    #[test]
    fn bootstrap_populates_lookups() {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let repo = Repo::new(db.conn.clone());
        repo.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                      started_at_ms: None, scheduled_at_ms: None }).unwrap();
        repo.upsert_athlete(&Athlete { id: 10, bib_number: 7, first_name: "a".into(),
                                        last_name: "b".into(), course_id: 1 }).unwrap();
        let s = bootstrap_from_db(&repo, &SystemClock).unwrap();
        assert!(s.athletes_by_bib.contains_key(&7));
        assert!(s.courses.contains_key(&1));
    }
}
