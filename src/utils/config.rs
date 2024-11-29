#![allow(clippy::missing_errors_doc)]
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::{long_date_format, HistoryEntry, State, StatusPageContext, HISTORY_PATH};

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryFile {
    pub watchers: Vec<HistorySection>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HistorySection {
    pub name: String,
    #[serde(with = "long_date_format")]
    pub last_updated: NaiveDateTime,
    pub uptime: Option<f64>,
    pub entries: Vec<HistoryEntry>,
}

pub fn read_config_file(config_path: &str) -> Result<StatusPageContext> {
    let config_file = std::fs::read_to_string(config_path)?;
    let config: StatusPageContext = serde_json::from_str(&config_file)?;
    Ok(config)
}

pub fn create_history_file(check_name: &str) -> Result<HistorySection> {
    // Turn a "Backend API" into "Backend_API"
    let check_file_path = check_name.split(' ').collect::<Vec<&str>>().join("_");

    let file_path = format!("{HISTORY_PATH}/{check_file_path}_history.json");

    let history = HistorySection {
        name: check_name.into(),
        last_updated: chrono::Utc::now().naive_utc(),
        uptime: None,
        entries: vec![],
    };

    let history_json = serde_json::to_string_pretty(&history)?;
    std::fs::write(file_path, history_json)?;
    Ok(history)
}

pub fn does_history_file_exist(check_name: &str) -> Result<bool> {
    // Turn a "Backend API" into "Backend_API"
    let check_file_path = check_name.split(' ').collect::<Vec<&str>>().join("_");

    let file_path = format!("{HISTORY_PATH}/{check_file_path}_history.json");

    Ok(std::path::Path::new(&file_path).exists())
}

pub fn read_history_file(check_name: &str) -> Result<HistorySection> {
    // Turn a "Backend API" into "Backend_API"
    let check_file_path = check_name.split(' ').collect::<Vec<&str>>().join("_");

    let file_path = format!("{HISTORY_PATH}/{check_file_path}_history.json");

    let history_file = std::fs::read_to_string(file_path)?;
    let history: HistorySection = serde_json::from_str(&history_file)?;
    Ok(history)
}

pub fn write_history_file(check_name: &str, history: &HistorySection) -> Result<()> {
    // Turn a "Backend API" into "Backend_API"
    let check_file_path = check_name.split(' ').collect::<Vec<&str>>().join("_");

    let file_path = format!("{HISTORY_PATH}/{check_file_path}_history.json");

    let history_json = serde_json::to_string_pretty(&history)?;
    std::fs::write(file_path, history_json)?;
    Ok(())
}

// History is stored so the newest entry is at the end of the array
pub fn append_history_event(section: &str, event: HistoryEntry) -> Result<()> {
    // We want to mutate to add because a copy could be really expensive
    let mut history = read_history_file(section)?;

    history.entries.push(event);

    write_history_file(section, &history)?;
    Ok(())
}

pub fn update_history_section(section: &str, event: HistoryEntry) -> Result<()> {
    let mut history = read_history_file(section)?;
    history.last_updated = chrono::Utc::now().naive_utc();

    match history.entries.last() {
        None => history.entries.push(event),
        Some(e) => {
            if e.date == event.date {
                #[allow(clippy::match_same_arms)]
                match (&e.state, &event.state) {
                    // Do nothing if both at success
                    (State::Success, State::Success) => (),
                    // If the old event was success and the new event is not, replace it
                    (State::Success, _) => {
                        history.entries.pop();
                        history.entries.push(event);
                    }
                    // If the old event was disabled and the new event is not, replace it
                    (State::Disabled, _) => {
                        history.entries.pop();
                        history.entries.push(event);
                    }
                    // We want a warning to be replaced by a danger
                    (State::Warning, State::Danger) => {
                        history.entries.pop();
                        history.entries.push(event);
                    }
                    // We want a warning to be replaced by a failure
                    (State::Warning, State::Failure) => {
                        history.entries.pop();
                        history.entries.push(event);
                    }
                    // We want a danger to be replaced by a failure
                    (State::Danger, State::Failure) => {
                        history.entries.pop();
                        history.entries.push(event);
                    }
                    _ => {}
                };
            } else {
                history.entries.push(event);
            }
        }
    };

    write_history_file(section, &history)?;
    Ok(())
}
