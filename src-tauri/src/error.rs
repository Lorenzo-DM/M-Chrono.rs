use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("not found: {0}")] NotFound(String),
    #[error("invalid state: {0}")] InvalidState(String),
    #[error("db error: {0}")] Db(String),
    #[error("api error: {0}")] Api(String),
    #[error("unauthorized")] Unauthorized,
    #[error("offline")] Offline,
    #[error("io error: {0}")] Io(String),
    #[error("internal: {0}")] Internal(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self { AppError::Db(e.to_string()) }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e.to_string()) }
}
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() || e.is_connect() { AppError::Offline }
        else { AppError::Api(e.to_string()) }
    }
}

pub type AppResult<T> = Result<T, AppError>;
