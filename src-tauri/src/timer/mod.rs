pub mod clock;

use crate::db::repo::Repo;
use crate::error::{AppError, AppResult};
use crate::models::{Course, Timing, TimingStatus};
use crate::state::{RaceState, SharedState};
use crate::timer::clock::ClockProvider;
use std::time::Duration;

pub async fn start_course(
    state: &SharedState,
    repo: &Repo,
    clock: &dyn ClockProvider,
    course_id: i64,
    operator_id: &str,
) -> AppResult<i64> {
    let ts_ms = clock.now_ms();
    let instant = clock.instant_now();

    let mut s = state.write().await;
    let course = s.courses.get(&course_id).cloned()
        .ok_or_else(|| AppError::NotFound(format!("course {}", course_id)))?;
    if course.started_at_ms.is_some() {
        return Err(AppError::InvalidState("course already started".into()));
    }

    let athlete_ids: Vec<i64> = s.athletes_by_id.values()
        .filter(|a| a.course_id == course_id)
        .map(|a| a.id).collect();

    let timing_ids = repo.insert_timings_bulk(course_id, &athlete_ids, ts_ms, operator_id)?;

    // refresh memory
    if let Some(c) = s.courses.get_mut(&course_id) { c.started_at_ms = Some(ts_ms); }
    s.course_clock_origin.insert(course_id, instant);
    for (i, tid) in timing_ids.iter().enumerate() {
        let aid = athlete_ids[i];
        let t = Timing {
            id: *tid, remote_id: None, athlete_id: Some(aid), course_id,
            start_timestamp_ms: Some(ts_ms), finish_timestamp_ms: None,
            status: TimingStatus::Running, total_time_ms: None,
            operator_id: operator_id.into(),
            duplicate_group_id: None, duplicate_flagged: false, synced: false,
        };
        s.timings_by_athlete.entry(aid).or_default().push(*tid);
        s.timings.insert(*tid, t);
    }
    Ok(ts_ms)
}

pub async fn finish_by_bib(
    state: &SharedState,
    repo: &Repo,
    clock: &dyn ClockProvider,
    bib: i64,
    operator_id: &str,
) -> AppResult<Timing> {
    let ts_ms = clock.now_ms();
    let mut s = state.write().await;
    let athlete = match s.athletes_by_bib.get(&bib).cloned() {
        Some(a) => a,
        None => {
            // capture as pending automatically
            let course_default = s.courses.keys().next().copied().unwrap_or(0);
            drop(s);
            let p = repo.insert_pending_finish(course_default, ts_ms, operator_id)?;
            state.write().await.pending.push(p);
            return Err(AppError::NotFound(format!("bib {} not found (saved as pending)", bib)));
        }
    };
    finish_athlete_inner(&mut s, repo, ts_ms, athlete.id, operator_id)
}

pub async fn finish_by_athlete_id(
    state: &SharedState,
    repo: &Repo,
    clock: &dyn ClockProvider,
    athlete_id: i64,
    operator_id: &str,
) -> AppResult<Timing> {
    let ts_ms = clock.now_ms();
    let mut s = state.write().await;
    finish_athlete_inner(&mut s, repo, ts_ms, athlete_id, operator_id)
}

fn finish_athlete_inner(
    s: &mut RaceState,
    repo: &Repo,
    ts_ms: i64,
    athlete_id: i64,
    operator_id: &str,
) -> AppResult<Timing> {
    let timing = repo.find_running_timing_for_athlete(athlete_id, operator_id)?
        .ok_or_else(|| AppError::InvalidState(
            format!("no running timing for athlete {} on operator {}", athlete_id, operator_id)))?;
    let start = timing.start_timestamp_ms.ok_or_else(||
        AppError::InvalidState("timing has no start".into()))?;
    let total = ts_ms - start;
    repo.update_finish(timing.id, ts_ms, total)?;
    let updated = repo.get_timing(timing.id)?.expect("just updated");
    s.timings.insert(updated.id, updated.clone());
    Ok(updated)
}

#[cfg(test)]
mod tests_finish {
    use super::*;
    use super::tests::setup;

    #[tokio::test]
    async fn finish_by_bib_computes_total() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        clock.advance(5_000);
        let t = finish_by_bib(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        assert_eq!(t.total_time_ms, Some(5_000));
        assert_eq!(t.status, TimingStatus::Finished);
    }

    #[tokio::test]
    async fn finish_unknown_bib_saves_pending() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        clock.advance(3_000);
        let err = finish_by_bib(&state, &repo, &*clock, 999, "PC-A").await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
        let s = state.read().await;
        assert_eq!(s.pending.len(), 1);
    }

    #[tokio::test]
    async fn finish_when_not_running_errors() {
        let (state, repo, clock) = setup().await;
        let err = finish_by_bib(&state, &repo, &*clock, 1, "PC-A").await.unwrap_err();
        assert!(matches!(err, AppError::InvalidState(_)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::models::Athlete;
    use crate::state::{bootstrap_from_db, new_shared};
    use crate::timer::clock::MockClock;
    use std::sync::Arc;

    pub async fn setup() -> (SharedState, Arc<Repo>, Arc<MockClock>) {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let clock = Arc::new(MockClock::new(1_000_000));
        let repo = Arc::new(Repo::with_clock(db.conn.clone(), clock.clone()));
        repo.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                      started_at_ms: None, scheduled_at_ms: None }).unwrap();
        for i in 1..=3 {
            repo.upsert_athlete(&Athlete {
                id: i, bib_number: i, first_name: "a".into(), last_name: "b".into(), course_id: 1
            }).unwrap();
        }
        let state = new_shared();
        let snap = bootstrap_from_db(&repo, &*clock).unwrap();
        *state.write().await = snap;
        (state, repo, clock)
    }

    #[tokio::test]
    async fn start_course_creates_timings_for_all_athletes() {
        let (state, repo, clock) = setup().await;
        let ts = start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        assert_eq!(ts, 1_000_000);
        let s = state.read().await;
        assert_eq!(s.timings.len(), 3);
        assert!(s.course_clock_origin.contains_key(&1));
    }

    #[tokio::test]
    async fn start_course_twice_errors() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        let err = start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap_err();
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[tokio::test]
    async fn start_course_unknown_course_errors() {
        let (state, repo, clock) = setup().await;
        let err = start_course(&state, &repo, &*clock, 99, "PC-A").await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
