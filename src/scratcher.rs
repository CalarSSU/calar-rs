use serde::{Deserialize, Serialize};

use crate::{Args, AsyncResult};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    lessons: Vec<Lesson>,
    student_group: StudentGroup,
    day: Day,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    id: u32,
    name: String,
    place: String,
    department: Department,
    student_group: StudentGroup,
    sub_group: String,
    day: Day,
    lesson_time: LessonTime,
    teacher: Teacher,
    week_type: String,
    lesson_type: String,
    updated_timestamp: u32,
    begin_timestamp: Option<u32>,
    end_timestamp: Option<u32>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Department {
    id: u32,
    full_name: String,
    short_name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentGroup {
    id: u32,
    group_number: String,
    group_number_rus: String,
    department: Department,
    education_form: String,
    group_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    id: Option<u32>,
    day_number: u16,
    week_day: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonTime {
    id: u32,
    lesson_number: u8,
    hour_start: u8,
    minute_start: u8,
    hour_end: u8,
    minute_end: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Teacher {
    id: u32,
    surname: String,
    name: String,
    patronymic: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentsList {
    pub departments_list: Vec<Department>,
}

const TRACTO_PREFIX: &str = "https://scribabot.tk/api/v1.0";

pub async fn get_schedule(ctx: &Args) -> AsyncResult<Schedule> {
    let client = reqwest::Client::new();
    let url = format!(
        "{TRACTO_PREFIX}/schedule/{}/{}/{}",
        ctx.form, ctx.department, ctx.group
    );
    let schedule = client.get(url).send().await?.json::<Schedule>().await?;

    Ok(schedule)
}

pub async fn get_departments() -> AsyncResult<DepartmentsList> {
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
