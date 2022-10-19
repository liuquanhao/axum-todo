//! Run example
//! 
//! ```bash
//! $ psql
//! postgres=# CREATE USER todouser WITH ENCRYPTED PASSWORD 'todopassword';
//! postgres=# CREATE DATABASE todos;
//! postgres=# CREATE TABLE todos (id UUID PRIMARY KEY NOT NULL, text VARCHAR(255) NOT NULL DEFAULT '', completed BOOLEAN NOT NULL DEFAULT false);
//! $ POSTGRESQL_URL="postgres://todouser:todopassword@127.0.0.1:5432/todos" cargo run
//! ```

mod models;
mod errors;
mod handlers;

use models::todo_repo::{
    TodoRepo,
    DynTodoRepo,
    TodoRepoTrait,
};
use handlers::{
    connect_pg,
    create_todo,
    list_todo,
    get_todo,
    update_todo,
    delete_todo,
};

use axum::{
    routing::{get, post},
    Router,
    extract::{
        Extension,
    },
};
use std::{
    sync::{Arc},
    net::SocketAddr,
    env,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pg_url = match env::var("POSTGRESQL_URL") {
        Ok(url) => url,
        Err(_) => panic!("Need env: POSTGRESQL_URL.")
    };
    let db_client = connect_pg(&pg_url).await;
    let todo_repo = Arc::new(TodoRepo::new(db_client).await) as DynTodoRepo;
    let app = Router::new()
        .route("/todos/", post(create_todo).get(list_todo))
        .route("/todos/:id", get(get_todo).put(update_todo).delete(delete_todo))
        .layer(Extension(todo_repo));
    let addr: &SocketAddr = &"0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
