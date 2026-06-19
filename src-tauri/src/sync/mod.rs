pub mod push;
pub mod pull;
pub mod dedup;
pub mod client;

use crate::auth::AuthService;
use crate::config::AppConfig;
use crate::db::repo::Repo;
use crate::error::AppError;
use crate::state::SharedState;
use crate::timer::clock::ClockProvider;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    app: AppHandle,
    state: SharedState,
    repo: Arc<Repo>,
    http: Arc<reqwest::Client>,
    auth: Arc<AuthService>,
    cfg: Arc<RwLock<AppConfig>>,
    clock: Arc<dyn ClockProvider>,
    cancel: CancellationToken,
) {
    tauri::async_runtime::spawn(async move {
        let mut auth_paused = false;
        // Athletes touched at or after this timestamp get re-deduped. 0 on the
        // first cycle so existing data is grouped once, then it advances.
        let mut last_dedup_ms: i64 = 0;
        loop {
            let secs = cfg.read().await.sync_interval_secs.max(1);
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = sleep(Duration::from_secs(secs)) => {}
            }
            let cfg_snap = cfg.read().await.clone();
            if !cfg_snap.sync_enabled {
                auth_paused = false;
                continue;
            }
            let token = match auth.get_access_token(&cfg_snap).await {
                Ok(t) => {
                    if auth_paused {
                        let _ = app.emit("network:status", serde_json::json!({ "online": true }));
                        auth_paused = false;
                    }
                    t
                }
                Err(AppError::Offline) => {
                    let _ = app.emit("network:status", serde_json::json!({ "online": false }));
                    continue;
                }
                Err(_) => {
                    if !auth_paused {
                        let _ = app.emit("auth:required", ());
                        auth_paused = true;
                    }
                    continue;
                }
            };

            let now = clock.now_ms();
            let mut sync_error: Option<String> = None;
            let mut went_offline = false;

            // Push/pull; collect the first error so the UI can surface it.
            let steps = [
                push::push_pending(&repo, &http, &cfg_snap.api_base_url, &token).await.map(|_| ()),
                push::push_timings(&repo, &http, &cfg_snap.api_base_url, &token).await.map(|_| ()),
                pull::pull_timings(&repo, &http, &cfg_snap.api_base_url, &token, now).await.map(|_| ()),
            ];
            for step in steps {
                if let Err(e) = step {
                    match e {
                        AppError::Offline => went_offline = true,
                        other => if sync_error.is_none() {
                            sync_error = Some(other.to_string());
                        },
                    }
                }
            }
            if went_offline {
                let _ = app.emit("network:status", serde_json::json!({ "online": false }));
            }

            // Re-group duplicates only for athletes whose finishes changed.
            if let Ok(aids) = repo.athletes_with_finishes_since(last_dedup_ms) {
                for aid in aids {
                    let _ = dedup::group_finishes(
                        &repo, aid, cfg_snap.dedup_window_ms, cfg_snap.dedup_warn_delta_ms,
                    );
                }
                last_dedup_ms = now;
            }

            // refresh state from db (covers pulled records)
            if let Ok(snap) = crate::state::bootstrap_from_db(&repo, &*clock) {
                *state.write().await = snap;
            }

            // Prefer a live step error; fall back to a persisted per-row error.
            let last_error = sync_error.or_else(|| repo.last_sync_error().ok().flatten());
            let online = !went_offline;
            let pending = repo.fetch_unsynced_timings(1000).map(|v| v.len()).unwrap_or(0)
                + repo.fetch_unsynced_pending(1000).map(|v| v.len()).unwrap_or(0);
            let _ = app.emit(
                "sync:status",
                serde_json::json!({
                    "pending_count": pending,
                    "last_success_at_ms": if last_error.is_none() { Some(now) } else { None },
                    "last_error": last_error,
                    "is_online": online
                }),
            );
        }
    });
}
