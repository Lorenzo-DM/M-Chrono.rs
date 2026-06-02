use super::device_code::{poll_once, request, PollOutcome};
use super::refresh::refresh;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn device_authorization_happy_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/v2/device_authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "device_code": "abc",
            "user_code": "USER",
            "verification_uri": "https://example/dev",
            "verification_uri_complete": null,
            "expires_in": 600,
            "interval": 5
        })))
        .mount(&server)
        .await;
    let http = reqwest::Client::new();
    let r = request(&http, &server.uri(), "cid", "scope").await.unwrap();
    assert_eq!(r.response.user_code, "USER");
    assert_eq!(r.device_code, "abc");
}

#[tokio::test]
async fn token_poll_pending_then_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/v2/token"))
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(serde_json::json!({ "error": "authorization_pending" })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth/v2/token"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "access_token": "AT",
                "refresh_token": "RT",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
        .mount(&server)
        .await;
    let http = reqwest::Client::new();
    let r1 = poll_once(&http, &server.uri(), "dc", "cid").await.unwrap();
    assert!(matches!(r1, PollOutcome::Pending));
    let r2 = poll_once(&http, &server.uri(), "dc", "cid").await.unwrap();
    match r2 {
        PollOutcome::Success(tok) => {
            assert_eq!(tok.access_token, "AT");
            assert_eq!(tok.refresh_token.as_deref(), Some("RT"));
        }
        _ => panic!("expected Success"),
    }
}

#[tokio::test]
async fn refresh_invalid_grant_maps_to_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/v2/token"))
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(serde_json::json!({ "error": "invalid_grant" })))
        .mount(&server)
        .await;
    let http = reqwest::Client::new();
    let r = refresh(&http, &server.uri(), "rt", "cid").await;
    assert!(matches!(r, Err(crate::error::AppError::Unauthorized)));
}
