use crate::domain::error;
use crate::domain::question::{Question, QuestionId};
use crate::infrastructure::pagination::{extract_pagination, limit_pagination};
use crate::infrastructure::store::Store;
use std::collections::HashMap;
use warp::{http::StatusCode, Rejection, Reply};

pub async fn get_questions(
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

pub async fn add_question(store: Store, question: Question) -> Result<impl Reply, Rejection> {
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

pub async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(error::Error::QuestionNotFound)),
    }

    Ok(warp::reply::with_status(
        "Question updated",
        StatusCode::CREATED,
    ))
}

pub async fn delete_question(id: String, store: Store) -> Result<impl Reply, Rejection> {
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(_) => Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => Err(warp::reject::custom(error::Error::QuestionNotFound)),
    }
}
