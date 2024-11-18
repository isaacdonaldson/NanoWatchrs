use chrono::Utc;
use minijinja::{context, path_loader, Environment};

use std::fs;

use crate::utils::config::HistorySection;
use crate::{Check, HistoryEntry, Result, State, ASSETS_PATH, HISTORY_LENGTH, LONG_DATE_FORMAT};

pub fn create_env<'a>() -> Environment<'a> {
    let mut env = Environment::new();
    // Load the templates from the src/templates directory
    env.set_loader(path_loader("src/templates"));

    return env;
}

pub fn write_string_to_asset_folder(file_name: &str, content: &str) -> Result<()> {
    let full_path = format!("{}/{}", ASSETS_PATH, file_name);
    fs::write(full_path, content)?;
    Ok(())
}

// Block name comes from the "checks" in the config file
pub fn render_status_block<'a>(
    env: &Environment<'a>,
    check: &Check,
    history_section: HistorySection,
) -> Result<String> {
    let date_cutoff = Utc::now().naive_utc() - chrono::Duration::days((HISTORY_LENGTH + 1) as i64);

    let mut unsuccessful_checks = 0;

    let mut history = history_section
        .entries
        .iter()
        .filter(|entry| entry.date >= date_cutoff.date())
        .map(|entry| entry.to_owned())
        .map(|entry| {
            if entry.state != State::Success {
                unsuccessful_checks += 1;
            }
            entry
        })
        .collect::<Vec<HistoryEntry>>();

    let uptime = match history_section.uptime {
        Some(uptime) => uptime,
        None => (1.0 - (unsuccessful_checks as f64 / history.len() as f64)) * 100.0,
    };

    if history.len() < HISTORY_LENGTH as usize {
        for _ in 0..(HISTORY_LENGTH - history.len()) {
            let date = Utc::now().naive_utc() - chrono::Duration::days(history.len() as i64);
            history.insert(0, HistoryEntry::default_for_date(date.date()));
        }
    }

    let state = match history.last() {
        Some(entry) => entry.state.clone(),
        None => State::Disabled,
    };

    let display_status = state.to_status();

    let context = context! {
        title => check.name,
        subtitle => check.description,
        status => display_status,
        state => state.to_state(),
        updated_at => history_section.last_updated.format(LONG_DATE_FORMAT).to_string(),
        uptime => format!("{:.02}", uptime),
        history_line => history,
    };

    let template = env.get_template("partials/status.html.jinja")?;

    Ok(template.render(context)?)
}
