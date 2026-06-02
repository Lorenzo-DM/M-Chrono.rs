#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod models;
mod state;
mod commands;
mod db;
mod timer;
mod sync;
mod api;
mod auth;
mod export;
mod app_ctx;

use crate::app_ctx::AppCtx;
use crate::config::AppConfig;
use crate::db::{migrations, repo::Repo, Db};
use crate::state::{bootstrap_from_db, new_shared};
use crate::timer::clock::SystemClock;
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("trailtrace_stopwatch_lib=info".parse().unwrap()))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("race.db");
            let config_path = app_dir.join("config.json");

            let db = Db::open(&db_path)?;
            migrations::run(&db.conn.lock().unwrap())?;

            let clock = Arc::new(SystemClock);
            let repo = Arc::new(Repo::with_clock(db.conn.clone(), clock.clone()));
            let config = Arc::new(tokio::sync::RwLock::new(AppConfig::load_or_default(&config_path)));

            let state = new_shared();
            let snap = bootstrap_from_db(&repo, &*clock)?;
            tauri::async_runtime::block_on(async {
                *state.write().await = snap;
            });

            let http = Arc::new(reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(15))
                .build()?);

            let auth = Arc::new(crate::auth::AuthService::new(http.clone(), clock.clone()));

            app.manage(AppCtx { state, repo, clock, config, config_path, http, auth });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_courses,
            commands::poll_display,
            commands::get_athletes_by_course,
            commands::get_pending_finishes,
            commands::get_config,
            commands::start_course,
            commands::finish_by_bib,
            commands::finish_by_athlete_id,
            commands::capture_pending_finish,
            commands::assign_pending,
            commands::withdraw_athlete,
            commands::undo_finish,
            commands::update_operator_id,
            commands::start_device_login,
            commands::is_authenticated,
            commands::logout,
            commands::fetch_remote_data,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
