use clap::Parser;

mod models;
mod tracto;

pub type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub department: String,
    #[arg(short, long)]
    pub form: String,
    #[arg(short, long)]
    pub group: String,
    #[arg(short, long)]
    pub subgroup: Option<String>,
}

pub async fn validate_args(args: &Args) -> AsyncResult<()> {
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
async fn main() -> AsyncResult<()> {
    let args = Args::parse();
    validate_args(&args).await?;

    let s = tracto::fetch_schedule(&args).await?;

    println!("{s:#?}");

    Ok(())
}
