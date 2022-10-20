use crate::errors::app_error::AppError;
use crate::models::pagination::Pagination;
use crate::models::todo::{CreateTodo, Todo, UpdateTodo};
use crate::models::todo_repo::DynTodoRepo;

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

pub async fn create_todo(
    Json(create_todo): Json<CreateTodo>,
    Extension(todo_repo): Extension<DynTodoRepo>,
) -> Result<Json<Todo>, AppError> {
    let todo = todo_repo.create_todo(create_todo).await?;
    Ok(todo.into())
}

pub async fn delete_todo(
    Path(id): Path<Uuid>,
    Extension(todo_repo): Extension<DynTodoRepo>,
) -> Result<StatusCode, AppError> {
    let _ = todo_repo.delete_todo(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_todo(
    Path(id): Path<Uuid>,
    Json(update_todo): Json<UpdateTodo>,
    Extension(todo_repo): Extension<DynTodoRepo>,
) -> Result<StatusCode, AppError> {
    let _ = todo_repo.update_todo(id, update_todo).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_todo(
    Path(id): Path<Uuid>,
    Extension(todo_repo): Extension<DynTodoRepo>,
) -> Result<Json<Todo>, AppError> {
    let todo = todo_repo.get_todo(id).await?;
    Ok(todo.into())
}

pub async fn list_todo(
    pagination: Option<Query<Pagination>>,
    Extension(todo_repo): Extension<DynTodoRepo>,
) -> Result<Json<Vec<Todo>>, AppError> {
    let Query(pagination) = pagination.unwrap_or_default();
    let todos = todo_repo.list_todo(pagination).await?;
    Ok(Json(todos))
}
