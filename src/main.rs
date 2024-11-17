use minijinja::context;

use std::collections::HashMap;

use nanowatchrs::utils::config::{read_config_file, read_history_file, HistorySection};
use nanowatchrs::utils::templates::{
    create_env, render_status_block, write_string_to_asset_folder,
};
use nanowatchrs::Result;
use nanowatchrs::{CONFIG_PATH, HISTORY_PATH};

fn main() -> Result<()> {
    let env = create_env();

    // Example: Render the "hello.txt" template
    let config = read_config_file(CONFIG_PATH)
        .expect(format!("Failed to read config file at '{}'", CONFIG_PATH).as_str());

    let history = read_history_file(HISTORY_PATH)
        .expect(format!("Failed to read history file at '{}'", HISTORY_PATH).as_str())
        .watchers
        .iter()
        .map(|e| (e.name.clone(), e.clone()))
        .collect::<HashMap<String, HistorySection>>();

    let template = env.get_template("index.html.jinja")?;

    // println!("Config: {:#?}", config);
    // println!("History: {:#?}", history);

    let status_blocks: Option<String> = config
        .checks
        .iter()
        .filter_map(|check| match history.get(&check.name) {
            None => {
                println!(
                    "History entry for '{}' is required but not found",
                    check.name
                );
                None
            }
            Some(history) => {
                // println!("Rendering status block for '{}'", check.name);
                // println!("{:#?}", history);
                render_status_block(&env, check, history.clone()).ok()
            }
        })
        .reduce(|a, b| format!("{}\n{}", a, b));

    if status_blocks.is_none() {
        return Err("Error rendering status blocks".into());
    }

    let context = context! {
        site => config.settings.site,
        page => config.settings.page,
        incidents => format!("{:#?}", config.incidents),
        // history => history,
        rendered_blocks => status_blocks.unwrap(),
    };

    let _ = write_string_to_asset_folder("index.html", &template.render(context)?);

    // let rendered_status_blocks = config
    //     .status_blocks
    //     .iter()
    //     .map(|block| {
    //         let complete_events = generate_history(&block.history_line);
    //         render_status(&env, block.clone(), complete_events)
    //     })
    //     .collect::<Result<Vec<String>>>()?
    //     .join("\n");

    // let context = context! {
    //     site => config.site,
    //     page => config.page,
    //     status_blocks => config.status_blocks,
    //     rendered_status_blocks => rendered_status_blocks,
    // };

    // let _ = write_string_to_asset_folder("index.html", &template.render(context)?);

    Ok(())
}

// fn generate_history(events: &Vec<HistoryEntry>) -> Vec<HistoryEntry> {
//     let today = chrono::Local::now().date_naive();
//     let mut history_map: HashMap<NaiveDate, HistoryEntry> = events
//         .iter()
//         .map(|entry| (entry.date, entry.clone()))
//         .collect();

//     let mut result = Vec::with_capacity(HISTORY_LENGTH);

//     for days_ago in 0..HISTORY_LENGTH {
//         let date = today - chrono::Duration::days(days_ago as i64);
//         let entry = history_map.entry(date).or_insert_with(|| HistoryEntry {
//             date,
//             state: State::Success,
//             notes: "No Incident".to_string(),
//         });
//         result.push(entry.clone());
//     }

//     result.reverse(); // To have the oldest date first
//     result
// }

// fn render_status(
//     env: &Environment,
//     block: StatusBlock,
//     complete_events: Vec<HistoryEntry>,
// ) -> Result<String> {
//     let template = match env.get_template("partials/status.html.jinja") {
//         Ok(template) => template,
//         Err(e) => {
//             println!("Template 'partials/status.html.jijna' not found.");
//             return Err(Box::new(e));
//         }
//     };

//     let ctx = context! {
//         title => block.title,
//         subtitle => block.subtitle,
//         status => block.status,
//         state => block.state,
//         updated_at => block.updated_at,
//         uptime => block.uptime,
//         history_line => complete_events,
//     };

//     template
//         .render(ctx)
//         .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
// }
