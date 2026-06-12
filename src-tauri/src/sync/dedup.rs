use crate::db::repo::Repo;
use crate::error::AppResult;
use uuid::Uuid;

/// Group Finished timings for `athlete_id` into duplicate groups using a sliding window.
/// Returns the list of new group_ids created (only multi-record groups).
pub fn group_finishes(
    repo: &Repo,
    athlete_id: i64,
    window_ms: i64,
    warn_delta_ms: i64,
) -> AppResult<Vec<String>> {
    let finishes = repo.list_finished_timings_for_athlete(athlete_id)?;
    if finishes.is_empty() {
        return Ok(vec![]);
    }
    let mut sorted: Vec<(i64, i64)> = finishes
        .into_iter()
        .filter_map(|t| t.finish_timestamp_ms.map(|ts| (ts, t.id)))
        .collect();
    sorted.sort_by_key(|(ts, _)| *ts);

    let mut groups: Vec<Vec<(i64, i64)>> = Vec::new();
    let mut current: Vec<(i64, i64)> = Vec::new();
    for entry in sorted {
        if current.is_empty() {
            current.push(entry);
        } else {
            let last_ts = current.last().unwrap().0;
            if entry.0 - last_ts <= window_ms {
                current.push(entry);
            } else {
                groups.push(std::mem::take(&mut current));
                current.push(entry);
            }
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }

    let mut new_group_ids = Vec::new();
    for g in groups {
        if g.len() < 2 {
            // singleton — clear any prior grouping for this id
            repo.set_duplicate_group(&[g[0].1], "", false)?;
            continue;
        }
        let delta = g.last().unwrap().0 - g.first().unwrap().0;
        let group_id = Uuid::new_v4().to_string();
        let flagged = delta > warn_delta_ms;
        let timing_ids: Vec<i64> = g.iter().map(|(_, id)| *id).collect();
        repo.set_duplicate_group(&timing_ids, &group_id, flagged)?;
        new_group_ids.push(group_id);
    }
    Ok(new_group_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::models::{Athlete, Course};
    use std::sync::Arc;

    fn fresh() -> Arc<Repo> {
        let db = Db::open_in_memory().unwrap();
        migrations::run(&db.conn.lock().unwrap()).unwrap();
        let repo = Arc::new(Repo::new(db.conn.clone()));
        repo.upsert_course(&Course {
            id: 1, name: "x".into(), distance_m: None,
            started_at_ms: None, scheduled_at_ms: None, ended_at_ms: None, race_id: None,
        }).unwrap();
        repo.upsert_athlete(&Athlete {
            id: 1, bib_number: 1, first_name: "a".into(),
            last_name: "b".into(), course_id: 1,
        }).unwrap();
        repo
    }

    fn add_finish(repo: &Repo, op: &str, ts: i64) {
        let id = repo.insert_timing_running(1, 1, 0, op).unwrap();
        repo.update_finish(id, ts, ts).unwrap();
    }

    #[test]
    fn single_finish_no_group() {
        let repo = fresh();
        add_finish(&repo, "PC-A", 1000);
        let groups = group_finishes(&repo, 1, 2000, 500).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn two_close_finishes_grouped_unflagged() {
        let repo = fresh();
        add_finish(&repo, "PC-A", 1000);
        add_finish(&repo, "PC-B", 1300);
        let groups = group_finishes(&repo, 1, 2000, 500).unwrap();
        assert_eq!(groups.len(), 1);
        let timings = repo.list_finished_timings_for_athlete(1).unwrap();
        assert!(timings.iter().all(|t| !t.duplicate_flagged));
    }

    #[test]
    fn flagged_when_delta_exceeds_warn() {
        let repo = fresh();
        add_finish(&repo, "PC-A", 1000);
        add_finish(&repo, "PC-B", 1800);
        group_finishes(&repo, 1, 2000, 500).unwrap();
        let timings = repo.list_finished_timings_for_athlete(1).unwrap();
        assert!(timings.iter().all(|t| t.duplicate_flagged));
    }

    #[test]
    fn distant_finishes_separate_groups() {
        let repo = fresh();
        add_finish(&repo, "PC-A", 1000);
        add_finish(&repo, "PC-B", 5000);
        group_finishes(&repo, 1, 2000, 500).unwrap();
        let timings = repo.list_finished_timings_for_athlete(1).unwrap();
        // distant => two singleton groups, no dup_group set, not flagged
        assert!(timings.iter().all(|t| !t.duplicate_flagged));
    }
}
