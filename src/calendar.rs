use crate::{config::*, models::*, Request};

use chrono::{prelude::*, Duration};
use chrono_tz::Europe::Saratov;
use icalendar::*;

impl Schedule {
    pub fn to_ical(&self, cfg: &Config, request: &Request) -> Calendar {
        let mut cal = Calendar::new();
        for lesson in &self.lessons {
            let same_subgroup = request
                .subgroups
                .contains(&lesson.sub_group.trim().to_string());
            if (request.subgroups.is_empty() || lesson.sub_group.is_empty() || same_subgroup)
                && (!lesson.name.contains(&cfg.translator_substr) || request.translator)
            {
                cal.push(lesson.to_event(cfg));
            }
        }

        cal.done()
    }
}

impl Lesson {
    fn to_event(&self, cfg: &Config) -> Event {
        let cur_year = Utc::now().with_timezone(&Saratov).date_naive().year();
        let mut event_start = Saratov
            .with_ymd_and_hms(
                cur_year,
                cfg.semester.start_md.0,
                cfg.semester.start_md.1 + self.day.day_number - 1,
                self.lesson_time.hour_start,
                self.lesson_time.minute_start,
                0,
            )
            .unwrap();
        let mut event_end = Saratov
            .with_ymd_and_hms(
                cur_year,
                cfg.semester.start_md.0,
                cfg.semester.start_md.1 + self.day.day_number - 1,
                self.lesson_time.hour_end,
                self.lesson_time.minute_end,
                0,
            )
            .unwrap();

        // Interval is steps in weeks for every recurring event.
        // If week_type is FULL, lesson occurs each week
        // otherwise every other week
        let interval = match self.week_type.as_str() {
            "FULL" => 1,
            _ => 2,
        };
        let rrule_end =
            NaiveDate::from_ymd_opt(cur_year, cfg.semester.end_md.0, cfg.semester.end_md.1)
                .unwrap()
                .format("%Y%m%dT235959")
                .to_string();
        let rrule = format!("FREQ=WEEKLY;INTERVAL={interval};UNTIL={rrule_end}");

        // This logic below uses the fact that every odd week is NOM
        // and every even week should be DENOM
        let first_week_of_sem = chrono::NaiveDate::from_ymd_opt(
            cur_year,
            cfg.semester.start_md.0,
            cfg.semester.start_md.1,
        )
        .unwrap()
        .iso_week()
        .week();
        if first_week_of_sem % 2 == 0 && self.week_type == "NOM"
            || first_week_of_sem % 2 == 1 && self.week_type == "DENOM"
        {
            event_start += chrono::Duration::weeks(1);
            event_end += chrono::Duration::weeks(1);
        }

        Event::new()
            .starts(CalendarDateTime::from_date_time(event_start))
            .ends(CalendarDateTime::from_date_time(event_end))
            .summary(self.summary().as_str())
            .description(self.teacher.full().as_str())
            .location(self.place.as_str())
            .append_property(Property::new("RRULE", rrule.as_str()).done())
            .done()
    }
}

impl ExamList {
    pub fn to_ical(&self) -> Calendar {
        let mut calendar = Calendar::new();
        for exam in &self.exam_period_events {
            calendar.push(exam.to_event());
        }
        calendar.done()
    }
}

impl ExamEvent {
    fn to_event(&self) -> Event {
        let cur_year = self.year.replace("г.", "").parse::<i32>().unwrap();
        let event_start = Saratov
            .with_ymd_and_hms(
                cur_year,
                self.month.number,
                self.day,
                self.hour,
                self.minute,
                0,
            )
            .unwrap();
        let event_end = event_start + Duration::hours(2);
        Event::new()
            .starts(CalendarDateTime::from_date_time(event_start))
            .ends(CalendarDateTime::from_date_time(event_end))
            .summary(self.summary().as_str())
            .description(self.teacher.full().as_str())
            .location(self.place.as_str())
            .done()
    }
}
