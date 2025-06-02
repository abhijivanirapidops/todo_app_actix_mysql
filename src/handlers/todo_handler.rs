use actix_web::{HttpRequest, HttpResponse, Responder, web};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    middleware::auth_middleware::get_current_user,
    models::todo_model::{CreateTodoRequest, Todo, UpdateTodoRequest},
    schema::todo_schema::{create_todo, delete_todo, get_all_todos, get_todo_by_id, update_todo},
};

fn parse_uuid(id: String) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(&id).map_err(|_| HttpResponse::BadRequest().json("Invalid UUID format"))
}

pub async fn create_todo_handler(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    todo_data: web::Json<CreateTodoRequest>,
) -> impl Responder {
    let auth_user = match get_current_user(&req) {
        Ok(user) => user,
        Err(response) => return response,
    };

    let new_todo = Todo::new(
        todo_data.title.clone(),
        todo_data.description.clone(),
        todo_data.status.clone(),
        auth_user.user_id.to_string(),
    );

    match create_todo(&pool, &new_todo).await {
        Ok(_) => HttpResponse::Created().json(&new_todo),
        Err(err) => {
            log::error!("Create todo error: {}", err);
            HttpResponse::InternalServerError().json("Error creating todo")
        }
    }
}

pub async fn get_todos_handler(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    let auth_user = match get_current_user(&req) {
        Ok(user) => user,
        Err(response) => return response,
    };

    match get_all_todos(&pool, &auth_user.user_id).await {
        Ok(todos) => HttpResponse::Ok().json(todos),
        Err(err) => {
            log::error!("Fetch todos error: {}", err);
            HttpResponse::InternalServerError().json("Error fetching todos")
        }
    }
}

pub async fn get_todo_handler(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = match parse_uuid(path.into_inner()) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match get_todo_by_id(&pool, &id).await {
        Ok(Some(todo)) => HttpResponse::Ok().json(todo),
        Ok(None) => HttpResponse::NotFound().json("Todo not found"),
        Err(err) => {
            log::error!("Get todo error: {}", err);
            HttpResponse::InternalServerError().json("Error fetching todo")
        }
    }
}

pub async fn update_todo_handler(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    update_data: web::Json<UpdateTodoRequest>,
) -> impl Responder {
    let id = match parse_uuid(path.into_inner()) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match update_todo(&pool, &id, &update_data).await {
        Ok(Some(updated)) => HttpResponse::Ok().json(updated),
        Ok(None) => HttpResponse::NotFound().json("Todo not found"),
        Err(err) => {
            log::error!("Update todo error: {}", err);
            HttpResponse::InternalServerError().json("Error updating todo")
        }
    }
}

pub async fn delete_todo_handler(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = match parse_uuid(path.into_inner()) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match delete_todo(&pool, &id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json("Todo not found"),
        Err(err) => {
            log::error!("Delete todo error: {}", err);
            HttpResponse::InternalServerError().json("Error deleting todo")
        }
    }
}
