use crate::error::{AppError, AppResult};
use crate::models::DeviceCodeResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct DeviceAuthRaw {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: Option<String>,
    expires_in: i64,
    interval: i64,
}

pub struct DeviceCodeInit {
    pub device_code: String,
    pub response: DeviceCodeResponse,
}

pub async fn request(
    http: &reqwest::Client,
    device_authorization_endpoint: &str,
    client_id: &str,
    scopes: &str,
) -> AppResult<DeviceCodeInit> {
    let resp = http
        .post(device_authorization_endpoint)
        .form(&[("client_id", client_id), ("scope", scopes)])
        .send()
        .await?
        .error_for_status()
        .map_err(|e| AppError::Api(format!("device_authorization: {e}")))?;
    let raw: DeviceAuthRaw = resp.json().await?;
    Ok(DeviceCodeInit {
        device_code: raw.device_code,
        response: DeviceCodeResponse {
            user_code: raw.user_code,
            verification_uri: raw.verification_uri,
            verification_uri_complete: raw.verification_uri_complete,
            expires_in: raw.expires_in,
            interval: raw.interval,
        },
    })
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    #[allow(dead_code)]
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

pub enum PollOutcome {
    Pending,
    SlowDown,
    Success(TokenResponse),
    Denied,
    Expired,
}

#[derive(Serialize)]
struct PollForm<'a> {
    grant_type: &'a str,
    device_code: &'a str,
    client_id: &'a str,
}

pub async fn poll_once(
    http: &reqwest::Client,
    token_endpoint: &str,
    device_code: &str,
    client_id: &str,
) -> AppResult<PollOutcome> {
    let form = PollForm {
        grant_type: "urn:ietf:params:oauth:grant-type:device_code",
        device_code,
        client_id,
    };
    let resp = http.post(token_endpoint).form(&form).send().await?;
    let status = resp.status();
    if status.is_success() {
        Ok(PollOutcome::Success(resp.json().await?))
    } else {
        let body: ErrorResponse = resp
            .json()
            .await
            .unwrap_or(ErrorResponse { error: "unknown".into() });
        Ok(match body.error.as_str() {
            "authorization_pending" => PollOutcome::Pending,
            "slow_down" => PollOutcome::SlowDown,
            "access_denied" => PollOutcome::Denied,
            "expired_token" => PollOutcome::Expired,
            other => return Err(AppError::Api(format!("token poll: {other}"))),
        })
    }
}
