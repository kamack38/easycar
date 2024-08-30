use crate::client::exam_schedule::{Exam, ExamSchedule};

pub fn find_first_non_empty_practice_exam(schedule: &ExamSchedule) -> Option<&Vec<Exam>> {
    for day in &schedule.schedule.scheduled_days {
        for hour in &day.scheduled_hours {
            if !hour.practice_exams.is_empty() {
                return Some(&hour.practice_exams);
            }
        }
    }
    None
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
