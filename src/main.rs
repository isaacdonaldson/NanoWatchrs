use chrono::NaiveDateTime;
use minijinja::{context, path_loader, Environment};
use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "examples/basic.json";
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusPageContext {
    site: SiteParams,
    page: PageParams,
    status_blocks: Vec<StatusBlock>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SiteParams {
    name: String,
    description: String,
    url: String,
    logo: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PageParams {
    title: String,
    header: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusBlock {
    title: String,
    subtitle: String,
    status: String,
    state: State,
    #[serde(with = "date_format")]
    updated_at: NaiveDateTime,
    history_line: HistoryLine,
    uptime: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Success,
    Warning,
    Danger,
    Failure,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HistoryLine(pub String);

fn main() -> Result<()> {
    let mut env = Environment::new();
    env.set_loader(path_loader("src/templates"));

    // Example: Render the "hello.txt" template
    let config = read_config_file(CONFIG_PATH)
        .expect(format!("Failed to read config file at '{}'", CONFIG_PATH).as_str());

    let template = match env.get_template("index.html.jinja") {
        Ok(template) => template,
        Err(e) => {
            println!("Template 'index.html.jijna' not found.");
            return Err(Box::new(e));
        }
    };

    let context = context! {
        site => config.site,
        page => config.page,
        status_blocks => config.status_blocks,
    };

    println!("{}", template.render(context)?);

    // Example: Render a template from a subfolder
    // if let Ok(template) = env.get_template("subfolder/example.txt") {
    //     println!("{}", template.render(context! { value => "Test" })?);
    // } else {
    //     println!("Template 'subfolder/example.txt' not found.");
    // }

    Ok(())
}

fn read_config_file(config_path: &str) -> Result<StatusPageContext> {
    let config_file = std::fs::read_to_string(config_path)?;
    let config: StatusPageContext = serde_json::from_str(&config_file)?;
    Ok(config)
}

mod date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::DATE_FORMAT;

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let dt =
            NaiveDateTime::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)?;

        Ok(dt)
    }
}
