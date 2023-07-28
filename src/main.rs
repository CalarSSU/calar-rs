use clap::{Parser, Subcommand};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{fs::File, io::Write, process::ExitCode};

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
async fn main() -> ExitCode {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let cfg = Config::default();
    let cli = Cli::parse();

    match cli.command {
        Command::Single(req) => make_single_request(cfg, req).await,
        Command::Server => server::run_server(cfg).await,
        Command::Prune => server::prune_cache(),
    }
}

async fn make_single_request(cfg: Config, req: Request) -> ExitCode {
    if let Err(e) = tracto::validate_request(&cfg, &req).await {
        eprintln!("Bad request: {e}");
        return ExitCode::FAILURE;
    }

    let schedule = match tracto::fetch_schedule(&cfg, &req).await {
        Ok(schedule) => schedule,
        Err(e) => {
            eprintln!("Cannot fetch schedule: {e}");
            return ExitCode::FAILURE;
        }
    };
    let calendar = schedule.to_ical(&cfg, &req);

    let mut file = match File::create(server::gen_filename::<models::Schedule>(&req)) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Cannot create file: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = file.write_all(calendar.to_string().as_bytes()) {
        eprintln!("Cannot write to file: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
