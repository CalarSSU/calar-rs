use crate::models::*;
use crate::{Args, AsyncResult};

const TRACTO_PREFIX: &str = "https://scribabot.tk/api/v1.0";

pub async fn fetch_schedule(ctx: &Args) -> AsyncResult<Schedule> {
    let client = reqwest::Client::new();
    let url = format!(
        "{TRACTO_PREFIX}/schedule/{}/{}/{}",
        ctx.form, ctx.department, ctx.group
    );
    let schedule = client.get(url).send().await?.json::<Schedule>().await?;

    Ok(schedule)
}

pub async fn fetch_departments() -> AsyncResult<DepartmentsList> {
    let client = reqwest::Client::new();
    let departments = client
        .get(format!("{TRACTO_PREFIX}/departments"))
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
    subgroups
}
