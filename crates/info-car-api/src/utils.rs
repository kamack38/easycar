use crate::types::{Exam, ExamSchedule};
use std::ops::Not;

pub fn find_n_practice_exams(schedule: ExamSchedule, number: usize) -> Option<Vec<Exam>> {
    let exams: Vec<Exam> = schedule
        .schedule
        .scheduled_days
        .into_iter()
        .flat_map(|day| day.scheduled_hours.into_iter())
        .flat_map(|hour| hour.practice_exams.into_iter())
        .take(number)
        .collect();
    exams.is_empty().not().then(|| exams)
}

pub fn find_all_practice_exams(schedule: &ExamSchedule) -> Vec<&Exam> {
    let mut all_practice_exams = Vec::new();

    for day in &schedule.schedule.scheduled_days {
        for hour in &day.scheduled_hours {
            for exam in &hour.practice_exams {
                all_practice_exams.push(exam);
            }
        }
    }

    all_practice_exams
}
