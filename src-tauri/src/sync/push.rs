use crate::db::repo::Repo;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TimingDto<'a> {
    local_id: i64,
    athlete_id: Option<i64>,
    course_id: i64,
    start_timestamp_ms: Option<i64>,
    finish_timestamp_ms: Option<i64>,
    status: &'a str,
    total_time_ms: Option<i64>,
    operator_id: &'a str,
}

#[derive(Serialize)]
struct PendingDto<'a> {
    local_id: i64,
    course_id: i64,
    finish_timestamp_ms: i64,
    operator_id: &'a str,
}

#[derive(Deserialize)]
pub struct PushAck {
    pub local_id: i64,
    pub remote_id: i64,
}

pub async fn push_timings(
    repo: &Repo,
    http: &reqwest::Client,
    base_url: &str,
    token: &str,
) -> AppResult<usize> {
    let batch = repo.fetch_unsynced_timings(50)?;
    if batch.is_empty() {
        return Ok(0);
    }
    let dtos: Vec<_> = batch.iter().map(|t| TimingDto {
        local_id: t.id,
        athlete_id: t.athlete_id,
        course_id: t.course_id,
        start_timestamp_ms: t.start_timestamp_ms,
        finish_timestamp_ms: t.finish_timestamp_ms,
        status: t.status.as_str(),
        total_time_ms: t.total_time_ms,
        operator_id: &t.operator_id,
    }).collect();
    let url = format!("{}/timings/batch", base_url.trim_end_matches('/'));
    let resp = http.post(&url).bearer_auth(token).json(&dtos).send().await?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Unauthorized);
    }
    let resp = resp.error_for_status().map_err(|e| AppError::Api(format!("push: {e}")))?;
    let acks: Vec<PushAck> = resp.json().await?;
    for ack in &acks {
        repo.mark_timing_synced(ack.local_id, ack.remote_id)?;
    }
    Ok(acks.len())
}

pub async fn push_pending(
    repo: &Repo,
    http: &reqwest::Client,
    base_url: &str,
    token: &str,
) -> AppResult<usize> {
    let batch = repo.fetch_unsynced_pending(50)?;
    if batch.is_empty() {
        return Ok(0);
    }
    let dtos: Vec<_> = batch.iter().map(|p| PendingDto {
        local_id: p.id,
        course_id: p.course_id,
        finish_timestamp_ms: p.finish_timestamp_ms,
        operator_id: &p.operator_id,
    }).collect();
    let url = format!("{}/pending_finishes/batch", base_url.trim_end_matches('/'));
    let resp = http.post(&url).bearer_auth(token).json(&dtos).send().await?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Unauthorized);
    }
    let resp = resp.error_for_status().map_err(|e| AppError::Api(format!("push pending: {e}")))?;
    let acks: Vec<PushAck> = resp.json().await?;
    for ack in &acks {
        repo.mark_pending_synced(ack.local_id, ack.remote_id)?;
    }
    Ok(acks.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrations, Db};
    use crate::models::{Athlete, Course};
    use std::sync::Arc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn push_marks_timings_synced() {
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
        let tid = repo.insert_timing_running(1, 1, 100, "PC-A").unwrap();
        repo.update_finish(tid, 200, 100).unwrap();

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/timings/batch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                { "local_id": tid, "remote_id": 99 }
            ])))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let n = push_timings(&repo, &http, &server.uri(), "AT").await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(repo.fetch_unsynced_timings(10).unwrap().len(), 0);
    }
}
