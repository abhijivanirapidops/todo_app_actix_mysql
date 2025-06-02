use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::todo_model::{Todo, TodoStatus, UpdateTodoRequest};

#[derive(sqlx::FromRow)]
struct TodoRow {
    id: String,
    title: String,
    description: Option<String>,
    status: String,
    user_id: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<TodoRow> for Todo {
    fn from(row: TodoRow) -> Self {
        let status = match row.status.as_str() {
            "pending" => TodoStatus::Pending,
            "in_process" => TodoStatus::InProcess,
            "completed" => TodoStatus::Completed,
            _ => TodoStatus::Pending,
        };

        let now = Utc::now();
        let created_at = row.created_at.unwrap_or(now);
        let updated_at = row.updated_at.unwrap_or(now);

        Todo {
            id: row.id,
            title: row.title,
            description: row.description,
            status,
            user_id: row.user_id,
            created_at,
            updated_at,
        }
    }
}

pub async fn create_todo(pool: &MySqlPool, todo: &Todo) -> Result<()> {
    let status_str = match todo.status {
        TodoStatus::Pending => "pending",
        TodoStatus::InProcess => "in_process",
        TodoStatus::Completed => "completed",
    };

    sqlx::query!(
        r#"
        INSERT INTO todos (id, title, description, status, user_id, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        todo.id,
        todo.title,
        todo.description,
        status_str,
        todo.user_id,
        todo.created_at,
        todo.updated_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_todos(pool: &MySqlPool, user_id: &Uuid) -> Result<Vec<Todo>> {
    let rows = sqlx::query_as!(
        TodoRow,
        r#"
        SELECT id, title, description, status, user_id, created_at, updated_at
        FROM todos
        WHERE user_id = ?
        ORDER BY created_at DESC
        "#,
        user_id.to_string()
    )
    .fetch_all(pool)
    .await?;

    let todos = rows.into_iter().map(Todo::from).collect();
    Ok(todos)
}

pub async fn get_todo_by_id(pool: &MySqlPool, id: &Uuid) -> Result<Option<Todo>> {
    let row = sqlx::query_as!(
        TodoRow,
        r#"
        SELECT id, title, description, status, user_id, created_at, updated_at 
        FROM todos 
        WHERE id = ?
        "#,
        id.to_string()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Todo::from))
}

pub async fn update_todo(
    pool: &MySqlPool,
    id: &Uuid,
    update_data: &UpdateTodoRequest,
) -> Result<Option<Todo>> {
    let now = Utc::now();

    let current_todo = get_todo_by_id(pool, id).await?;

    if let Some(todo) = current_todo {
        let title = update_data.title.as_ref().unwrap_or(&todo.title);
        let description = update_data
            .description
            .as_ref()
            .or(todo.description.as_ref());
        let status = update_data.status.as_ref().unwrap_or(&todo.status);

        let status_str = match status {
            TodoStatus::Pending => "pending",
            TodoStatus::InProcess => "in_process",
            TodoStatus::Completed => "completed",
        };

        sqlx::query!(
            r#"
            UPDATE todos
            SET title = ?, description = ?, status = ?, updated_at = ?
            WHERE id = ?
            "#,
            title,
            description,
            status_str,
            now,
            id.to_string()
        )
        .execute(pool)
        .await?;

        return get_todo_by_id(pool, id).await;
    }

    Ok(None)
}

pub async fn delete_todo(pool: &MySqlPool, id: &Uuid) -> Result<bool> {
    let result = sqlx::query!("DELETE FROM todos WHERE id = ?", id.to_string())
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
