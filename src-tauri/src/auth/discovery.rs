use crate::error::{AppError, AppResult};
use serde::Deserialize;

/// Subset of the OpenID Connect Discovery document we rely on.
///
/// Endpoints are read from `<issuer>/.well-known/openid-configuration` instead
/// of being derived from provider-specific URL layouts, so any spec-compliant
/// OIDC provider works.
#[derive(Debug, Clone, Deserialize)]
pub struct OidcEndpoints {
    pub token_endpoint: String,
    #[serde(default)]
    pub device_authorization_endpoint: Option<String>,
}

impl OidcEndpoints {
    /// The device authorization endpoint, or an error when the provider does
    /// not advertise support for the Device Authorization Grant.
    pub fn require_device_endpoint(&self) -> AppResult<&str> {
        self.device_authorization_endpoint.as_deref().ok_or_else(|| {
            AppError::Api(
                "provider does not advertise a device_authorization_endpoint; \
                 enable the Device Authorization Grant on the OIDC client"
                    .into(),
            )
        })
    }
}

pub async fn discover(http: &reqwest::Client, issuer: &str) -> AppResult<OidcEndpoints> {
    let url = format!(
        "{}/.well-known/openid-configuration",
        issuer.trim_end_matches('/')
    );
    let resp = http
        .get(&url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::Api(format!("oidc discovery: {e}")))?;
    resp.json::<OidcEndpoints>()
        .await
        .map_err(|e| AppError::Api(format!("oidc discovery: malformed document: {e}")))
}
