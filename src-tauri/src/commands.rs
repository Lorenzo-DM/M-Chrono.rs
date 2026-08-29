use crate::app_ctx::AppCtx;
use crate::error::AppError;
use crate::models::{Athlete, Course, DeviceCodeResponse, PendingFinish, Race, Timing};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
pub struct CourseSnapshot {
    pub id: i64,
    pub elapsed_ms: Option<i64>,
    pub finishers_count: i64,
    pub started: bool,
    pub ended: bool,
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
    pub timing_id: Option<i64>,
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
        let elapsed_ms = match (course.started_at_ms, course.ended_at_ms) {
            (Some(start), Some(end)) => Some((end - start).max(0)),
            (Some(_), None) => s.course_clock_origin.get(id)
                .map(|origin| now_inst.duration_since(*origin).as_millis() as i64),
            _ => None,
        };
        let finishers_count = s.timings.values()
            .filter(|t| t.course_id == *id
                && matches!(t.status, crate::models::TimingStatus::Finished))
            .count() as i64;
        courses.push(CourseSnapshot {
            id: *id,
            elapsed_ms,
            finishers_count,
            started: course.started_at_ms.is_some(),
            ended: course.ended_at_ms.is_some(),
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
            timing_id: timing.map(|t| t.id),
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
pub async fn capture_pending_tie(app: AppHandle, ctx: State<'_, AppCtx>, course_id: i64)
    -> Result<PendingFinish, AppError> {
    let op = current_operator(&ctx).await?;
    let p = crate::timer::capture_pending_tie(&ctx.state, &ctx.repo, course_id, &op).await?;
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
pub async fn delete_pending_finish(ctx: State<'_, AppCtx>, pending_id: i64) -> Result<(), AppError> {
    crate::timer::delete_pending_finish(&ctx.state, &ctx.repo, pending_id).await
}

#[tauri::command]
pub async fn move_pending_to_course(app: AppHandle, ctx: State<'_, AppCtx>, pending_id: i64, target_course_id: i64) -> Result<(), AppError> {
    let p = crate::timer::move_pending_to_course(&ctx.state, &ctx.repo, pending_id, target_course_id).await?;
    let _ = app.emit("pending:captured", &p);
    Ok(())
}

#[tauri::command]
pub async fn reassign_bib(app: AppHandle, ctx: State<'_, AppCtx>, timing_id: i64, new_bib: i64)
    -> Result<Timing, AppError> {
    let op = current_operator(&ctx).await?;
    let t = crate::timer::reassign_bib(&ctx.state, &ctx.repo, timing_id, new_bib, &op).await?;
    let _ = app.emit("athlete:finished", &t);
    Ok(t)
}

#[tauri::command]
pub async fn end_course(app: AppHandle, ctx: State<'_, AppCtx>, course_id: i64, confirm_name: String)
    -> Result<i64, AppError> {
    let expected = {
        let s = ctx.state.read().await;
        s.courses.get(&course_id)
            .ok_or_else(|| AppError::NotFound(format!("course {}", course_id)))?
            .name.clone()
    };
    if confirm_name.trim() != expected {
        return Err(AppError::InvalidState(
            "nome del percorso non corrisponde".into()
        ));
    }
    let ts = crate::timer::end_course(&ctx.state, &ctx.repo, &*ctx.clock, course_id).await?;
    let _ = app.emit("course:ended", serde_json::json!({
        "course_id": course_id,
        "ended_at_ms": ts,
    }));
    Ok(ts)
}

#[tauri::command]
pub async fn restart_course(app: AppHandle, ctx: State<'_, AppCtx>, course_id: i64, confirm_name: String)
    -> Result<(), AppError> {
    let expected = {
        let s = ctx.state.read().await;
        s.courses.get(&course_id)
            .ok_or_else(|| AppError::NotFound(format!("course {}", course_id)))?
            .name.clone()
    };
    if confirm_name.trim() != expected {
        return Err(AppError::InvalidState(
            "nome del percorso non corrisponde".into()
        ));
    }
    crate::timer::restart_course(&ctx.state, &ctx.repo, course_id).await?;
    let _ = app.emit("course:reset", serde_json::json!({
        "course_id": course_id,
    }));
    Ok(())
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
    pub sync_enabled: Option<bool>,
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
    if let Some(v) = patch.sync_enabled        { cfg.sync_enabled = v; }
    cfg.save(&ctx.config_path)?;
    // The provider may have changed: drop the cached discovery document.
    ctx.auth.invalidate_endpoints().await;
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

async fn refresh_state(ctx: &AppCtx) -> Result<(), AppError> {
    let snap = crate::state::bootstrap_from_db(&ctx.repo, &*ctx.clock)?;
    *ctx.state.write().await = snap;
    Ok(())
}

#[tauri::command]
pub async fn get_races(ctx: State<'_, AppCtx>) -> Result<Vec<Race>, AppError> {
    ctx.repo.list_races()
}

#[derive(serde::Deserialize)]
pub struct RaceInput {
    pub name: String,
    pub scheduled_at_ms: Option<i64>,
}

#[tauri::command]
pub async fn save_race(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: Option<i64>,
    input: RaceInput,
) -> Result<Race, AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::InvalidState("nome gara obbligatorio".into()));
    }
    let race = Race {
        id: match id {
            Some(v) => v,
            None => ctx.repo.next_local_race_id()?,
        },
        name,
        scheduled_at_ms: input.scheduled_at_ms,
    };
    ctx.repo.upsert_race(&race)?;
    let _ = app.emit("data:changed", ());
    Ok(race)
}

#[tauri::command]
pub async fn delete_race(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: i64,
) -> Result<(), AppError> {
    ctx.repo.delete_race(id)?;
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct CourseInput {
    pub name: String,
    pub race_id: Option<i64>,
    #[serde(default)]
    pub distance_m: Option<i64>,
}

#[tauri::command]
pub async fn save_course(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: Option<i64>,
    input: CourseInput,
) -> Result<Course, AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::InvalidState("nome percorso obbligatorio".into()));
    }
    let distance_m = input.distance_m.filter(|d| *d > 0);
    let course = match id {
        Some(cid) => {
            ctx.repo.update_course(cid, &name, input.race_id, distance_m)?;
            ctx.repo
                .list_courses()?
                .into_iter()
                .find(|c| c.id == cid)
                .ok_or_else(|| AppError::NotFound(format!("course {}", cid)))?
        }
        None => {
            let course = Course {
                id: ctx.repo.next_local_course_id()?,
                name,
                distance_m,
                started_at_ms: None,
                scheduled_at_ms: None,
                ended_at_ms: None,
                race_id: input.race_id,
            };
            ctx.repo.upsert_course(&course)?;
            course
        }
    };
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(course)
}

#[tauri::command]
pub async fn delete_course(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: i64,
) -> Result<(), AppError> {
    ctx.repo.delete_course(id)?;
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(())
}

#[tauri::command]
pub async fn import_athletes_file(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    path: String,
) -> Result<crate::import::ImportSummary, AppError> {
    let repo = ctx.repo.clone();
    let p = std::path::PathBuf::from(path);
    let summary = tokio::task::spawn_blocking(move || crate::import::import_file(&repo, &p))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(summary)
}

#[derive(serde::Deserialize)]
pub struct AthleteInput {
    pub bib_number: i64,
    pub first_name: String,
    pub last_name: String,
    pub course_id: Option<i64>,
    pub course_name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub anonymous: bool,
}

#[tauri::command]
pub async fn save_athlete(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: Option<i64>,
    input: AthleteInput,
) -> Result<Athlete, AppError> {
    if input.bib_number <= 0 {
        return Err(AppError::InvalidState("pettorale deve essere positivo".into()));
    }
    let first = input.first_name.trim().to_string();
    let last = input.last_name.trim().to_string();
    // Anonymous athletes (free bib entry) carry no name on purpose.
    if !input.anonymous && first.is_empty() && last.is_empty() {
        return Err(AppError::InvalidState("nome e cognome mancanti".into()));
    }

    let course_id = match (input.course_id, input.course_name.as_deref()) {
        (Some(cid), _) => cid,
        (None, Some(name)) if !name.trim().is_empty() => {
            crate::import::get_or_create_course(&ctx.repo, name)?.0
        }
        _ => return Err(AppError::InvalidState("percorso mancante".into())),
    };

    if let Some(existing) = ctx.repo.find_athlete_by_bib(input.bib_number)? {
        if Some(existing.id) != id {
            return Err(AppError::InvalidState(format!(
                "pettorale {} già assegnato a {} {}",
                input.bib_number, existing.first_name, existing.last_name
            )));
        }
    }

    let category = input.category
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty());
    let athlete = Athlete {
        id: match id {
            Some(v) => v,
            None => ctx.repo.next_local_athlete_id()?,
        },
        bib_number: input.bib_number,
        first_name: first,
        last_name: last,
        course_id,
        category,
        anonymous: input.anonymous,
    };
    ctx.repo.upsert_athlete(&athlete)?;
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(athlete)
}

#[tauri::command]
pub async fn delete_athlete(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: i64,
) -> Result<(), AppError> {
    ctx.repo.delete_athlete(id)?;
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(())
}

#[tauri::command]
pub async fn get_all_athletes(ctx: State<'_, AppCtx>) -> Result<Vec<Athlete>, AppError> {
    ctx.repo.list_athletes()
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

#[tauri::command]
pub async fn export_results_csv(
    ctx: State<'_, AppCtx>,
    path: String,
) -> Result<crate::export::csv::ExportSummary, AppError> {
    let repo = ctx.repo.clone();
    let p = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || crate::export::csv::write_results(&repo, &p))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_results_by_course(
    ctx: State<'_, AppCtx>,
    course_id: i64,
) -> Result<Vec<crate::db::repo::ResultRow>, AppError> {
    ctx.repo.list_results_by_course(course_id)
}

#[tauri::command]
pub async fn withdraw_by_athlete_id(app: AppHandle, ctx: State<'_, AppCtx>, athlete_id: i64)
    -> Result<Timing, AppError> {
    let op = current_operator(&ctx).await?;
    let t = crate::timer::set_timing_status_by_athlete(
        &ctx.state, &ctx.repo, athlete_id, &op,
        crate::models::TimingStatus::Withdrawn,
    ).await?;
    let _ = app.emit("athlete:finished", &t);
    Ok(t)
}

#[tauri::command]
pub async fn mark_dns_by_athlete_id(app: AppHandle, ctx: State<'_, AppCtx>, athlete_id: i64)
    -> Result<Timing, AppError> {
    let op = current_operator(&ctx).await?;
    let t = crate::timer::set_timing_status_by_athlete(
        &ctx.state, &ctx.repo, athlete_id, &op,
        crate::models::TimingStatus::Dns,
    ).await?;
    let _ = app.emit("athlete:finished", &t);
    Ok(t)
}

// ---- Checkpoints / splits --------------------------------------------------

#[tauri::command]
pub async fn get_checkpoints(ctx: State<'_, AppCtx>) -> Result<Vec<crate::models::Checkpoint>, AppError> {
    ctx.repo.list_checkpoints()
}

#[derive(serde::Deserialize)]
pub struct CheckpointInput {
    pub course_id: i64,
    pub name: String,
    pub position: i64,
}

#[tauri::command]
pub async fn save_checkpoint(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    id: Option<i64>,
    input: CheckpointInput,
) -> Result<crate::models::Checkpoint, AppError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::InvalidState("nome checkpoint obbligatorio".into()));
    }
    let cp = crate::models::Checkpoint {
        id: match id { Some(v) => v, None => ctx.repo.next_local_checkpoint_id()? },
        course_id: input.course_id,
        name,
        position: input.position,
    };
    ctx.repo.upsert_checkpoint(&cp)?;
    let _ = app.emit("data:changed", ());
    Ok(cp)
}

#[tauri::command]
pub async fn delete_checkpoint(app: AppHandle, ctx: State<'_, AppCtx>, id: i64)
    -> Result<(), AppError> {
    ctx.repo.delete_checkpoint(id)?;
    let _ = app.emit("data:changed", ());
    Ok(())
}

#[tauri::command]
pub async fn record_split(
    app: AppHandle,
    ctx: State<'_, AppCtx>,
    checkpoint_id: i64,
    bib: i64,
) -> Result<crate::models::Split, AppError> {
    let op = current_operator(&ctx).await?;
    let (athlete_id, course_id) = {
        let s = ctx.state.read().await;
        let a = s.athletes_by_bib.get(&bib).cloned()
            .ok_or_else(|| AppError::NotFound(format!("pettorale {} non trovato", bib)))?;
        (a.id, a.course_id)
    };
    let split = crate::timer::record_split(
        &ctx.state, &ctx.repo, &*ctx.clock, athlete_id, checkpoint_id, course_id, &op,
    ).await?;
    let _ = app.emit("split:recorded", &split);
    Ok(split)
}

#[tauri::command]
pub async fn get_splits_by_course(ctx: State<'_, AppCtx>, course_id: i64)
    -> Result<Vec<crate::models::Split>, AppError> {
    ctx.repo.list_splits_by_course(course_id)
}

// ---- Backup / restore ------------------------------------------------------

#[tauri::command]
pub async fn backup_database(ctx: State<'_, AppCtx>, path: String) -> Result<String, AppError> {
    let repo = ctx.repo.clone();
    let dest = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || repo.backup_to(&dest))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
    Ok(path)
}

#[tauri::command]
pub async fn restore_database(app: AppHandle, ctx: State<'_, AppCtx>, path: String)
    -> Result<(), AppError> {
    let repo = ctx.repo.clone();
    let src = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || repo.restore_from(&src))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))??;
    refresh_state(&ctx).await?;
    let _ = app.emit("data:changed", ());
    Ok(())
}
