use crate::errors::app_error::AppError;
use crate::models::pagination::Pagination;
use crate::models::todo::{CreateTodo, Todo, UpdateTodo};
use crate::models::todo_repo::DynTodoRepo;

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tokio_postgres::{connect, Client, NoTls};
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
    Ok(StatusCode::OK)
}

pub async fn update_todo(
    Path(id): Path<Uuid>,
    Json(update_todo): Json<UpdateTodo>,
    Extension(todo_repo): Extension<DynTodoRepo>,
) -> Result<StatusCode, AppError> {
    let _ = todo_repo.update_todo(id, update_todo).await?;
    Ok(StatusCode::OK)
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

pub async fn connect_pg(pg_url: &str) -> Arc<Client> {
    let (client, conn) = connect(pg_url, NoTls).await.expect("无法连接数据库");
    tokio::spawn(async move {
        if let Err(error) = conn.await {
            eprintln!("数据库连接失败：{}", error);
        }
    });
    Arc::new(client)
}
