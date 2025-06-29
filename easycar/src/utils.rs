use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;

pub fn date_from_string(timestamp: &str) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")
        .expect("Failed to parse timestamp");
    let datetime_cest: DateTime<chrono_tz::Tz> =
        Warsaw.from_local_datetime(&naive_datetime).unwrap();
    datetime_cest.with_timezone(&Utc)
}

/// Returns a readable date with hours in bold and underlined. If it fails to convert the timestamp
/// to a date returns the provided timestamp.
pub fn readable_date_from_string(timestamp: String) -> String {
    NaiveDateTime::parse_from_str(&timestamp, "%Y-%m-%dT%H:%M:%S")
        .map(|date| date.format("<u><b>%H:%M</b></u>%e-%m-%Y").to_string())
        .unwrap_or(timestamp)
}

pub fn readable_time_delta(duration: Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    format!("{days} days, {hours} hours, {minutes} minutes, {seconds} seconds")
}
