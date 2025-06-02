use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "status", rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProcess,
    Completed,
}

impl Default for TodoStatus {
    fn default() -> Self {
        TodoStatus::Pending
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TodoStatus,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
}

impl Todo {
    pub fn new(
        title: String,
        description: Option<String>,
        status: Option<TodoStatus>,
        user_id: String,
    ) -> Self {
        let now = Utc::now();
        let status = status.unwrap_or_default();

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            status,
            user_id,
            created_at: now,
            updated_at: now,
        }
    }
}
