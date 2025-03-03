use minijinja::context;
use nanowatchrs::utils::checks::run_check;

use nanowatchrs::utils::config::{
    create_history_file, does_history_file_exist, read_config_file, read_history_file,
};
use nanowatchrs::utils::templates::{
    create_env, render_incident, render_status_block, write_string_to_asset_folder,
};
use nanowatchrs::CONFIG_PATH;
use nanowatchrs::{Check, Result, StatusPageContext};

#[tokio::main]
async fn main() -> Result<()> {
    let config = read_config_file(CONFIG_PATH)
        .unwrap_or_else(|_| panic!("Failed to read config file at '{CONFIG_PATH}'"));

    let filtered_checks: Vec<Check> = match parse_args() {
        RunMode::All => config.checks.iter().cloned().collect(),
        RunMode::Some(checks) => {
            // Only run the specified checks
            config
                .checks
                .iter()
                .filter(|check| checks.contains(&check.name))
                .cloned()
                .collect()
        }
    };

    for check in &filtered_checks {
        println!("Running check '{}'", check.name);
        match does_history_file_exist(&check.name) {
            // Match on file does not exist
            Err(e) => {
                println!(
                    "Error encountered looking for history file for '{}': '{:#?}'",
                    check.name, e
                );
            }
            Ok(false) => {
                println!("No history file found for '{}', creating one", check.name);
                match create_history_file(&check.name) {
                    Err(e) => {
                        println!(
                            "Error encountered creating history file for '{}': '{:#?}'",
                            check.name, e
                        );
                    }
                    Ok(_) => {
                        println!("Successfully created history file for '{}'", check.name);
                    }
                }
            }
            Ok(true) => (),
        };
        let _ = run_check(check).await?;
    }

    run_template_rendering(&config)?;

    Ok(())
}

fn run_template_rendering(config: &StatusPageContext) -> Result<()> {
    let env = create_env();

    let template = env.get_template("index.html.jinja")?;

    let status_blocks: Option<String> = config
        .checks
        .iter()
        .filter_map(|check| match read_history_file(&check.name) {
            Err(e) => {
                println!(
                    "Error encountered reading history entry for '{}': '{:#?}'",
                    check.name, e
                );
                None
            }
            Ok(history) => render_status_block(&env, check, &history).ok(),
        })
        .reduce(|a, b| format!("{a}\n{b}"));

    if status_blocks.is_none() {
        return Err("Error rendering status blocks".into());
    }

    let incident_rendering = config
        .incidents
        .iter()
        .filter_map(|incident| match render_incident(&env, incident) {
            Err(e) => {
                println!("Error rendering incident '{}': '{:#?}'", incident.title, e);
                None
            }
            Ok(template) => Some(template),
        })
        .reduce(|a, b| format!("{a}\n{b}"));

    let context = context! {
        site => config.settings.site,
        page => config.settings.page,
        incidents => format!("{:#?}", config.incidents),
        rendered_blocks => status_blocks.unwrap(),
        incidents => incident_rendering.unwrap(),
    };

    let _ = write_string_to_asset_folder("index.html", &template.render(context)?);

    Ok(())
}

enum RunMode {
    Some(Vec<String>),
    All,
}

fn parse_args() -> RunMode {
    let mut checks = vec![];
    let mut run_all = false;

    let mut args = std::env::args();
    let _program_name = args.next();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" | "--check" => {
                let Some(value) = args.next() else {
                    fatal("--check: a check name value is required in order to be run");
                };
                checks.push(value);
            }
            "-a" | "--all" => {
                run_all = true;
            }
            _ => {
                fatal(format!("Unknown argument '{arg}'").as_str());
            }
        }
    }

    if run_all {
        RunMode::All
    } else if checks.is_empty() {
        fatal("specifiying a check with --check or -c is required");
    } else {
        RunMode::Some(checks)
    }
}

fn fatal(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}
