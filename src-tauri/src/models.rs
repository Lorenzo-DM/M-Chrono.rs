use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub scheduled_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub name: String,
    pub distance_m: Option<i64>,
    pub started_at_ms: Option<i64>,
    pub scheduled_at_ms: Option<i64>,
    #[serde(default)]
    pub ended_at_ms: Option<i64>,
    // Local-only link to a race; backend courses omit it (serde default = None).
    #[serde(default)]
    pub race_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Athlete {
    pub id: i64,
    pub bib_number: i64,
    pub first_name: String,
    pub last_name: String,
    pub course_id: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimingStatus { Registered, Running, Finished, Withdrawn }

impl TimingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Registered => "Registered",
            Self::Running => "Running",
            Self::Finished => "Finished",
            Self::Withdrawn => "Withdrawn",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Registered" => Some(Self::Registered),
            "Running" => Some(Self::Running),
            "Finished" => Some(Self::Finished),
            "Withdrawn" => Some(Self::Withdrawn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timing {
    pub id: i64,
    pub remote_id: Option<i64>,
    pub athlete_id: Option<i64>,
    pub course_id: i64,
    pub start_timestamp_ms: Option<i64>,
    pub finish_timestamp_ms: Option<i64>,
    pub status: TimingStatus,
    pub total_time_ms: Option<i64>,
    pub operator_id: String,
    pub duplicate_group_id: Option<String>,
    pub duplicate_flagged: bool,
    pub synced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingFinish {
    pub id: i64,
    pub remote_id: Option<i64>,
    pub course_id: i64,
    pub finish_timestamp_ms: i64,
    pub operator_id: String,
    pub assigned: bool,
    pub synced: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo { pub sub: String, pub name: Option<String>, pub email: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: i64,
    pub interval: i64,
}
