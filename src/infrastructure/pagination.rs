use crate::domain::error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Pagination {
    pub start: usize,
    pub end: usize,
}

pub struct InvalidPagination;

impl Pagination {
    fn new(start: usize, end: usize) -> Result<Self, InvalidPagination> {
        if start > end {
            Err(InvalidPagination)
        } else {
            Ok(Self { start, end })
        }
    }
}

pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, error::Error> {
    if params.contains_key("start") && params.contains_key("end") {
        let start = params
            .get("start")
            .unwrap()
            .parse::<usize>()
            .map_err(error::Error::ParseError)?;
        let end = params
            .get("end")
            .unwrap()
            .parse::<usize>()
            .map_err(error::Error::ParseError)?;
        let pagination =
            Pagination::new(start, end).map_err(|_| error::Error::InvalidParameters)?;
        Ok(pagination)
    } else {
        Err(error::Error::MissingParameters)
    }
}

pub fn limit_pagination<T>(pagination: Pagination, elements: &Vec<T>) -> Pagination {
    if pagination.end > elements.len() {
        Pagination {
            start: pagination.start,
            end: elements.len(),
        }
    } else {
        pagination
    }
}
