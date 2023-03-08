use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
};

mod calendar;
mod models;
mod tracto;

const QUALIFIER: &str = "dev";
const APP_NAME: &str = "calar";
const ORG_NAME: &str = "calar";
const CONFIG_FILE: &str = "config.toml";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub tracto_prefix: String,
    pub semester: Semester,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Semester {
    start_md: (u32, u32),
    end_md: (u32, u32),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tracto_prefix: String::from("https://scribabot.tk/api/v1.0"),
            semester: Semester {
                start_md: (2, 6),
                end_md: (5, 31),
            },
        }
    }
}

impl Config {
    pub fn from_config_dir() -> Result<Config> {
        let proj_dirs = ProjectDirs::from(QUALIFIER, ORG_NAME, APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");
        let config_file = proj_dirs.config_dir().join(CONFIG_FILE);

        let mut config_file = match File::open(config_file) {
            Ok(file) => file,
            Err(_) => {
                let cfg = Config::default();
                cfg.dump_to_config_dir()?;
                return Ok(cfg);
            }
        };

        let mut config_content = String::new();
        config_file.read_to_string(&mut config_content)?;

        Ok(toml::from_str(&config_content)?)
    }

    pub fn dump_to_config_dir(&self) -> Result<()> {
        let serialized = toml::to_string(self)?;

        let proj_dirs = ProjectDirs::from("dev", "calar", APP_NAME)
            .expect("No valid config directory could be retrieved from the operating system");
        let config_dir = proj_dirs.config_dir();
        create_dir_all(config_dir)?;
        let config_file = config_dir.join(CONFIG_FILE);

        let mut config_file = File::create(config_file)?;
        config_file.write_all(serialized.as_bytes())?;

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Request {
    #[arg(short, long)]
    pub department: String,
    #[arg(short, long)]
    pub form: String,
    #[arg(short, long)]
    pub group: String,
    #[arg(short, long, num_args(0..))]
    pub subgroups: Vec<String>,
    #[arg(short, long)]
    pub translator: bool,
}

pub async fn validate_request(cfg: &Config, request: &Request) -> Result<()> {
    let available_departments: Vec<String> = tracto::fetch_departments(cfg)
        .await?
        .departments_list
        .into_iter()
        .map(|x| x.url)
        .collect();

    if !available_departments.contains(&request.department) {
        return Err(String::from("Incorrect department").into());
    }

    if !vec!["full", "extramural"].contains(&request.form.as_str()) {
        return Err(String::from("Incorrect education form").into());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_config_dir()?;
    let request = Request::parse();
    validate_request(&cfg, &request).await?;

    let schedule = tracto::fetch_schedule(&cfg, &request).await?;
    let calendar = schedule.to_ical(&cfg, &request);

    let mut file = File::create(format!(
        "{}-{}-{}-{}{}.ics",
        request.department,
        request.form,
        request.group,
        request.subgroups.join("_"),
        if request.translator { "-t" } else { "" }
    ))?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(())
}
