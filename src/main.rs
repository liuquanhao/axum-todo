//! Run example
//!
//! ```bash
//! $ psql
//! postgres=# CREATE USER todouser WITH ENCRYPTED PASSWORD 'todopassword';
//! postgres=# CREATE DATABASE todos;
//! postgres=# GRANT ALL PRIVILEGES ON DATABASE todos to todouser;
//! postgres=# \c todos
//! postgres=# CREATE TABLE todos (id UUID PRIMARY KEY NOT NULL, text VARCHAR(255) NOT NULL DEFAULT '', completed BOOLEAN NOT NULL DEFAULT false);
//! $ POSTGRESQL_URL="postgres://todouser:todopassword@127.0.0.1:5432/todos" cargo run
//! ```

mod errors;
mod handlers;
mod models;
mod pg_database;

use handlers::{create_todo, delete_todo, get_todo, list_todo, update_todo};
use models::todo_repo::{DynTodoRepo, TodoRepo};
use pg_database::PgDatabase;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pg_database = PgDatabase::new(&PgDatabase::get_pg_url("POSTGRESQL_URL")).await;
    let app = app(TodoRepo::new(pg_database).await.to_dyn());
    let addr: &SocketAddr = &"0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn app(todo_repo: DynTodoRepo) -> Router {
    Router::new()
        .route("/todos/", post(create_todo).get(list_todo))
        .route(
            "/todos/:id",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .layer(Extension(todo_repo))
}
