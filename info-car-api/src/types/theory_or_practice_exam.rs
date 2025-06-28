use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TheoryOrPracticeExam {
    pub exam_id: String,
    pub date: String,
    pub additional_info: Option<String>,
    pub room: Option<String>,
}
