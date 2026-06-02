pub mod device_code;
pub mod refresh;
pub mod token_store;

#[cfg(test)]
mod tests;

pub use device_code::*;
pub use token_store::*;

use crate::auth::device_code::{poll_once, request as device_request, PollOutcome};
use crate::auth::refresh::refresh as do_refresh;
use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::models::DeviceCodeResponse;
use crate::timer::clock::ClockProvider;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub struct AuthService {
    pub store: Arc<TokenStore>,
    pub http: Arc<reqwest::Client>,
    pub clock: Arc<dyn ClockProvider>,
    pub pending_device_code: Mutex<Option<String>>,
}

impl AuthService {
    pub fn new(http: Arc<reqwest::Client>, clock: Arc<dyn ClockProvider>) -> Self {
        Self {
            store: Arc::new(TokenStore::new()),
            http,
            clock,
            pending_device_code: Mutex::new(None),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.store.read_refresh_token().ok().flatten().is_some()
    }

    pub async fn get_access_token(&self, cfg: &AppConfig) -> AppResult<String> {
        let now = self.clock.now_ms();
        if let Some(cached) = self.store.cached_valid(now, 30_000) {
            return Ok(cached.access_token);
        }
        let rt = self.store.read_refresh_token()?.ok_or(AppError::Unauthorized)?;
        let resp = do_refresh(&self.http, &cfg.oidc_issuer_url, &rt, &cfg.oidc_client_id).await?;
        if let Some(new_rt) = &resp.refresh_token {
            self.store.write_refresh_token(new_rt)?;
        }
        let exp = self.clock.now_ms() + resp.expires_in * 1000;
        let cache = TokenCache {
            access_token: resp.access_token.clone(),
            expires_at_ms: exp,
        };
        self.store.cache_access(cache);
        Ok(resp.access_token)
    }

    pub async fn start_device_login(&self, cfg: &AppConfig) -> AppResult<DeviceCodeResponse> {
        let init = device_request(
            &self.http,
            &cfg.oidc_issuer_url,
            &cfg.oidc_client_id,
            &cfg.oidc_scopes,
        )
        .await?;
        *self.pending_device_code.lock().await = Some(init.device_code);
        Ok(init.response)
    }

    pub async fn poll_until_authorized(
        &self,
        cfg: &AppConfig,
        interval_secs: i64,
    ) -> AppResult<()> {
        let mut interval = interval_secs.max(1) as u64;
        loop {
            sleep(Duration::from_secs(interval)).await;
            let dc = match self.pending_device_code.lock().await.clone() {
                Some(c) => c,
                None => return Err(AppError::InvalidState("no device code".into())),
            };
            match poll_once(&self.http, &cfg.oidc_issuer_url, &dc, &cfg.oidc_client_id).await? {
                PollOutcome::Pending => continue,
                PollOutcome::SlowDown => {
                    interval = (interval * 2).min(60);
                    continue;
                }
                PollOutcome::Denied => return Err(AppError::Unauthorized),
                PollOutcome::Expired => {
                    return Err(AppError::InvalidState("device code expired".into()))
                }
                PollOutcome::Success(tok) => {
                    if let Some(rt) = &tok.refresh_token {
                        self.store.write_refresh_token(rt)?;
                    }
                    let exp = self.clock.now_ms() + tok.expires_in * 1000;
                    self.store.cache_access(TokenCache {
                        access_token: tok.access_token,
                        expires_at_ms: exp,
                    });
                    *self.pending_device_code.lock().await = None;
                    return Ok(());
                }
            }
        }
    }

    pub fn logout(&self) -> AppResult<()> {
        self.store.clear_cache();
        self.store.delete_refresh_token()
    }
}
