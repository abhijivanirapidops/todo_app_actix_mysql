use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::user_model::{UpdateUserRequest, User, UserRole};

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    name: String,
    email: String,
    password: String,
    role: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        let role = match row.role.as_str() {
            "user" => UserRole::User,
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        };

        let now = Utc::now();
        let created_at = row.created_at.unwrap_or(now);
        let updated_at = row.updated_at.unwrap_or(now);

        User {
            id: row.id,
            name: row.name,
            email: row.email,
            password: row.password,
            role,
            created_at,
            updated_at,
        }
    }
}

pub async fn create_user(pool: &MySqlPool, user: &User) -> Result<()> {
    let role_str = match user.role {
        UserRole::User => "user",
        UserRole::Admin => "admin",
    };

    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password, role, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user.id,
        user.name,
        user.email,
        user.password,
        role_str,
        user.created_at,
        user.updated_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_users(pool: &MySqlPool) -> Result<Vec<User>> {
    let rows = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, name, email, password, role, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let users = rows.into_iter().map(User::from).collect();
    Ok(users)
}

pub async fn get_user_by_id(pool: &MySqlPool, id: &Uuid) -> Result<Option<User>> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, name, email, password, role, created_at, updated_at
        FROM users
        WHERE id = ?
        "#,
        id.to_string()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(User::from))
}

pub async fn get_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, name, email, password, role, created_at, updated_at
        FROM users
        WHERE email = ?
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(User::from))
}

pub async fn update_user(
    pool: &MySqlPool,
    id: &Uuid,
    update_data: &UpdateUserRequest,
) -> Result<Option<User>> {
    let now = Utc::now();

    let current_user = get_user_by_id(pool, id).await?;

    if let Some(user) = current_user {
        let name = update_data.name.as_ref().unwrap_or(&user.name);
        let email = update_data.email.as_ref().unwrap_or(&user.email);
        let password = update_data.password.as_ref().unwrap_or(&user.password);
        let role = update_data.role.as_ref().unwrap_or(&user.role);

        let role_str = match role {
            UserRole::User => "user",
            UserRole::Admin => "admin",
        };

        sqlx::query!(
            r#"
            UPDATE users
            SET name = ?, email = ?, password = ?, role = ?, updated_at = ?
            WHERE id = ?
            "#,
            name,
            email,
            password,
            role_str,
            now,
            id.to_string()
        )
        .execute(pool)
        .await?;

        return get_user_by_id(pool, id).await;
    }

    Ok(None)
}

pub async fn delete_user(pool: &MySqlPool, id: &Uuid) -> Result<bool> {
    let result = sqlx::query!("DELETE FROM users WHERE id = ?", id.to_string())
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn check_email_exists(pool: &MySqlPool, email: &str) -> Result<bool> {
    let result = sqlx::query!("SELECT id FROM users WHERE email = ?", email)
        .fetch_optional(pool)
        .await?;

    Ok(result.is_some())
}
