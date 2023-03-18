use clap::{Parser, Subcommand};
use std::{fs::File, io::Write};

mod calendar;
mod config;
mod models;
mod server;
mod tracto;

use config::*;

#[derive(Debug, Parser)]
#[clap(name = "my-app", version)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Get single calendar
    Single(Request),
    /// Run as web server
    Server,
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

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_config_dir()?;
    let app = Cli::parse();

    match app.command {
        Command::Single(req) => make_single_request(&cfg, req).await?,
        Command::Server => server::run_server(&cfg).await?,
    };

    Ok(())
}

async fn make_single_request(cfg: &Config, req: Request) -> Result<()> {
    validate_request(cfg, &req).await?;

    let schedule = tracto::fetch_schedule(cfg, &req).await?;
    let calendar = schedule.to_ical(cfg, &req);

    let mut file = File::create(format!(
        "{}-{}-{}-{}{}.ics",
        req.department,
        req.form,
        req.group,
        req.subgroups.join("_"),
        if req.translator { "-t" } else { "" }
    ))?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(())
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
