pub mod clock;

use crate::db::repo::Repo;
use crate::error::{AppError, AppResult};
use crate::models::{Timing, TimingStatus};
use crate::state::{RaceState, SharedState};
use crate::timer::clock::ClockProvider;
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
    let athlete = s.athletes_by_bib.get(&bib).cloned()
        .ok_or_else(|| AppError::NotFound(format!("bib {} not found", bib)))?;
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

use crate::models::PendingFinish;

pub async fn capture_pending_finish(
    state: &SharedState,
    repo: &Repo,
    clock: &dyn ClockProvider,
    course_id: i64,
    operator_id: &str,
) -> AppResult<PendingFinish> {
    let ts_ms = clock.now_ms();
    let p = repo.insert_pending_finish(course_id, ts_ms, operator_id)?;
    state.write().await.pending.push(p.clone());
    Ok(p)
}

pub async fn capture_pending_tie(
    state: &SharedState,
    repo: &Repo,
    course_id: i64,
    operator_id: &str,
) -> AppResult<PendingFinish> {
    let reference_ts = {
        let s = state.read().await;
        s.pending.iter()
            .filter(|p| p.course_id == course_id && !p.assigned)
            .map(|p| p.finish_timestamp_ms)
            .max()
            .ok_or_else(|| AppError::InvalidState(
                "nessun arrivo recente da agganciare per il tie".into()
            ))?
    };
    let p = repo.insert_pending_finish(course_id, reference_ts, operator_id)?;
    state.write().await.pending.push(p.clone());
    Ok(p)
}

pub async fn assign_pending(
    state: &SharedState,
    repo: &Repo,
    pending_id: i64,
    bib: i64,
    operator_id: &str,
) -> AppResult<Timing> {
    let mut s = state.write().await;
    let pending = s.pending.iter().find(|p| p.id == pending_id).cloned()
        .ok_or_else(|| AppError::NotFound(format!("pending {}", pending_id)))?;
    let athlete = s.athletes_by_bib.get(&bib).cloned()
        .ok_or_else(|| AppError::NotFound(format!("bib {}", bib)))?;
    let timing = repo.find_running_timing_for_athlete(athlete.id, operator_id)?
        .ok_or_else(|| AppError::InvalidState(
            format!("no running timing for athlete {}", athlete.id)))?;
    let start = timing.start_timestamp_ms.ok_or_else(||
        AppError::InvalidState("timing has no start".into()))?;
    let total = pending.finish_timestamp_ms - start;
    repo.update_finish(timing.id, pending.finish_timestamp_ms, total)?;
    repo.mark_pending_assigned(pending_id)?;
    let updated = repo.get_timing(timing.id)?.expect("updated");
    s.timings.insert(updated.id, updated.clone());
    s.pending.retain(|p| p.id != pending_id);
    Ok(updated)
}

pub async fn withdraw_athlete(
    state: &SharedState,
    repo: &Repo,
    bib: i64,
    operator_id: &str,
) -> AppResult<()> {
    let mut s = state.write().await;
    let athlete = s.athletes_by_bib.get(&bib).cloned()
        .ok_or_else(|| AppError::NotFound(format!("bib {}", bib)))?;
    let timing = repo.find_running_timing_for_athlete(athlete.id, operator_id)?
        .ok_or_else(|| AppError::InvalidState("no running timing".into()))?;
    repo.update_status(timing.id, TimingStatus::Withdrawn)?;
    let updated = repo.get_timing(timing.id)?.expect("updated");
    s.timings.insert(updated.id, updated);
    Ok(())
}

pub async fn undo_finish(
    state: &SharedState,
    repo: &Repo,
    timing_id: i64,
) -> AppResult<()> {
    let mut s = state.write().await;
    repo.undo_finish(timing_id)?;
    let updated = repo.get_timing(timing_id)?.ok_or_else(|| AppError::NotFound("timing".into()))?;
    s.timings.insert(updated.id, updated);
    Ok(())
}

pub async fn delete_pending_finish(
    state: &SharedState,
    repo: &Repo,
    pending_id: i64,
) -> AppResult<()> {
    let mut s = state.write().await;
    repo.delete_pending_finish(pending_id)?;
    s.pending.retain(|p| p.id != pending_id);
    Ok(())
}

pub async fn restart_course(
    state: &SharedState,
    repo: &Repo,
    course_id: i64,
) -> AppResult<()> {
    let mut s = state.write().await;
    if !s.courses.contains_key(&course_id) {
        return Err(AppError::NotFound(format!("course {}", course_id)));
    }
    repo.restart_course(course_id)?;
    if let Some(c) = s.courses.get_mut(&course_id) {
        c.started_at_ms = None;
        c.ended_at_ms = None;
    }
    s.course_clock_origin.remove(&course_id);
    let to_remove: Vec<i64> = s.timings.values()
        .filter(|t| t.course_id == course_id)
        .map(|t| t.id).collect();
    for id in &to_remove { s.timings.remove(id); }
    for v in s.timings_by_athlete.values_mut() {
        v.retain(|id| !to_remove.contains(id));
    }
    s.pending.retain(|p| p.course_id != course_id);
    Ok(())
}

