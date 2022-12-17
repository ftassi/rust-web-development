use std::io::{Error, ErrorKind};
use std::str::FromStr;
use warp::Filter;

#[derive(Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug)]
struct QuestionId(String);

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id: id,
            title: title,
            content: content,
            tags: tags,
        }
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            true => Err(Error::new(ErrorKind::InvalidInput, "No ID provided")),
            false => Ok(QuestionId(id.to_string())),
        }
    }
}

#[tokio::main]
async fn main() {
    let hello = warp::get().map(|| format!("Hello, World!"));

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
