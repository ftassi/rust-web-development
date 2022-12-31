use crate::domain::question::{Question, QuestionId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Store {
    pub questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    pub answers: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(Self::init())),
        }
    }
    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../../data/questions.json");
        serde_json::from_str(file).expect("Failed to parse questions.json")
    }
}
