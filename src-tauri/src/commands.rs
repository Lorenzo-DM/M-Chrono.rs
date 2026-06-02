use crate::app_ctx::AppCtx;
use crate::error::AppError;
use crate::models::{Athlete, Course, DeviceCodeResponse, PendingFinish, Timing};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
pub struct CourseSnapshot {
    pub id: i64,
    pub elapsed_ms: Option<i64>,
    pub finishers_count: i64,
    pub started: bool,
}

#[derive(Serialize, Clone)]
pub struct DisplaySnapshot {
    pub courses: Vec<CourseSnapshot>,
    pub now_ms: i64,
}

#[derive(Serialize, Clone)]
pub struct AthleteRow {
    pub athlete: Athlete,
    pub status: String,
    pub finish_ms: Option<i64>,
    pub total_ms: Option<i64>,
}

#[tauri::command]
pub async fn get_courses(ctx: State<'_, AppCtx>) -> Result<Vec<Course>, AppError> {
    let s = ctx.state.read().await;
    Ok(s.courses.values().cloned().collect())
}

#[tauri::command]
pub async fn poll_display(ctx: State<'_, AppCtx>) -> Result<DisplaySnapshot, AppError> {
    let s = ctx.state.read().await;
    let now_ms = ctx.clock.now_ms();
    let now_inst = ctx.clock.instant_now();
    let mut courses = Vec::new();
    for (id, course) in &s.courses {
        let elapsed_ms = s.course_clock_origin.get(id)
            .map(|origin| now_inst.duration_since(*origin).as_millis() as i64);
        let finishers_count = s.timings.values()
            .filter(|t| t.course_id == *id
                && matches!(t.status, crate::models::TimingStatus::Finished))
            .count() as i64;
        courses.push(CourseSnapshot {
            id: *id,
            elapsed_ms,
            finishers_count,
            started: course.started_at_ms.is_some(),
        });
    }
    courses.sort_by_key(|c| c.id);
    Ok(DisplaySnapshot { courses, now_ms })
}

#[tauri::command]
pub async fn get_athletes_by_course(
    ctx: State<'_, AppCtx>,
    course_id: i64,
) -> Result<Vec<AthleteRow>, AppError> {
    let s = ctx.state.read().await;
    let mut rows = Vec::new();
    for a in s.athletes_by_id.values().filter(|a| a.course_id == course_id) {
        let timing = s.timings_by_athlete.get(&a.id)
            .and_then(|ids| ids.iter().filter_map(|id| s.timings.get(id)).next());
        rows.push(AthleteRow {
            athlete: a.clone(),
            status: timing
                .map(|t| t.status.as_str().into())
                .unwrap_or_else(|| "Registered".into()),
            finish_ms: timing.and_then(|t| t.finish_timestamp_ms),
            total_ms: timing.and_then(|t| t.total_time_ms),
        });
    }
    rows.sort_by_key(|r| r.athlete.bib_number);
    Ok(rows)
}

#[tauri::command]
pub async fn get_pending_finishes(
    ctx: State<'_, AppCtx>,
    course_id: i64,
) -> Result<Vec<PendingFinish>, AppError> {
    let s = ctx.state.read().await;
    Ok(s.pending.iter().filter(|p| p.course_id == course_id).cloned().collect())
}

#[tauri::command]
pub async fn get_config(ctx: State<'_, AppCtx>) -> Result<crate::config::AppConfig, AppError> {
    Ok(ctx.config.read().await.clone())
}

async fn current_operator(ctx: &AppCtx) -> Result<String, AppError> {
    let cfg = ctx.config.read().await;
    if cfg.operator_id.is_empty() {
        return Err(AppError::InvalidState("operator_id not configured".into()));
    }
    Ok(cfg.operator_id.clone())
}

