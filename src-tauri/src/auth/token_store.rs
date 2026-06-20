use crate::error::{AppError, AppResult};
use std::sync::RwLock;

const SERVICE: &str = "m-chrono";
const ACCOUNT_REFRESH: &str = "refresh_token";

#[derive(Debug, Clone)]
pub struct TokenCache {
    pub access_token: String,
    pub expires_at_ms: i64,
}

pub struct TokenStore {
    cache: RwLock<Option<TokenCache>>,
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStore {
    pub fn new() -> Self {
        Self { cache: RwLock::new(None) }
    }

    pub fn read_refresh_token(&self) -> AppResult<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT_REFRESH)
            .map_err(|e| AppError::Internal(format!("keyring: {e}")))?;
        match entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Internal(format!("keyring: {e}"))),
        }
    }

    pub fn write_refresh_token(&self, token: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT_REFRESH)
            .map_err(|e| AppError::Internal(format!("keyring: {e}")))?;
        entry.set_password(token)
            .map_err(|e| AppError::Internal(format!("keyring set: {e}")))?;
        Ok(())
    }

    pub fn delete_refresh_token(&self) -> AppResult<()> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT_REFRESH)
            .map_err(|e| AppError::Internal(format!("keyring: {e}")))?;
        let _ = entry.delete_credential();
        Ok(())
    }

    pub fn cache_access(&self, token: TokenCache) {
        *self.cache.write().unwrap() = Some(token);
    }

    pub fn cached_valid(&self, now_ms: i64, margin_ms: i64) -> Option<TokenCache> {
        self.cache.read().unwrap().clone()
            .filter(|c| now_ms + margin_ms < c.expires_at_ms)
    }

    pub fn clear_cache(&self) {
        *self.cache.write().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_valid_within_window() {
        let s = TokenStore::new();
        s.cache_access(TokenCache { access_token: "x".into(), expires_at_ms: 1000 });
        assert!(s.cached_valid(500, 100).is_some());
    }

    #[test]
    fn cache_invalid_inside_margin() {
        let s = TokenStore::new();
        s.cache_access(TokenCache { access_token: "x".into(), expires_at_ms: 1000 });
        assert!(s.cached_valid(950, 100).is_none()); // 950 + 100 == 1050 > 1000
    }

    #[test]
    fn clear_cache_drops_value() {
        let s = TokenStore::new();
        s.cache_access(TokenCache { access_token: "x".into(), expires_at_ms: 5000 });
        s.clear_cache();
        assert!(s.cached_valid(0, 0).is_none());
    }
}
