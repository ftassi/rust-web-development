use warp::{
    body::BodyDeserializeError, cors::CorsForbidden, hyper::StatusCode, reject::Reject, Rejection,
    Reply,
};

#[derive(Debug)]
pub enum Error {
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
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
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
