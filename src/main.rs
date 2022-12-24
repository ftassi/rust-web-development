use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::Method,
    http::StatusCode,
    reject::Reject,
    Filter, Rejection, Reply,
};

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    fn new() -> Self {
        Self {
            questions: Arc::new(RwLock::new(Self::init())),
        }
    }
    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../data/questions.json");
        serde_json::from_str(file).expect("Failed to parse questions.json")
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Debug)]
struct QuestionId(String);

#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    InvalidParameters,
    QuestionNotFound,
}

impl Reject for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "Cannot parse paramter: {}", e),
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::InvalidParameters => write!(f, "Start must be smaller than end"),
            Error::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

struct InvalidPagination;

impl Pagination {
    fn new(start: usize, end: usize) -> Result<Self, InvalidPagination> {
        if start > end {
            Err(InvalidPagination)
        } else {
            Ok(Self { start, end })
        }
    }
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let start = params
            .get("start")
            .unwrap()
            .parse::<usize>()
            .map_err(Error::ParseError)?;
        let end = params
            .get("end")
            .unwrap()
            .parse::<usize>()
            .map_err(Error::ParseError)?;
        let pagination = Pagination::new(start, end).map_err(|_| Error::InvalidParameters)?;
        Ok(pagination)
    } else {
        Err(Error::MissingParameters)
    }
}

fn limit_pagination<T>(pagination: Pagination, elements: &Vec<T>) -> Pagination {
    if pagination.end > elements.len() {
        Pagination {
            start: pagination.start,
            end: elements.len(),
        }
    } else {
        pagination
    }
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl Reply, Rejection> {
    let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let pagination = limit_pagination(pagination, &res);
        let res = &res[pagination.start..pagination.end];

        Ok(warp::reply::json(&res))
    } else {
        Ok(warp::reply::json(&res))
    }
}

async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);

    Ok(warp::reply::with_status(
        "Question added",
        StatusCode::CREATED,
    ))
}

async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }

    Ok(warp::reply::with_status(
        "Question updated",
        StatusCode::CREATED,
    ))
}

async fn delete_question(id: String, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::Ok)),
        None => Err(warp::reject::custom(Error::QuestionNotFound)),
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        if let Error::QuestionNotFound = error {
            return Ok(warp::reply::with_status(
                "Question not found".to_string(),
                StatusCode::NOT_FOUND,
            ));
        }

        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_header("content-type");

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
