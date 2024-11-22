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

pub fn read_history_file(history_path: &str) -> Result<HistoryFile> {
    let history_file = std::fs::read_to_string(history_path)?;
    let history: HistoryFile = serde_json::from_str(&history_file)?;
    Ok(history)
}

pub fn write_history_file(history: &HistoryFile) -> Result<()> {
    let history_json = serde_json::to_string_pretty(&history)?;
    std::fs::write(HISTORY_PATH, history_json)?;
    Ok(())
}

// History is stored so the newest entry is at the end of the array
pub fn append_history_event(section: &str, event: HistoryEntry) -> Result<()> {
    // We want to mutate to add because a copy could be really expensive
    let mut history = read_history_file(HISTORY_PATH)?;
    history
        .watchers
        .iter_mut()
        .find(|w| w.name == section)
        .expect(format!("History section {} could not be found", section).as_str())
        .entries
        .push(event);

    write_history_file(&history)?;
    Ok(())
}

pub fn update_history_section(section: &str, event: HistoryEntry) -> Result<()> {
    let mut history = read_history_file(HISTORY_PATH)?;
    let section = history
        .watchers
        .iter_mut()
        .find(|w| w.name == section)
        .expect(format!("History section {} could not be found", section).as_str());
    section.last_updated = chrono::Utc::now().naive_utc();

    match section.entries.last() {
        None => section.entries.push(event),
        Some(e) => {
            if e.date != event.date {
                section.entries.push(event);
            } else {
                match (&e.state, &event.state) {
                    // Do nothing if both at success
                    (State::Success, State::Success) => (),
                    // If the old event was success and the new event is not, replace it
                    (State::Success, _) => {
                        section.entries.pop();
                        section.entries.push(event)
                    }
                    // If the old event was disabled and the new event is not, replace it
                    (State::Disabled, _) => {
                        section.entries.pop();
                        section.entries.push(event);
                    }
                    // We want a warning to be replaced by a danger
                    (State::Warning, State::Danger) => {
                        section.entries.pop();
                        section.entries.push(event);
                    }
                    // We want a warning to be replaced by a failure
                    (State::Warning, State::Failure) => {
                        section.entries.pop();
                        section.entries.push(event);
                    }
                    // We want a danger to be replaced by a failure
                    (State::Danger, State::Failure) => {
                        section.entries.pop();
                        section.entries.push(event);
                    }
                    _ => {}
                };
            }
        }
    };

    write_history_file(&history)?;
    Ok(())
}
