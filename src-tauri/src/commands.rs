use crate::app_ctx::AppCtx;
use crate::error::AppError;
use crate::models::{Athlete, Course, PendingFinish};
use serde::Serialize;
use tauri::State;

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
