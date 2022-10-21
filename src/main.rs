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
mod server;

use handlers::{create_todo, delete_todo, get_todo, hello_world, list_todo, update_todo};
use models::todo_repo::TodoRepo;
use pg_database::PgDatabase;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get())
        .build()
        .unwrap();
    rt.block_on(serve());
}

async fn serve() {
    let pg_database = PgDatabase::new(&PgDatabase::get_pg_url("POSTGRESQL_URL")).await;
    let todo_repo = TodoRepo::new(pg_database).await.to_dyn();

    let router = Router::new()
        .route("/helloworld/", get(hello_world))
        .route("/todos/", post(create_todo).get(list_todo))
        .route(
            "/todos/:id",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .layer(Extension(todo_repo));

    server::builder()
        .serve(router.into_make_service())
        .await
        .unwrap();
}
