use crate::db::repo::Repo;
use crate::error::{AppError, AppResult};
use crate::models::{Timing, TimingStatus};
use serde::Deserialize;

#[derive(Deserialize)]
struct RemoteTiming {
    remote_id: i64,
    athlete_id: Option<i64>,
    course_id: i64,
    start_timestamp_ms: Option<i64>,
    finish_timestamp_ms: Option<i64>,
    status: String,
    total_time_ms: Option<i64>,
    operator_id: String,
}

pub async fn pull_timings(
    repo: &Repo,
    http: &reqwest::Client,
    base_url: &str,
    token: &str,
    now_ms: i64,
) -> AppResult<usize> {
    let cursor = repo.get_sync_cursor("timings")?;
    let url = format!(
        "{}/timings?since={}&limit=200",
        base_url.trim_end_matches('/'),
        cursor.last_seen_remote_id
    );
    let resp = http.get(&url).bearer_auth(token).send().await?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Unauthorized);
    }
    let resp = resp.error_for_status().map_err(|e| AppError::Api(format!("pull: {e}")))?;
    let items: Vec<RemoteTiming> = resp.json().await?;
    if items.is_empty() {
        return Ok(0);
    }
    let mut max_id = cursor.last_seen_remote_id;
    for r in &items {
        let t = Timing {
            id: 0,
            remote_id: Some(r.remote_id),
            athlete_id: r.athlete_id,
            course_id: r.course_id,
            start_timestamp_ms: r.start_timestamp_ms,
            finish_timestamp_ms: r.finish_timestamp_ms,
            status: TimingStatus::from_str(&r.status).unwrap_or(TimingStatus::Running),
            total_time_ms: r.total_time_ms,
            operator_id: r.operator_id.clone(),
            duplicate_group_id: None,
            duplicate_flagged: false,
            synced: true,
        };
        repo.upsert_remote_timing(&t)?;
        if r.remote_id > max_id {
            max_id = r.remote_id;
        }
    }
    repo.update_sync_cursor("timings", max_id, now_ms)?;
    Ok(items.len())
}