#[tauri::command]
pub async fn start_course(app: AppHandle, ctx: State<'_, AppCtx>, course_id: i64)
    -> Result<i64, AppError> {
    let op = current_operator(&ctx).await?;
    let ts = crate::timer::start_course(&ctx.state, &ctx.repo, &*ctx.clock, course_id, &op).await?;
    let _ = app.emit(
        "course:started",
        serde_json::json!({ "course_id": course_id, "started_at_ms": ts }),
    );
    Ok(ts)
}

#[tauri::command]
pub async fn finish_by_bib(app: AppHandle, ctx: State<'_, AppCtx>, bib: i64)
    -> Result<Timing, AppError> {
    let op = current_operator(&ctx).await?;
    let t = crate::timer::finish_by_bib(&ctx.state, &ctx.repo, &*ctx.clock, bib, &op).await?;
    let _ = app.emit("athlete:finished", &t);
    Ok(t)
}

#[tauri::command]
pub async fn finish_by_athlete_id(app: AppHandle, ctx: State<'_, AppCtx>, athlete_id: i64)
    -> Result<Timing, AppError> {
    let op = current_operator(&ctx).await?;
    let t = crate::timer::finish_by_athlete_id(&ctx.state, &ctx.repo, &*ctx.clock, athlete_id, &op).await?;
    let _ = app.emit("athlete:finished", &t);
    Ok(t)
}

#[tauri::command]
pub async fn capture_pending_finish(app: AppHandle, ctx: State<'_, AppCtx>, course_id: i64)
    -> Result<PendingFinish, AppError> {
    let op = current_operator(&ctx).await?;
    let p = crate::timer::capture_pending_finish(&ctx.state, &ctx.repo, &*ctx.clock, course_id, &op).await?;
    let _ = app.emit("pending:captured", &p);
    Ok(p)
}

#[tauri::command]
pub async fn assign_pending(app: AppHandle, ctx: State<'_, AppCtx>, pending_id: i64, bib: i64)
    -> Result<Timing, AppError> {
    let op = current_operator(&ctx).await?;
    let t = crate::timer::assign_pending(&ctx.state, &ctx.repo, pending_id, bib, &op).await?;
    let _ = app.emit("athlete:finished", &t);
    Ok(t)
}

#[tauri::command]
pub async fn withdraw_athlete(ctx: State<'_, AppCtx>, bib: i64) -> Result<(), AppError> {
    let op = current_operator(&ctx).await?;
    crate::timer::withdraw_athlete(&ctx.state, &ctx.repo, bib, &op).await
}

#[tauri::command]
pub async fn undo_finish(ctx: State<'_, AppCtx>, timing_id: i64) -> Result<(), AppError> {
    crate::timer::undo_finish(&ctx.state, &ctx.repo, timing_id).await
}

#[tauri::command]
pub async fn update_operator_id(ctx: State<'_, AppCtx>, id: String) -> Result<(), AppError> {
    let mut cfg = ctx.config.write().await;
    cfg.operator_id = id;
    cfg.save(&ctx.config_path)?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ConfigPatch {
    pub oidc_issuer_url: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_scopes: Option<String>,
    pub api_base_url: Option<String>,
    pub sync_interval_secs: Option<u64>,
    pub operator_id: Option<String>,
    pub dedup_window_ms: Option<i64>,
    pub dedup_warn_delta_ms: Option<i64>,
}

#[tauri::command]
pub async fn update_config(
    ctx: State<'_, AppCtx>,
    patch: ConfigPatch,
) -> Result<crate::config::AppConfig, AppError> {
    let mut cfg = ctx.config.write().await;
    if let Some(v) = patch.oidc_issuer_url   { cfg.oidc_issuer_url = v; }
    if let Some(v) = patch.oidc_client_id    { cfg.oidc_client_id = v; }
    if let Some(v) = patch.oidc_scopes       { cfg.oidc_scopes = v; }
    if let Some(v) = patch.api_base_url      { cfg.api_base_url = v; }
    if let Some(v) = patch.sync_interval_secs { cfg.sync_interval_secs = v; }
    if let Some(v) = patch.operator_id       { cfg.operator_id = v; }
    if let Some(v) = patch.dedup_window_ms   { cfg.dedup_window_ms = v; }
    if let Some(v) = patch.dedup_warn_delta_ms { cfg.dedup_warn_delta_ms = v; }
    cfg.save(&ctx.config_path)?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn start_device_login(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
) -> Result<DeviceCodeResponse, AppError> {
    let cfg = ctx.config.read().await.clone();
    let resp = ctx.auth.start_device_login(&cfg).await?;
    let interval = resp.interval;
    let auth = ctx.auth.clone();
    let cfg_clone = cfg.clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        match auth.poll_until_authorized(&cfg_clone, interval).await {
            Ok(_) => {
                let _ = app_clone.emit("auth:success", ());
            }
            Err(e) => {
                let _ = app_clone.emit(
                    "auth:failed",
                    serde_json::json!({ "reason": e.to_string() }),
                );
            }
        }
    });
    Ok(resp)
}

#[tauri::command]
pub fn is_authenticated(ctx: State<'_, AppCtx>) -> bool {
    ctx.auth.is_authenticated()
}

#[tauri::command]
pub async fn logout(app: AppHandle, ctx: State<'_, AppCtx>) -> Result<(), AppError> {
    ctx.auth.logout()?;
    let _ = app.emit("auth:logged_out", ());
    Ok(())
}

#[derive(Serialize)]
pub struct FetchSummary {
    pub courses_count: usize,
    pub athletes_count: usize,
}

#[tauri::command]
pub async fn fetch_remote_data(ctx: State<'_, AppCtx>) -> Result<FetchSummary, AppError> {
    let cfg = ctx.config.read().await.clone();
    let token = ctx.auth.get_access_token(&cfg).await?;
    let courses = crate::api::fetch::fetch_courses(&ctx.http, &cfg.api_base_url, &token).await?;
    let athletes = crate::api::fetch::fetch_athletes(&ctx.http, &cfg.api_base_url, &token).await?;
    for c in &courses {
        ctx.repo.upsert_course(c)?;
    }
    for a in &athletes {
        ctx.repo.upsert_athlete(a)?;
    }
    // refresh in-memory state from DB
    let snap = crate::state::bootstrap_from_db(&ctx.repo, &*ctx.clock)?;
    *ctx.state.write().await = snap;
    Ok(FetchSummary {
        courses_count: courses.len(),
        athletes_count: athletes.len(),
    })
}

#[tauri::command]
pub async fn get_duplicate_groups(
    ctx: State<'_, AppCtx>,
) -> Result<Vec<crate::db::repo::DuplicateGroup>, AppError> {
    let repo = ctx.repo.clone();
    let groups = tokio::task::spawn_blocking(move || repo.list_duplicate_groups())
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(groups)
}

#[tauri::command]
pub async fn export_results_xlsx(
    ctx: State<'_, AppCtx>,
    path: String,
) -> Result<crate::export::xlsx::ExportSummary, AppError> {
    let repo = ctx.repo.clone();
    let p = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || crate::export::xlsx::write_results(&repo, &p))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}
