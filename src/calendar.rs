use crate::models::*;

use chrono::prelude::*;
use chrono_tz::Europe::Saratov;
use icalendar::*;

const SEMESTER_START_MONTH: u32 = 2;
const SEMESTER_START_DAY: u32 = 6;

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
        let event_start = CalendarDateTime::from_ymd_hm_tzid(
            cur_year,
            SEMESTER_START_MONTH,
            SEMESTER_START_DAY + self.day.day_number - 1,
            self.lesson_time.hour_start,
            self.lesson_time.minute_start,
            Saratov,
        )
        .unwrap();
        let event_end = CalendarDateTime::from_ymd_hm_tzid(
            cur_year,
            SEMESTER_START_MONTH,
            SEMESTER_START_DAY + self.day.day_number - 1,
            self.lesson_time.hour_end,
            self.lesson_time.minute_end,
            Saratov,
        )
        .unwrap();

        Event::new()
            .summary(self.name.as_str())
            .starts(event_start)
            .ends(event_end)
            .done()
    }
}
