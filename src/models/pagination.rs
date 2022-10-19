use serde::{Deserialize};

#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}