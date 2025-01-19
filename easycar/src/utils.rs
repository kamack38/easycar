use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;

pub fn date_from_string(timestamp: &str) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse timestamp");
    let datetime_cest: DateTime<chrono_tz::Tz> =
        Warsaw.from_local_datetime(&naive_datetime).unwrap();
    datetime_cest.with_timezone(&Utc)
}

pub fn readable_time_delta(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    format!("{days} days, {hours} hours, {minutes} minutes, {seconds} seconds")
}