pub async fn reassign_bib(
    state: &SharedState,
    repo: &Repo,
    timing_id: i64,
    new_bib: i64,
    operator_id: &str,
) -> AppResult<Timing> {
    let mut s = state.write().await;

    let old = s.timings.get(&timing_id).cloned()
        .ok_or_else(|| AppError::NotFound(format!("timing {}", timing_id)))?;
    if !matches!(old.status, TimingStatus::Finished) {
        return Err(AppError::InvalidState("timing non è in stato Finished".into()));
    }
    let finish_ts = old.finish_timestamp_ms
        .ok_or_else(|| AppError::InvalidState("timing privo di finish".into()))?;

    let new_athlete = s.athletes_by_bib.get(&new_bib).cloned()
        .ok_or_else(|| AppError::NotFound(format!("pettorale {} non trovato", new_bib)))?;
    if Some(new_athlete.id) == old.athlete_id {
        return Err(AppError::InvalidState("pettorale identico, nessuna modifica".into()));
    }
    if new_athlete.course_id != old.course_id {
        return Err(AppError::InvalidState(
            format!("pettorale {} non appartiene a questo percorso", new_bib)
        ));
    }

    let target = repo.find_running_timing_for_athlete(new_athlete.id, operator_id)?
        .ok_or_else(|| AppError::InvalidState(
            format!("pettorale {} non ha un timing in corso", new_bib)
        ))?;
    let start = target.start_timestamp_ms
        .ok_or_else(|| AppError::InvalidState("nuovo timing privo di start".into()))?;
    let total = finish_ts - start;

    repo.undo_finish(timing_id)?;
    repo.update_finish(target.id, finish_ts, total)?;

    let reverted = repo.get_timing(timing_id)?.expect("reverted");
    let updated = repo.get_timing(target.id)?.expect("updated");
    s.timings.insert(reverted.id, reverted);
    s.timings.insert(updated.id, updated.clone());
    Ok(updated)
}

pub async fn end_course(
    state: &SharedState,
    repo: &Repo,
    clock: &dyn ClockProvider,
    course_id: i64,
) -> AppResult<i64> {
    let ts_ms = clock.now_ms();
    let mut s = state.write().await;
    let course = s.courses.get(&course_id).cloned()
        .ok_or_else(|| AppError::NotFound(format!("course {}", course_id)))?;
    if course.started_at_ms.is_none() {
        return Err(AppError::InvalidState(
            format!("course {} not started", course_id)
        ));
    }
    if course.ended_at_ms.is_some() {
        return Err(AppError::InvalidState(
            format!("course {} already ended", course_id)
        ));
    }
    repo.end_course(course_id, ts_ms)?;
    if let Some(c) = s.courses.get_mut(&course_id) { c.ended_at_ms = Some(ts_ms); }
    Ok(ts_ms)
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
    async fn finish_unknown_bib_errors() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        clock.advance(3_000);
        let err = finish_by_bib(&state, &repo, &*clock, 999, "PC-A").await.unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
        let s = state.read().await;
        assert_eq!(s.pending.len(), 0);
    }

    #[tokio::test]
    async fn finish_when_not_running_errors() {
        let (state, repo, clock) = setup().await;
        let err = finish_by_bib(&state, &repo, &*clock, 1, "PC-A").await.unwrap_err();
        assert!(matches!(err, AppError::InvalidState(_)));
    }
}

#[cfg(test)]
mod tests_pending {
    use super::*;
    use super::tests::setup;

    #[tokio::test]
    async fn capture_and_assign_pending() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        clock.advance(4_000);
        let p = capture_pending_finish(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        let t = assign_pending(&state, &repo, p.id, 2, "PC-A").await.unwrap();
        assert_eq!(t.total_time_ms, Some(4_000));
        assert!(state.read().await.pending.is_empty());
    }

    #[tokio::test]
    async fn withdraw_changes_status() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        withdraw_athlete(&state, &repo, 1, "PC-A").await.unwrap();
        let s = state.read().await;
        let t = s.timings.values().find(|t| t.athlete_id == Some(1)).unwrap();
        assert_eq!(t.status, TimingStatus::Withdrawn);
    }

    #[tokio::test]
    async fn undo_finish_reverts_to_running() {
        let (state, repo, clock) = setup().await;
        start_course(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        clock.advance(1_000);
        let t = finish_by_bib(&state, &repo, &*clock, 1, "PC-A").await.unwrap();
        undo_finish(&state, &repo, t.id).await.unwrap();
        let after = state.read().await.timings.get(&t.id).cloned().unwrap();
        assert_eq!(after.status, TimingStatus::Running);
        assert!(after.finish_timestamp_ms.is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::models::{Athlete, Course};
    use crate::state::{bootstrap_from_db, new_shared};
    use crate::timer::clock::MockClock;
    use std::sync::Arc;

    pub async fn setup() -> (SharedState, Arc<Repo>, Arc<MockClock>) {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let clock = Arc::new(MockClock::new(1_000_000));
        let repo = Arc::new(Repo::with_clock(db.conn.clone(), clock.clone()));
        repo.upsert_course(&Course { id: 1, name: "x".into(), distance_m: None,
                                      started_at_ms: None, scheduled_at_ms: None, ended_at_ms: None }).unwrap();
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
