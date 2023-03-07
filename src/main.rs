use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

mod calendar;
mod models;
mod tracto;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const APP_NAME: &str = "calar";
const CONFIG_FILE: &str = "config";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub tracto_prefix: String,
    pub semester_start_m: u32,
    pub semester_start_d: u32,
    pub semester_end_m: u32,
    pub semester_end_d: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tracto_prefix: String::from("https://scribabot.tk/api/v1.0"),
            semester_start_m: 2,
            semester_start_d: 6,
            semester_end_m: 5,
            semester_end_d: 31,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
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

pub async fn validate_args(cfg: &Config, args: &Args) -> Result<()> {
    let available_departments: Vec<String> = tracto::fetch_departments(cfg)
        .await?
        .departments_list
        .into_iter()
        .map(|x| x.url)
        .collect();

    if !available_departments.contains(&args.department) {
        return Err(String::from("Incorrect department").into());
    }

    if !vec!["full", "extramural"].contains(&args.form.as_str()) {
        return Err(String::from("Incorrect education form").into());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg: Config = confy::load(APP_NAME, CONFIG_FILE)?;
    let args = Args::parse();
    validate_args(&cfg, &args).await?;

    let schedule = tracto::fetch_schedule(&cfg, &args).await?;
    let calendar = schedule.to_ical(&cfg, &args);

    let mut file = File::create(format!(
        "{}-{}-{}-{}{}.ics",
        args.department,
        args.form,
        args.group,
        args.subgroups.join("_"),
        if args.translator { "-t" } else { "" }
    ))?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(())
}
