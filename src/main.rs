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
        // FromRequest,
        // RequestParts,
    },
    // async_trait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{
    sync::{Arc},
    net::SocketAddr,
};
use tokio_postgres::{connect, Client, NoTls};
// use futures::{
//     stream::futures_unordered::FuturesUnordered
// };

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();

    let db_connect = connect_pg().await;

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


async fn create_todo(Extension(client): Extension<Arc<Client>>, Json(payload): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: payload.text,
        completed: false,
    };
    client.execute("INSERT INTO todos (id, text, completed) VALUES ($1, $2, $3)", &[&todo.id, &todo.text, &todo.completed]).await.expect("插入失败");
    (StatusCode::CREATED, Json(todo))
}

async fn delete_todo(Path(id): Path<Uuid>, Extension(client): Extension<Arc<Client>>) -> impl IntoResponse {
    match client.execute("DELETE FROM todos WHERE id = $1", &[&id]).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND
    }
}

async fn update_todo(Path(id): Path<Uuid>, Json(payload): Json<UpdateTodo>, Extension(client): Extension<Arc<Client>>) -> impl IntoResponse {
    let row = client.query_one("SELECT * FROM todos WHERE id = $1", &[&id]).await.unwrap();
    let mut tmp_text = String::new();
    if let Some(text) = payload.text {
        tmp_text = text;
    } else {
        tmp_text = row.get("text");
    }
    let mut tmp_completed = false;
    if let Some(completed) = payload.completed {
        tmp_completed = completed;
    } else {
        tmp_completed = row.get("completed");
    }
    client.query_one("UPDATE todos SET text = $1, completed = $2", &[&tmp_text, &tmp_completed]).await.unwrap();
    StatusCode::NO_CONTENT
}

async fn get_todo(Path(id): Path<Uuid>, Extension(client): Extension<Arc<Client>>) -> impl IntoResponse {
    let row = client.query_one("SELECT * FROM todos WHERE id = $1", &[&id]).await.unwrap();
    let todo = Todo {
        id: row.get("id"),
        text: row.get("text"),
        completed: row.get("completed"),
    };
    (StatusCode::OK, Json(todo))
}

async fn list_todo(pagination: Option<Query<Pagination>>, Extension(client): Extension<Arc<Client>>) -> impl IntoResponse {
    let Query(pagination) = pagination.unwrap_or_default();
    let offset = (pagination.page.unwrap_or(0) * pagination.per_page.unwrap_or(10)) as u32;
    let per_page = pagination.per_page.unwrap_or(10) as u32;
    let stream = client.query("SELECT * FROM todos OFFSET $1 LIMIT $2", &[&offset, &per_page]).await.unwrap();
    let mut todos = Vec::new();
    for row in &stream {
        let mut todo = Todo {
            id: row.get("id"),
            text: row.get("text"),
            completed: row.get("completed"),
        };
        todos.push(todo);
    }
    (StatusCode::OK, Json(todos))
}

async fn connect_pg() -> Arc<Client> {
    let pg_url = "";
    let (client, conn) = connect(pg_url, NoTls).await.expect("无法连接数据库");
    tokio::spawn(async move {
        if let Err(error) = conn.await {
            eprintln!("数据库连接失败：{}", error);
        }
    });
    Arc::new(client)
}

// pub struct DatabaseConnection(pub Arc<Client>);

// #[async_trait]
// impl<B> FromRequest<B> for DatabaseConnection
// where
//     B: Send,
// {
//     type Rejection = (StatusCode, String);

//     async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
//         let Extension(pg_connection) = Extension::<Arc<Client>>::from_request(req)
//             .await
//             .map_err(innernal_error)?;

//         Ok(Self(pg_connection))
//     }
// }

// fn innernal_error<E>(err: E) -> (StatusCode, String)
// where
//     E: std::error::Error,
// {
//     (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
// }

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