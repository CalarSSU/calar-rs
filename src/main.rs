use clap::{Parser, Subcommand};
use std::{fs::File, io::Write};

mod calendar;
mod config;
mod models;
mod server;
mod tracto;

use config::*;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
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
    /// Clear all cache
    Prune,
}

#[derive(Parser, Debug)]
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

#[actix_web::main]
async fn main() -> Result<()> {
    let cfg = Config::from_config_dir()?;
    let cli = Cli::parse();

    match cli.command {
        Command::Single(req) => make_single_request(cfg, req).await?,
        Command::Server => server::run_server(cfg).await?,
        Command::Prune => server::prune_cache()?,
    }

    Ok(())
}

async fn make_single_request(cfg: Config, req: Request) -> Result<()> {
    validate_request(&cfg, &req).await?;

    let schedule = tracto::fetch_schedule(&cfg, &req).await?;
    let calendar = schedule.to_ical(&cfg, &req);

    let mut file = File::create(server::gen_filename(&req))?;
    file.write_all(calendar.to_string().as_bytes())?;

    Ok(())
}

pub async fn validate_request(cfg: &Config, req: &Request) -> Result<()> {
    let available_departments: Vec<String> = tracto::fetch_departments(cfg)
        .await?
        .departments_list
        .into_iter()
        .map(|x| x.url)
        .collect();
    let schedule = tracto::fetch_schedule(&cfg, &req)
        .await?;

    let subgroups = tracto::find_subgroups(&schedule);

    if !available_departments.contains(&req.department) {
        return Err(String::from("Incorrect department").into());
    }

    if !vec!["full", "extramural"].contains(&req.form.as_str()) {
        return Err(String::from("Incorrect education form").into());
    }

    if req.subgroups.iter().any(|x| !subgroups.contains(x)) {
        return Err(String::from("Incorrect subgroup(s)").into());
    }

    Ok(())
}
