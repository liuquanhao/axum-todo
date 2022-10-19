use super::todo_error::*;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    TodoRepo(TodoRepoError),
}

impl From<TodoRepoError> for AppError {
    fn from(inner: TodoRepoError) -> Self {
        AppError::TodoRepo(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            AppError::TodoRepo(TodoRepoError::NotFound) => {
                (StatusCode::NOT_FOUND, "没有找到Todo")
            },
            AppError::TodoRepo(TodoRepoError::DatabaseError) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误")
            },
        };
        let body = Json(json!({
            "error": error_msg,
        }));
        (status, body).into_response()
    }
}