use crate::utils::config::append_history_event;
use crate::{Check, CheckResult, CheckType};
use crate::{HistoryEntry, Result, State};

pub async fn run_check(check: &Check) -> Result<CheckResult> {
    let timeout = std::time::Duration::from_millis(check.timeout_ms);

    let result = tokio::time::timeout(timeout, perform_check(check)).await?;

    match result {
        // Write to history file
        Ok(CheckResult::Success) => {
            // TODO: Only append if the last entry is a worse status
            // or if the last entry is a different date
            append_history_event(
                check.name.as_str(),
                HistoryEntry::new_today(State::Success, "No Incident".into()),
            )?;
            Ok(CheckResult::Success)
        }
        Ok(CheckResult::Failure) => return Ok(CheckResult::Failure),
        Err(_) => return Ok(CheckResult::Failure),
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
            Ok(CheckResult::Failure)
        }
    }
}

pub async fn perform_http_check(check: &Check) -> Result<CheckResult> {
    let response = reqwest::get(&check.target).await?;
    let status = response.status().as_u16();
    let expected_status = check.expected_status.unwrap_or(200);

    if status == expected_status {
        Ok(CheckResult::Success)
    } else {
        Ok(CheckResult::Failure)
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

    match output {
        Ok(_) => Ok(CheckResult::Success),
        Err(_) => Ok(CheckResult::Failure),
    }
}

pub async fn perform_port_check(check: &Check) -> Result<CheckResult> {
    let target = format!("{}:{}", check.target, check.port.unwrap());
    let output = tokio::net::TcpStream::connect(&target).await;

    match output {
        Ok(_) => Ok(CheckResult::Success),
        Err(_) => Ok(CheckResult::Failure),
    }
}
