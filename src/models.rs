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

impl Lesson {
    pub fn summary(&self) -> String {
        let type_letter = match self.lesson_type.as_str() {
            "LECTURE" => 'Л',
            _ => 'П',
        };
        format!("{} ({})", self.name, type_letter)
    }
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

impl Teacher {
    pub fn full(&self) -> String {
        format!("{} {} {}", self.surname, self.name, self.patronymic)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentsList {
    pub departments_list: Vec<Department>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamList {
    pub exam_period_events: Vec<ExamEvent>,
    pub student_group: StudentGroup,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamEvent {
    pub id: u32,
    pub exam_period_event_type: String,
    pub day: u32,
    pub month: Month,
    pub year: String,
    pub hour: u32,
    pub minute: u32,
    pub subject_name: String,
    pub teacher: Teacher,
    pub student_group: StudentGroup,
    pub place: String,
}

impl ExamEvent {
    pub fn summary(&self) -> String {
        let type_letter = match self.exam_period_event_type.as_str() {
            "CONSULTATION" => "Консультация",
            "EXAM" => "Экзамен",
            "MIDTERM_WITH_MARK" => "Зачет с оценкой",
            _ => "Зачет",
        };
        format!("{} ({})", self.subject_name, type_letter)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Month {
    pub number: u32,
    pub rus_nominative: String,
    pub rus_genitive: String,
    pub eng: String,
}