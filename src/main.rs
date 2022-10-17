//! Run example
//! 
//! ```bash
//! POSTGRESQL_URL="postgres://todouser:todopassword@127.0.0.1:5432/todos" cargo run
//! ```

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
    extract::{
        Path,
        Json,
        Extension,
        Query,
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{
    sync::{Arc},
    net::SocketAddr,
    env,
};
use tokio_postgres::{
    connect,
    Client,
    NoTls,
};

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let pg_url = match env::var("POSTGRESQL_URL") {
        Ok(url) => url,
        Err(_) => panic!("Need env: POSTGRESQL_URL.")
    };
    let db_connect = connect_pg(&pg_url).await;

    let app = Router::new()
        .route("/todos/", post(create_todo).get(list_todo))
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .layer(Extension(db_connect.clone()));
    let addr: &SocketAddr = &"0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn create_todo(Extension(client): Extension<Arc<Client>>, Json(payload): Json<CreateTodo>) -> Result<impl IntoResponse, StatusCode> {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: payload.text,
        completed: false,
    };
    match client.execute("INSERT INTO todos (id, text, completed) VALUES ($1, $2, $3)", &[&todo.id, &todo.text, &todo.completed]).await {
        Ok(_) => Ok((StatusCode::CREATED, Json(todo))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_todo(Path(id): Path<Uuid>, Extension(client): Extension<Arc<Client>>) -> impl IntoResponse {
    match client.execute("DELETE FROM todos WHERE id = $1", &[&id]).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn update_todo(Path(id): Path<Uuid>, Json(payload): Json<UpdateTodo>, Extension(client): Extension<Arc<Client>>) -> Result<impl IntoResponse, StatusCode> {
    let row = match client.query_one("SELECT * FROM todos WHERE id = $1", &[&id]).await {
        Ok(row) => row,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };
    let tmp_text:String;
    if let Some(text) = payload.text {
        tmp_text = text;
    } else {
        tmp_text = row.get("text");
    }
    let tmp_completed:bool;
    if let Some(completed) = payload.completed {
        tmp_completed = completed;
    } else {
        tmp_completed = row.get("completed");
    }
    match client.execute("UPDATE todos SET text = $1, completed = $2 WHERE id = $3", &[&tmp_text, &tmp_completed, &id]).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_todo(Path(id): Path<Uuid>, Extension(client): Extension<Arc<Client>>) -> Result<impl IntoResponse, StatusCode>  {
    let row = match client.query_one("SELECT * FROM todos WHERE id = $1", &[&id]).await {
        Ok(row) => row,
        Err(_) => return Err(StatusCode::NOT_FOUND)
    };
    let todo = Todo {
        id: row.get("id"),
        text: row.get("text"),
        completed: row.get("completed"),
    };
    Ok((StatusCode::OK, Json(todo)))
}

async fn list_todo(pagination: Option<Query<Pagination>>, Extension(client): Extension<Arc<Client>>) -> impl IntoResponse {
    let Query(pagination) = pagination.unwrap_or_default();
    let offset = ((pagination.page.unwrap_or(1) - 1) * pagination.per_page.unwrap_or(10)) as i64;
    let per_page = pagination.per_page.unwrap_or(10) as i64;
    let stream = client.query("SELECT * FROM todos OFFSET $1 LIMIT $2", &[&offset, &per_page]).await;
    let rows = match stream {
        Ok(rows) => rows,
        Err(_) => vec![],
    };
    let mut todos = Vec::new();
    for row in &rows {
        let todo = Todo {
            id: row.get("id"),
            text: row.get("text"),
            completed: row.get("completed"),
        };
        todos.push(todo);
    }
    (StatusCode::OK, Json(todos))
}

async fn connect_pg(pg_url: &str) -> Arc<Client> {
    // let pg_url = "postgres://todouser:todopassword@127.0.0.1:5432/todos";
    let (client, conn) = connect(pg_url, NoTls).await.expect("无法连接数据库");
    tokio::spawn(async move {
        if let Err(error) = conn.await {
            eprintln!("数据库连接失败：{}", error);
        }
    });
    Arc::new(client)
}

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
    page: Option<usize>,
    per_page: Option<usize>,
}