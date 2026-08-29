use super::device_code::{poll_once, request, PollOutcome};
use super::discovery::discover;
use super::refresh::refresh;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Mount an OIDC discovery document advertising the given endpoints.
async fn mount_discovery(server: &MockServer, device_endpoint: Option<&str>) {
    let mut doc = serde_json::json!({
        "issuer": server.uri(),
        "token_endpoint": format!("{}/protocol/openid-connect/token", server.uri()),
    });
    if let Some(de) = device_endpoint {
        doc["device_authorization_endpoint"] = serde_json::json!(format!("{}{}", server.uri(), de));
    }
    Mock::given(method("GET"))
        .and(path("/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(doc))
        .mount(server)
        .await;
}

#[tokio::test]
async fn discovery_reads_endpoints_from_well_known_document() {
    let server = MockServer::start().await;
    mount_discovery(&server, Some("/protocol/openid-connect/auth/device")).await;
    let http = reqwest::Client::new();

    let eps = discover(&http, &server.uri()).await.unwrap();

    assert_eq!(
        eps.token_endpoint,
        format!("{}/protocol/openid-connect/token", server.uri())
    );
    assert_eq!(
        eps.require_device_endpoint().unwrap(),
        format!("{}/protocol/openid-connect/auth/device", server.uri())
    );
}

#[tokio::test]
async fn discovery_tolerates_trailing_slash_on_issuer() {
    let server = MockServer::start().await;
    mount_discovery(&server, Some("/device")).await;
    let http = reqwest::Client::new();

    let eps = discover(&http, &format!("{}/", server.uri())).await.unwrap();

    assert!(eps.token_endpoint.ends_with("/protocol/openid-connect/token"));
}

#[tokio::test]
async fn provider_without_device_grant_is_reported() {
    let server = MockServer::start().await;
    mount_discovery(&server, None).await;
    let http = reqwest::Client::new();

    let eps = discover(&http, &server.uri()).await.unwrap();

    match eps.require_device_endpoint() {
        Err(crate::error::AppError::Api(msg)) => {
            assert!(msg.contains("device_authorization_endpoint"), "got: {msg}")
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

#[tokio::test]
async fn discovery_missing_document_is_an_api_error() {
    let server = MockServer::start().await;
    let http = reqwest::Client::new();

    let r = discover(&http, &server.uri()).await;

    assert!(matches!(r, Err(crate::error::AppError::Api(_))));
}

#[tokio::test]
async fn device_authorization_happy_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/auth/device"))
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
    let endpoint = format!("{}/protocol/openid-connect/auth/device", server.uri());
    let r = request(&http, &endpoint, "cid", "scope").await.unwrap();
    assert_eq!(r.response.user_code, "USER");
    assert_eq!(r.device_code, "abc");
}

#[tokio::test]
async fn token_poll_pending_then_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(serde_json::json!({ "error": "authorization_pending" })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/protocol/openid-connect/token"))
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
    let endpoint = format!("{}/protocol/openid-connect/token", server.uri());
    let r1 = poll_once(&http, &endpoint, "dc", "cid").await.unwrap();
    assert!(matches!(r1, PollOutcome::Pending));
    let r2 = poll_once(&http, &endpoint, "dc", "cid").await.unwrap();
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
        .and(path("/protocol/openid-connect/token"))
        .respond_with(ResponseTemplate::new(400)
            .set_body_json(serde_json::json!({ "error": "invalid_grant" })))
        .mount(&server)
        .await;
    let http = reqwest::Client::new();
    let endpoint = format!("{}/protocol/openid-connect/token", server.uri());
    let r = refresh(&http, &endpoint, "rt", "cid").await;
    assert!(matches!(r, Err(crate::error::AppError::Unauthorized)));
}
