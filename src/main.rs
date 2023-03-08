use clap::Parser;
use std::{fs::File, io::Write};

mod calendar;
mod config;
mod models;
mod tracto;

use config::*;

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
