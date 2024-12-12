#![allow(clippy::missing_errors_doc, clippy::option_if_let_else)]
use chrono::{NaiveDateTime, Utc};
use minijinja::{context, path_loader, Environment};

use std::fs;

use crate::utils::config::HistorySection;
use crate::{
    Check, HistoryEntry, Incident, Result, State, ASSETS_PATH, DATE_FORMAT, HISTORY_LENGTH,
    LONG_DATE_FORMAT, TIME_FORMAT,
};

fn date(date_str: &str) -> String {
    match NaiveDateTime::parse_from_str(date_str, LONG_DATE_FORMAT) {
        Ok(date) => date.format(DATE_FORMAT).to_string(),
        Err(_) => date_str.to_owned(),
    }
}

fn time(date_str: &str) -> String {
    match NaiveDateTime::parse_from_str(date_str, LONG_DATE_FORMAT) {
        Ok(date) => date.format(TIME_FORMAT).to_string(),
        Err(_) => date_str.to_owned(),
    }
}

pub fn create_env<'a>() -> Environment<'a> {
    let mut env = Environment::new();
    // Load the templates from the src/templates directory
    env.set_loader(path_loader("src/templates"));

    // Add custom filters
    env.add_filter("date", date);
    env.add_filter("time", time);

    env
}

pub fn write_string_to_asset_folder(file_name: &str, content: &str) -> Result<()> {
    let full_path = format!("{ASSETS_PATH}/{file_name}");
    fs::write(full_path, content)?;
    Ok(())
}

// Block name comes from the "checks" in the config file
pub fn render_status_block(
    env: &Environment<'_>,
    check: &Check,
    history_section: &HistorySection,
) -> Result<String> {
    #[allow(clippy::cast_possible_wrap)]
    let date_cutoff = Utc::now().naive_utc() - chrono::Duration::days((HISTORY_LENGTH) as i64);

    let mut unsuccessful_checks = 0;

    let incomplete_history = history_section
        .entries
        .iter()
        .filter(|entry| entry.date > date_cutoff.date())
        .map(std::borrow::ToOwned::to_owned)
        .inspect(|entry| {
            if entry.state != State::Success {
                unsuccessful_checks += 1;
            }
        })
        .collect::<Vec<HistoryEntry>>();

    let uptime = match history_section.uptime {
        Some(uptime) => uptime,
        #[allow(clippy::cast_precision_loss)]
        None => (1.0 - (f64::from(unsuccessful_checks) / incomplete_history.len() as f64)) * 100.0,
    };

    let mut history = vec![];

    for idx in 0..HISTORY_LENGTH {
        #[allow(clippy::cast_possible_wrap)]
        let date = (Utc::now().naive_utc() - chrono::Duration::days(idx as i64)).date();

        let matching_entry = incomplete_history
            .iter()
            .find(|entry| entry.date == date)
            .cloned()
            .unwrap_or_else(|| HistoryEntry::default_for_date(date));

        history.push(matching_entry);
    }

    let history = history.into_iter().rev().collect::<Vec<HistoryEntry>>();

    let state = match history.last() {
        Some(entry) => entry.state.clone(),
        None => State::Disabled,
    };

    let display_status = state.to_status();

    let context = context! {
        title => check.name,
        subtitle => check.description,
        page_link => check.page_link,
        status => display_status,
        state => state.to_state(),
        updated_at => history_section.last_updated.format(LONG_DATE_FORMAT).to_string(),
        uptime => format!("{:.02}", uptime),
        history_line => history,
    };

    let template = env.get_template("partials/status.html.jinja")?;

    let rendered = template.render(context);

    match rendered {
        Ok(rendered) => Ok(rendered),
        Err(e) => {
            eprintln!("Template Render Error: {e:#?}");
            Err(e.into())
        }
    }
}

// Incidents are defined as text in the config.json file
pub fn render_incident(env: &Environment<'_>, incident: &Incident) -> Result<String> {
    let state = match incident.status.as_str() {
        "Resolved" | "resolved" => "success",
        "Ongoing" | "ongoing" => "warning",
        _ => "",
    };
    // TODO: status enum for color
    // TODO: longer form text, WYSIWYG? Markdown?
    let context = context! {
        title => incident.title,
        description => format_incident_description(&incident.description),
        state => state,
        status => incident.status,
        display_date => incident.display_date,
        started_at => incident.started_at.format(LONG_DATE_FORMAT).to_string(),
        resolved_at => incident.resolved_at.format(LONG_DATE_FORMAT).to_string(),
    };

    let template = env.get_template("partials/incident.html.jinja")?;

    let rendered = template.render(context);

    match rendered {
        Ok(rendered) => Ok(rendered),
        Err(e) => {
            eprintln!("Template Render Error: {e:#?}");
            Err(e.into())
        }
    }
}

fn format_incident_description(description: &str) -> String {
    description.split('\n').collect::<Vec<&str>>().join("<br>")
}
