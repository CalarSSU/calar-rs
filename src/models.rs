use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub lessons: Vec<Lesson>,
    pub student_group: StudentGroup,
    pub day: Day,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub id: u32,
    pub name: String,
    pub place: String,
    pub department: Department,
    pub student_group: StudentGroup,
    pub sub_group: String,
    pub day: Day,
    pub lesson_time: LessonTime,
    pub teacher: Teacher,
    pub week_type: String,
    pub lesson_type: String,
    pub updated_timestamp: u32,
    pub begin_timestamp: Option<u32>,
    pub end_timestamp: Option<u32>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Department {
    pub id: u32,
    pub full_name: String,
    pub short_name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentGroup {
    pub id: u32,
    pub group_number: String,
    pub group_number_rus: String,
    pub department: Department,
    pub education_form: String,
    pub group_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub id: Option<u32>,
    pub day_number: u32,
    pub week_day: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonTime {
    pub id: u32,
    pub lesson_number: u8,
    pub hour_start: u32,
    pub minute_start: u32,
    pub hour_end: u32,
    pub minute_end: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Teacher {
    pub id: u32,
    pub surname: String,
    pub name: String,
    pub patronymic: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentsList {
    pub departments_list: Vec<Department>,
}
