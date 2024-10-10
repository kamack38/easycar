use crate::types::{Exam, ExamSchedule};

pub fn find_n_practice_exams(schedule: ExamSchedule, number: usize) -> Option<Vec<Exam>> {
    let exams: Vec<Exam> = schedule
        .schedule
        .scheduled_days
        .into_iter()
        .flat_map(|day| day.scheduled_hours.into_iter())
        .flat_map(|hour| hour.practice_exams.into_iter())
        .take(number)
        .collect();
    (!exams.is_empty()).then_some(exams)
}

pub fn find_all_practice_exams(schedule: &ExamSchedule) -> Vec<&Exam> {
    schedule
        .schedule
        .scheduled_days
        .iter()
        .flat_map(|day| day.scheduled_hours.iter())
        .flat_map(|hour| hour.practice_exams.iter())
        .collect()
}
