use clap::Parser;
use std::{fs::File, io::Write};

mod calendar;
mod models;
mod tracto;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
}

pub async fn validate_args(args: &Args) -> Result<()> {
    let available_departments: Vec<String> = tracto::fetch_departments()
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
    let args = Args::parse();
    validate_args(&args).await?;

    let schedule = tracto::fetch_schedule(&args).await?;
    let calendar = schedule.to_ical();

    let filename = format!(
        "{}_{}_{}_{}.ics",
        args.department,
        args.form,
        args.group,
        args.subgroups.join("_")
    );
    let mut file = File::create(filename)?;
    file.write_fmt(format_args!("{calendar}"))?;

    Ok(())
}
