use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Answer {
    id: AnswerId,
    content: String,
    answer_id: AnswerId,
}
#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct AnswerId(String);
