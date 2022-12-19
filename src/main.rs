use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use warp::{
    filters::cors::CorsForbidden, http::Method, http::StatusCode, reject::Reject, Filter,
    Rejection, Reply,
};

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}
impl Store {
    fn new() -> Self {
        Self {
            questions: Self::init(),
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
}

impl Reject for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "Cannot parse paramter: {}", e),
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::InvalidParameters => write!(f, "Start must be smaller than end"),
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
    let res: Vec<Question> = store.questions.values().cloned().collect();
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let pagination = limit_pagination(pagination, &res);
        let res = &res[pagination.start..pagination.end];

        Ok(warp::reply::json(&res))
    } else {
        Ok(warp::reply::json(&res))
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
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

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items.with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
