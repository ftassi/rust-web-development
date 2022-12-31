mod domain;
mod infrastructure;

use crate::domain::error;
use crate::infrastructure::actions;
use crate::infrastructure::store::Store;

use std::collections::HashMap;
use warp::{http::Method, Filter};

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
        .and_then(actions::get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(actions::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(actions::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(actions::delete_question);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(error::return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
