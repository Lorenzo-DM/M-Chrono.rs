use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_scopes: String,
    pub api_base_url: String,
    pub sync_interval_secs: u64,
    pub operator_id: String,
    pub dedup_window_ms: i64,
    pub dedup_warn_delta_ms: i64,
    #[serde(default)]
    pub sync_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            oidc_issuer_url: "https://idp.example.com".into(),
            oidc_client_id: "REPLACE_ME".into(),
            oidc_scopes: "openid profile email offline_access".into(),
            api_base_url: "https://api.example.com".into(),
            sync_interval_secs: 10,
            operator_id: "".into(),
            dedup_window_ms: 2000,
            dedup_warn_delta_ms: 500,
            sync_enabled: false,
        }
    }
}

impl AppConfig {
    pub fn load_or_default(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => {
                let cfg = AppConfig::default();
                let _ = fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")));
                let _ = fs::write(path, serde_json::to_string_pretty(&cfg).unwrap());
                cfg
            }
        }
    }
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, serde_json::to_string_pretty(self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_default_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let cfg = AppConfig::load_or_default(&path);
        assert_eq!(cfg.sync_interval_secs, 10);
        assert!(path.exists());
    }

    #[test]
    fn legacy_config_without_sync_enabled_still_loads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let legacy = r#"{
            "oidc_issuer_url": "https://idp.example.com",
            "oidc_client_id": "client123",
            "oidc_scopes": "openid",
            "api_base_url": "https://api.example.com",
            "sync_interval_secs": 15,
            "operator_id": "PC-B",
            "dedup_window_ms": 2000,
            "dedup_warn_delta_ms": 500
        }"#;
        fs::write(&path, legacy).unwrap();
        let cfg = AppConfig::load_or_default(&path);
        assert_eq!(cfg.operator_id, "PC-B");
        assert_eq!(cfg.sync_interval_secs, 15);
        assert!(!cfg.sync_enabled);
    }

    #[test]
    fn roundtrip_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut cfg = AppConfig::default();
        cfg.operator_id = "PC-A".into();
        cfg.save(&path).unwrap();
        let loaded = AppConfig::load_or_default(&path);
        assert_eq!(loaded.operator_id, "PC-A");
    }
}
