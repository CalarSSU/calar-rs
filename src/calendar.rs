use crate::models::*;

use chrono::prelude::*;
use chrono_tz::Europe::Saratov;
use icalendar::*;

const SEM_START_MONTH: u32 = 2;
const SEM_START_DAY: u32 = 6;
const SEM_END_MONTH: u32 = 5;
const SEM_END_DAY: u32 = 31;

impl Schedule {
    pub fn to_ical(&self) -> Calendar {
        let mut cal = Calendar::new();
        for lesson in &self.lessons {
            cal.push(lesson.to_event());
        }

        cal.done()
    }
}

impl Lesson {
    fn to_event(&self) -> Event {
        let cur_year = Utc::now().with_timezone(&Saratov).date_naive().year();
        let mut event_start = Saratov
            .with_ymd_and_hms(
                cur_year,
                SEM_START_MONTH,
                SEM_START_DAY + self.day.day_number - 1,
                self.lesson_time.hour_start,
                self.lesson_time.minute_start,
                0,
            )
            .unwrap();
        let mut event_end = Saratov
            .with_ymd_and_hms(
                cur_year,
                SEM_START_MONTH,
                SEM_START_DAY + self.day.day_number - 1,
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
        let rrule_end = NaiveDate::from_ymd_opt(cur_year, SEM_END_MONTH, SEM_END_DAY)
            .unwrap()
            .format("%Y%m%dT235959")
            .to_string();
        let rrule = format!("FREQ=WEEKLY;INTERVAL={interval};UNTIL={rrule_end}");

        // This logic below uses the fact that every odd week is NOM
        // and every even week should be DENOM
        let first_week_of_sem =
            chrono::NaiveDate::from_ymd_opt(cur_year, SEM_START_MONTH, SEM_START_DAY)
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
