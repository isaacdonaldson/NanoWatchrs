use crate::Result;
use crate::{Check, CheckResult, CheckType};
// {
//      "name": "Backend API",
//      "type": "http",
//      "target": "https://example.com",
//      "page_link": "https://example.com",
//      "expected_status": 200,
//      "timeout_ms": 5000,
//      "polling_interval": 10000
//    },
//    {
//      "name": "Domain",
//      "type": "ping",
//      "target": "www.example.com",
//      "page_link": "https://example.com",
//      "timeout_ms": 5000,
//      "polling_interval": 10000
//    },
//    {
//      "name": "Database Connection",
//      "type": "port",
//      "target": "db.example.com",
//      "port": 5432,
//      "timeout_ms": 5000,
//      "polling_interval": 60000
//    }

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
            // TODO: Log error to history file
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
