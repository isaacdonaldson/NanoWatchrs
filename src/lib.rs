#![allow(clippy::missing_errors_doc)]
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod utils;

pub const ASSETS_PATH: &str = "assets";
pub const CONFIG_PATH: &str = "config/config.json";
pub const DATE_FORMAT: &str = "%Y-%m-%d";
pub const TIME_FORMAT: &str = "%H:%M:%S";
pub const LONG_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const HISTORY_PATH: &str = "config";
pub const HISTORY_LENGTH: usize = 30;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusPageContext {
    pub settings: SiteSettings,
    pub checks: Vec<Check>,
    pub incidents: Vec<Incident>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SiteSettings {
    pub site: SiteParams,
    pub page: PageParams,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SiteParams {
    pub name: String,
    pub description: String,
    pub url: String,
    pub logo: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PageParams {
    pub title: String,
    pub header: String,
    pub header_link: String,
    pub subheader: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusBlock {
    pub title: String,
    pub subtitle: String,
    pub status: String,
    pub state: State,
    #[serde(with = "long_date_format")]
    pub updated_at: NaiveDateTime,
    pub history_line: Vec<HistoryEntry>,
    pub uptime: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum State {
    #[serde(rename = "unknown")]
    Disabled,
    Success,
    Warning,
    Danger,
    Failure,
}

impl State {
    #[must_use]
    pub const fn to_state(&self) -> &str {
        match self {
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
            Self::Failure => "failure",
            Self::Disabled => "disabled",
        }
    }

    #[must_use]
    pub const fn to_status(&self) -> &str {
        match self {
            Self::Success => "OK",
            Self::Warning => "Degraded",
            Self::Danger => "Issues",
            Self::Failure => "Down",
            Self::Disabled => "Unknown",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HistoryEntry {
    #[serde(with = "date_format")]
    pub date: NaiveDate,
    pub state: State,
    pub notes: String,
}

impl HistoryEntry {
    #[must_use]
    pub fn new_today(state: State, notes: String) -> Self {
        Self {
            date: Utc::now().naive_utc().date(),
            state,
            notes,
        }
    }

    #[must_use]
    pub fn default_for_date(date: NaiveDate) -> Self {
        Self {
            date,
            state: State::Disabled,
            notes: String::from("Information N/A"),
        }
    }
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            date: Utc::now().naive_utc().date(),
            state: State::Disabled,
            notes: String::from("Information N/A"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Check {
    pub name: String,
    pub description: Option<String>,
    pub target: String,
    pub page_link: Option<String>,
    pub expected_status: Option<u16>,
    pub timeout_ms: u64,
    #[serde(rename = "type")]
    pub check_type: CheckType,
    pub port: Option<u16>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CheckType {
    Http,
    Ping,
    Port,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CheckResult {
    Success,
    Failure(State),
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Incident {
    pub title: String,
    pub description: String,
    pub status: String,
    pub display_date: String,
    #[serde(with = "long_date_format")]
    pub started_at: NaiveDateTime,
    #[serde(with = "long_date_format")]
    pub resolved_at: NaiveDateTime,
}

pub mod date_format {
    // This is for dates in the format of "2021-01-01"
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::DATE_FORMAT;

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let dt = NaiveDate::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)?;

        Ok(dt)
    }
}

pub mod long_date_format {
    // This is for dates in the format of "2021-01-01 12:00:00"
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::LONG_DATE_FORMAT;

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(LONG_DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let dt = NaiveDateTime::parse_from_str(&s, LONG_DATE_FORMAT)
            .map_err(serde::de::Error::custom)?;

        Ok(dt)
    }
}
