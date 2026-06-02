use crate::auth::AuthService;
use crate::config::AppConfig;
use crate::db::repo::Repo;
use crate::state::SharedState;
use crate::timer::clock::ClockProvider;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppCtx {
    pub state: SharedState,
    pub repo: Arc<Repo>,
    pub clock: Arc<dyn ClockProvider>,
    pub config: Arc<RwLock<AppConfig>>,
    pub config_path: std::path::PathBuf,
    pub http: Arc<reqwest::Client>,
    pub auth: Arc<AuthService>,
}
