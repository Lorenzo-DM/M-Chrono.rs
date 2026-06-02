use crate::error::{AppError, AppResult};
use crate::models::{Athlete, Course};

pub async fn fetch_courses(
    http: &reqwest::Client,
    base_url: &str,
    access_token: &str,
) -> AppResult<Vec<Course>> {
    let url = format!("{}/courses", base_url.trim_end_matches('/'));
    let resp = http.get(&url).bearer_auth(access_token).send().await?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Unauthorized);
    }
    let resp = resp
        .error_for_status()
        .map_err(|e| AppError::Api(format!("courses: {e}")))?;
    Ok(resp.json().await?)
}

pub async fn fetch_athletes(
    http: &reqwest::Client,
    base_url: &str,
    access_token: &str,
) -> AppResult<Vec<Athlete>> {
    let url = format!("{}/athletes", base_url.trim_end_matches('/'));
    let resp = http.get(&url).bearer_auth(access_token).send().await?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::Unauthorized);
    }
    let resp = resp
        .error_for_status()
        .map_err(|e| AppError::Api(format!("athletes: {e}")))?;
    Ok(resp.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetch_courses_sends_bearer() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/courses"))
            .and(header("authorization", "Bearer AT"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": 1, "name": "21K", "distance_m": null,
                    "started_at_ms": null, "scheduled_at_ms": null
                }
            ])))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let r = fetch_courses(&http, &server.uri(), "AT").await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "21K");
    }

    #[tokio::test]
    async fn unauthorized_maps_correctly() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/courses"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let r = fetch_courses(&http, &server.uri(), "bad").await;
        assert!(matches!(r, Err(AppError::Unauthorized)));
    }

    #[tokio::test]
    async fn fetch_athletes_returns_list() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/athletes"))
            .and(header("authorization", "Bearer AT"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                { "id": 1, "bib_number": 7, "first_name": "Mario",
                  "last_name": "Rossi", "course_id": 1 }
            ])))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let r = fetch_athletes(&http, &server.uri(), "AT").await.unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].bib_number, 7);
    }
}
