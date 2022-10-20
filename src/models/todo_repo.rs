use super::pagination::Pagination;
use super::todo::{CreateTodo, Todo, UpdateTodo};
use crate::errors::todo_error::TodoRepoError;
use crate::pg_database::PgDatabase;

use axum::async_trait;
use std::sync::Arc;
use tokio_postgres::{Client, Statement};
use uuid::Uuid;

pub type DynTodoRepo = Arc<dyn TodoRepoTrait + Send + Sync>;

pub struct TodoRepo {
    pub client: Arc<Client>,
    pub create_todo: Statement,
    pub delete_todo: Statement,
    pub update_todo: Statement,
    pub get_todo: Statement,
    pub list_todo: Statement,
}

#[async_trait]
pub trait TodoRepoTrait {
    async fn create_todo(&self, create_todo: CreateTodo) -> Result<Todo, TodoRepoError>;
    async fn delete_todo(&self, id: Uuid) -> Result<u64, TodoRepoError>;
    async fn update_todo(&self, id: Uuid, update_todo: UpdateTodo) -> Result<u64, TodoRepoError>;
    async fn get_todo(&self, id: Uuid) -> Result<Todo, TodoRepoError>;
    async fn list_todo(&self, pagination: Pagination) -> Result<Vec<Todo>, TodoRepoError>;
}

#[async_trait]
impl TodoRepoTrait for TodoRepo {
    async fn create_todo(&self, create_todo: CreateTodo) -> Result<Todo, TodoRepoError> {
        let todo = Todo {
            id: Uuid::new_v4(),
            text: create_todo.text,
            completed: false,
        };
        match self
            .client
            .execute(&self.create_todo, &[&todo.id, &todo.text, &todo.completed])
            .await
        {
            Ok(_) => Ok(todo),
            Err(_) => Err(TodoRepoError::DatabaseError),
        }
    }

    async fn delete_todo(&self, id: Uuid) -> Result<u64, TodoRepoError> {
        match self.client.execute(&self.delete_todo, &[&id]).await {
            Ok(num) => Ok(num),
            Err(_) => Err(TodoRepoError::DatabaseError),
        }
    }

    async fn update_todo(&self, id: Uuid, update_todo: UpdateTodo) -> Result<u64, TodoRepoError> {
        let row = match self.client.query_one(&self.get_todo, &[&id]).await {
            Ok(row) => row,
            Err(_) => return Err(TodoRepoError::NotFound),
        };
        let tmp_text: String;
        if let Some(text) = update_todo.text {
            tmp_text = text;
        } else {
            tmp_text = row.get("text");
        }
        let tmp_completed: bool;
        if let Some(completed) = update_todo.completed {
            tmp_completed = completed;
        } else {
            tmp_completed = row.get("completed");
        }
        match self
            .client
            .execute(&self.update_todo, &[&tmp_text, &tmp_completed, &id])
            .await
        {
            Ok(num) => Ok(num),
            Err(_) => Err(TodoRepoError::DatabaseError),
        }
    }

    async fn get_todo(&self, id: Uuid) -> Result<Todo, TodoRepoError> {
        let row = match self.client.query_one(&self.get_todo, &[&id]).await {
            Ok(row) => row,
            Err(_) => return Err(TodoRepoError::NotFound),
        };
        let todo = Todo {
            id: row.get("id"),
            text: row.get("text"),
            completed: row.get("completed"),
        };
        Ok(todo)
    }

    async fn list_todo(&self, pagination: Pagination) -> Result<Vec<Todo>, TodoRepoError> {
        let offset =
            ((pagination.page.unwrap_or(1) - 1) * pagination.per_page.unwrap_or(10)) as i64;
        let per_page = pagination.per_page.unwrap_or(10) as i64;
        let stream = self
            .client
            .query(&self.list_todo, &[&offset, &per_page])
            .await;
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
        Ok(todos)
    }
}

impl TodoRepo {
    pub async fn new(pg_database: PgDatabase) -> Self {
        TodoRepo {
            client: Arc::clone(&pg_database.client),
            create_todo: pg_database
                .client
                .prepare("INSERT INTO todos (id, text, completed) VALUES ($1, $2, $3)")
                .await
                .unwrap(),
            delete_todo: pg_database
                .client
                .prepare("DELETE FROM todos WHERE id = $1")
                .await
                .unwrap(),
            update_todo: pg_database
                .client
                .prepare("UPDATE todos SET text = $1, completed = $2 WHERE id = $3")
                .await
                .unwrap(),
            get_todo: pg_database
                .client
                .prepare("SELECT * FROM todos WHERE id = $1")
                .await
                .unwrap(),
            list_todo: pg_database
                .client
                .prepare("SELECT * FROM todos OFFSET $1 LIMIT $2")
                .await
                .unwrap(),
        }
    }

    pub fn to_dyn(self) -> DynTodoRepo {
        Arc::new(self) as DynTodoRepo
    }
}
