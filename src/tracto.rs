use crate::{models::*, Config};
use crate::{Args, Result};

pub async fn fetch_schedule(cfg: &Config, cx: &Args) -> Result<Schedule> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/schedule/{}/{}/{}",
        cfg.tracto_prefix, cx.form, cx.department, cx.group
    );
    let schedule = client.get(url).send().await?.json::<Schedule>().await?;

    Ok(schedule)
}

pub async fn fetch_departments(cfg: &Config) -> Result<DepartmentsList> {
    let client = reqwest::Client::new();
    let departments = client
        .get(format!("{}/departments", cfg.tracto_prefix))
        .send()
        .await?
        .json::<DepartmentsList>()
        .await?;
    Ok(departments)
}

pub fn _find_subgroups(schedule: &Schedule) -> Vec<String> {
    let mut subgroups = schedule
        .lessons
        .iter()
        .map(|l| l.sub_group.trim().to_string())
        .filter(|sg| !sg.is_empty())
        .collect::<Vec<_>>();
    subgroups.sort_unstable();
    subgroups.dedup();
    subgroups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn try_fetch_departments() -> Result<()> {
        let cfg = Config::default();
        fetch_departments(&cfg).await?;
        Ok(())
    }

    #[tokio::test]
    async fn try_fetch_schedule_1() -> Result<()> {
        let cfg = Config::default();
        let args = Args {
            department: String::from("knt"),
            form: String::from("full"),
            group: String::from("351"),
            subgroups: vec![String::from("1_под."), String::from("цифровая_кафедра")],
            translator: false,
        };
        fetch_schedule(&cfg, &args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn try_fetch_schedule_2() -> Result<()> {
        let cfg = Config::default();
        let args = Args {
            department: String::from("knt"),
            form: String::from("full"),
            group: String::from("351"),
            subgroups: Vec::new(),
            translator: false,
        };
        fetch_schedule(&cfg, &args).await?;
        Ok(())
    }

    #[tokio::test]
    async fn try_fetch_schedule_3() -> Result<()> {
        let cfg = Config::default();
        let args = Args {
            department: String::from("knt"),
            form: String::from("full"),
            group: String::from("351"),
            subgroups: vec![
                String::from("2_под."),
                String::from("цифровая_кафедра"),
                String::from("анг.ст.3"),
            ],
            translator: true,
        };
        fetch_schedule(&cfg, &args).await?;
        Ok(())
    }
}
