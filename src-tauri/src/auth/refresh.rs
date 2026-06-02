use crate::auth::device_code::TokenResponse;
use crate::error::{AppError, AppResult};
use serde::Serialize;

#[derive(Serialize)]
struct RefreshForm<'a> {
    grant_type: &'a str,
    refresh_token: &'a str,
    client_id: &'a str,
}

pub async fn refresh(
    http: &reqwest::Client,
    issuer: &str,
    refresh_token: &str,
    client_id: &str,
) -> AppResult<TokenResponse> {
    let url = format!("{}/oauth/v2/token", issuer.trim_end_matches('/'));
    let form = RefreshForm {
        grant_type: "refresh_token",
        refresh_token,
        client_id,
    };
    let resp = http.post(&url).form(&form).send().await?;
    let status = resp.status();
    if status.is_success() {
        Ok(resp.json().await?)
    } else if status == reqwest::StatusCode::BAD_REQUEST
        || status == reqwest::StatusCode::UNAUTHORIZED
    {
        Err(AppError::Unauthorized)
    } else {
        Err(AppError::Api(format!("refresh: HTTP {status}")))
    }
}
