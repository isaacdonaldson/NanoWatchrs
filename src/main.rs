use minijinja::context;
use nanowatchrs::utils::checks::run_check;

use nanowatchrs::utils::config::{read_config_file, read_history_file};
use nanowatchrs::utils::templates::{
    create_env, render_incident, render_status_block, write_string_to_asset_folder,
};
use nanowatchrs::CONFIG_PATH;
use nanowatchrs::{Result, StatusPageContext};

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = read_config_file(CONFIG_PATH)
        .expect(format!("Failed to read config file at '{}'", CONFIG_PATH).as_str());

    match parse_args() {
        RunMode::All => {}
        RunMode::Some(checks) => {
            // Only run the specified checks
            config.checks = config
                .checks
                .iter()
                .filter(|check| checks.contains(&check.name))
                .map(|check| check.clone())
                .collect();
        }
    };

    // Create new immutable StatusPageContext from the mutable config
    let config = StatusPageContext::from(config);

    for check in &config.checks {
        println!("Running check '{}'", check.name);
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
            Err(_) => {
                println!(
                    "History entry for '{}' is required but not found",
                    check.name
                );
                None
            }
            Ok(history) => render_status_block(&env, check, history.clone()).ok(),
        })
        .reduce(|a, b| format!("{}\n{}", a, b));

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
        .reduce(|a, b| format!("{}\n{}", a, b));

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
                fatal(format!("Unknown argument '{}'", arg).as_str());
            }
        }
    }

    if run_all {
        return RunMode::All;
    } else if checks.len() == 0 {
        fatal("specifiying a check with --check or -c is required");
    } else {
        return RunMode::Some(checks);
    }
}

fn fatal(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}
