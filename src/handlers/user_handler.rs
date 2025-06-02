use actix_web::{HttpResponse, Responder, web};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    models::user_model::{CreateUserRequest, UpdateUserRequest, User},
    schema::user_schema::{
        check_email_exists, create_user, delete_user, get_all_users, get_user_by_id, update_user,
    },
};

fn parse_uuid(id: String) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(&id).map_err(|_| HttpResponse::BadRequest().json("Invalid UUID format"))
}

pub async fn create_user_handler(
    pool: web::Data<MySqlPool>,
    user_data: web::Json<CreateUserRequest>,
) -> impl Responder {
    // Check if email already exists
    match check_email_exists(&pool, &user_data.email).await {
        Ok(true) => return HttpResponse::Conflict().json("Email already exists"),
        Ok(false) => {}
        Err(err) => {
            log::error!("Email check error: {}", err);
            return HttpResponse::InternalServerError().json("Error checking email");
        }
    }

    let new_user = User::new(
        user_data.name.clone(),
        user_data.email.clone(),
        user_data.password.clone(), // Note: In production, hash the password here
        user_data.role.clone(),
    );

    match create_user(&pool, &new_user).await {
        Ok(_) => HttpResponse::Created().json(new_user.to_response()),
        Err(err) => {
            log::error!("Create user error: {}", err);
            HttpResponse::InternalServerError().json("Error creating user")
        }
    }
}

pub async fn get_users_handler(pool: web::Data<MySqlPool>) -> impl Responder {
    match get_all_users(&pool).await {
        Ok(users) => {
            let user_responses: Vec<_> = users.into_iter().map(|u| u.to_response()).collect();
            HttpResponse::Ok().json(user_responses)
        }
        Err(err) => {
            log::error!("Fetch users error: {}", err);
            HttpResponse::InternalServerError().json("Error fetching users")
        }
    }
}

pub async fn get_user_handler(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = match parse_uuid(path.into_inner()) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match get_user_by_id(&pool, &id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user.to_response()),
        Ok(None) => HttpResponse::NotFound().json("User not found"),
        Err(err) => {
            log::error!("Get user error: {}", err);
            HttpResponse::InternalServerError().json("Error fetching user")
        }
    }
}

pub async fn update_user_handler(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    update_data: web::Json<UpdateUserRequest>,
) -> impl Responder {
    let id = match parse_uuid(path.into_inner()) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    // If email is being updated, check if it already exists
    if let Some(ref email) = update_data.email {
        match check_email_exists(&pool, email).await {
            Ok(true) => {
                // Check if the email belongs to the current user
                match get_user_by_id(&pool, &id).await {
                    Ok(Some(current_user)) => {
                        if current_user.email != *email {
                            return HttpResponse::Conflict().json("Email already exists");
                        }
                    }
                    Ok(None) => return HttpResponse::NotFound().json("User not found"),
                    Err(err) => {
                        log::error!("Get user error: {}", err);
                        return HttpResponse::InternalServerError().json("Error checking user");
                    }
                }
            }
            Ok(false) => {}
            Err(err) => {
                log::error!("Email check error: {}", err);
                return HttpResponse::InternalServerError().json("Error checking email");
            }
        }
    }

    match update_user(&pool, &id, &update_data).await {
        Ok(Some(updated)) => HttpResponse::Ok().json(updated.to_response()),
        Ok(None) => HttpResponse::NotFound().json("User not found"),
        Err(err) => {
            log::error!("Update user error: {}", err);
            HttpResponse::InternalServerError().json("Error updating user")
        }
    }
}

pub async fn delete_user_handler(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> impl Responder {
    let id = match parse_uuid(path.into_inner()) {
        Ok(uuid) => uuid,
        Err(resp) => return resp,
    };

    match delete_user(&pool, &id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json("User not found"),
        Err(err) => {
            log::error!("Delete user error: {}", err);
            HttpResponse::InternalServerError().json("Error deleting user")
        }
    }
}
