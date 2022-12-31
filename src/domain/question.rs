use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct QuestionId(pub String);
