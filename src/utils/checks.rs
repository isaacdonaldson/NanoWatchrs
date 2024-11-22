use crate::{Check, CheckResult, CheckType};
use crate::{HistoryEntry, Result, State};

use super::config::update_history_section;

pub async fn run_check(check: &Check) -> Result<CheckResult> {
    let timeout = std::time::Duration::from_millis(check.timeout_ms);

    let result = tokio::time::timeout(timeout, perform_check(check)).await?;

    match result {
        // Write to history file
        Ok(CheckResult::Success) => {
            // This function takes care of only writing additions based on some rules
            update_history_section(
                check.name.as_str(),
                HistoryEntry::new_today(State::Success, "No Incident".into()),
            )?;
            Ok(CheckResult::Success)
        }
        Ok(CheckResult::Failure(state)) => {
            let history_entry = match state {
                State::Failure => {
                    HistoryEntry::new_today(State::Failure, "Ongoing Incident".into())
                }
                State::Danger => {
                    HistoryEntry::new_today(State::Danger, "Potential Outage or Issue".into())
                }
                State::Warning => {
                    HistoryEntry::new_today(State::Warning, "Degraded Performance".into())
                }
                State::Disabled => {
                    HistoryEntry::new_today(State::Disabled, "Information N/A".into())
                }
                _ => unreachable!("There is a state returned from a check that doesn't make sense"),
            };

            update_history_section(check.name.as_str(), history_entry)?;
            Ok(CheckResult::Failure(state))
        }
        Err(_) => {
            // If there is an error with the checking program we don't want that recorded
            // as an incident in the history file
            Ok(CheckResult::Unknown)
        }
        _ => unreachable!("Getting a result from a check that doesn't make sense"),
    }
}

pub async fn perform_check(check: &Check) -> Result<CheckResult> {
    let start_time = chrono::Utc::now();
    let result = match check.check_type {
        CheckType::Http => perform_http_check(check).await,
        CheckType::Ping => perform_ping_check(check),
        CheckType::Port => perform_port_check(check).await,
    };

    match result {
        Ok(result) => {
            println!("Check success: {:?} @ {:?}", check.name, start_time,);
            Ok(result)
        }
        Err(err) => {
            eprintln!("Error performing check: {:?}", err);
            Ok(CheckResult::Unknown)
        }
    }
}

pub async fn perform_http_check(check: &Check) -> Result<CheckResult> {
    let response = match reqwest::get(&check.target).await {
        Ok(response) => response,
        Err(_) => return Ok(CheckResult::Failure(State::Danger)),
    };

    let status = response.status().as_u16();
    let expected_status = check.expected_status.unwrap_or(200);

    // This is the only success case
    if status == expected_status {
        return Ok(CheckResult::Success);
    }

    // These are all failure cases
    match status {
        301 | 302 | 303 => Ok(CheckResult::Failure(State::Warning)),
        308 => Ok(CheckResult::Failure(State::Danger)),
        401 => Ok(CheckResult::Failure(State::Warning)),
        400 => Ok(CheckResult::Failure(State::Warning)),
        403 => Ok(CheckResult::Failure(State::Warning)),
        404 => Ok(CheckResult::Failure(State::Danger)),
        405 => Ok(CheckResult::Failure(State::Danger)),
        422 => Ok(CheckResult::Failure(State::Warning)),
        429 => Ok(CheckResult::Failure(State::Warning)),
        500 => Ok(CheckResult::Failure(State::Failure)),
        // 5xx status codes are considered failures?
        _ if status >= 500 => Ok(CheckResult::Failure(State::Failure)),
        // Any other status code is considered a danger (mid between warning and failure)
        _ => Ok(CheckResult::Failure(State::Danger)),
    }
}

pub fn perform_ping_check(check: &Check) -> Result<CheckResult> {
    let output = std::process::Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("-W")
        .arg(check.timeout_ms.to_string())
        .arg(&check.target)
        .output();

    // There is not a well defined granularity for ping checks
    // and no support to specify them (yet?)
    match output {
        Ok(_) => Ok(CheckResult::Success),
        Err(_) => Ok(CheckResult::Failure(State::Danger)),
    }
}

pub async fn perform_port_check(check: &Check) -> Result<CheckResult> {
    let target = format!("{}:{}", check.target, check.port.unwrap());
    let output = tokio::net::TcpStream::connect(&target).await;

    // There is not a well defined granularity for port checks
    // and no support to specify them (yet?)
    match output {
        Ok(_) => Ok(CheckResult::Success),
        Err(_) => Ok(CheckResult::Failure(State::Danger)),
    }
}
