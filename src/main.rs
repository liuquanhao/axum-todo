use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
    extract::{Path, Json, Extension, Query},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let db = Db::default();

    let app = Router::new()
        .route("/todos/", post(create_todo).get(list_todo))
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .layer(Extension(db));
    let addr: &SocketAddr = &"0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn create_todo(Extension(db): Extension<Db>, Json(payload): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: payload.text,
        completed: false,
    };
    db.write().unwrap().insert(todo.id, todo.clone());
    (StatusCode::CREATED, Json(todo))
}

async fn delete_todo(Path(id): Path<Uuid>, Extension(db): Extension<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn update_todo(Path(id): Path<Uuid>, Json(payload): Json<UpdateTodo>, Extension(db): Extension<Db>) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = db.read().unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Some(text) = payload.text {
        todo.text = text;
    }
    if let Some(completed) = payload.completed {
        todo.completed = completed;
    }
    db.write().unwrap().insert(todo.id, todo.clone());
    Ok(Json(todo))
}

async fn get_todo(Path(id): Path<Uuid>, Extension(db): Extension<Db>) -> Result<impl IntoResponse, StatusCode> {
    match db.read().unwrap().get(&id).cloned() {
        Some(todo) => Ok(Json(todo)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn list_todo(pagination: Option<Query<Pagination>>, Extension(db): Extension<Db>) -> impl IntoResponse {
    let Query(pagination) = pagination.unwrap_or_default();
    let todos = db.read().unwrap().values()
        .skip(pagination.offset.unwrap_or(0))
        .take(pagination.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();
    Json(todos)
}

type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

#[derive(Debug, Deserialize, Default)]
struct Pagination {
    offset: Option<usize>,
    limit: Option<usize>,
}