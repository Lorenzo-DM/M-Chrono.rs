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
        loop {
            let secs = cfg.read().await.sync_interval_secs.max(1);
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = sleep(Duration::from_secs(secs)) => {}
            }
            let cfg_snap = cfg.read().await.clone();
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

            let _ = push::push_pending(&repo, &http, &cfg_snap.api_base_url, &token).await;
            let _ = push::push_timings(&repo, &http, &cfg_snap.api_base_url, &token).await;
            let now = clock.now_ms();
            let _ = pull::pull_timings(&repo, &http, &cfg_snap.api_base_url, &token, now).await;

            // dedup athletes that have any timing
            let unique_aids: std::collections::HashSet<i64> = {
                let s = state.read().await;
                s.timings.values().filter_map(|t| t.athlete_id).collect()
            };
            for aid in unique_aids {
                let _ = dedup::group_finishes(
                    &repo, aid, cfg_snap.dedup_window_ms, cfg_snap.dedup_warn_delta_ms,
                );
            }

            // refresh state from db (covers pulled records)
            if let Ok(snap) = crate::state::bootstrap_from_db(&repo, &*clock) {
                *state.write().await = snap;
            }

            let pending = repo.fetch_unsynced_timings(1000).map(|v| v.len()).unwrap_or(0)
                + repo.fetch_unsynced_pending(1000).map(|v| v.len()).unwrap_or(0);
            let _ = app.emit(
                "sync:status",
                serde_json::json!({
                    "pending_count": pending,
                    "last_success_at_ms": now,
                    "last_error": null,
                    "is_online": true
                }),
            );
        }
    });
}
