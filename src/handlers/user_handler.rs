use actix_web::{HttpRequest, HttpResponse, Responder, web};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    middleware::auth_middleware::get_current_user,
    models::user_model::UpdateUserRequest,
    schema::user_schema::{
        check_email_exists, delete_user, get_all_users, get_user_by_id, update_user,
    },
};

fn parse_uuid(id: String) -> Result<Uuid, HttpResponse> {
    Uuid::parse_str(&id).map_err(|_| HttpResponse::BadRequest().json("Invalid UUID format"))
}

// Me (GET, PUT, DELETE):
pub async fn get_me_handler(req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    let auth_user = match get_current_user(&req) {
        Ok(user) => user,
        Err(response) => return response,
    };

    match get_user_by_id(&pool, &auth_user.user_id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user.to_response()),
        Ok(None) => HttpResponse::NotFound().json("User not found"),
        Err(err) => {
            log::error!("Get user error: {}", err);
            HttpResponse::InternalServerError().json("Error fetching user")
        }
    }
}

pub async fn update_me_handler(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    update_data: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let auth_user = match get_current_user(&req) {
        Ok(user) => user,
        Err(response) => return response,
    };
    let user_id_str = auth_user.user_id.to_string();
    let user_id_path = web::Path::from(user_id_str);

    update_user_handler(pool, user_id_path, update_data).await
}

pub async fn delete_me_handler(req: HttpRequest, pool: web::Data<MySqlPool>) -> HttpResponse {
    let auth_user = match get_current_user(&req) {
        Ok(user) => user,
        Err(response) => return response,
    };

    let user_id_str = auth_user.user_id.to_string();
    let user_id_path = web::Path::from(user_id_str);

    delete_user_handler(pool, user_id_path).await
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
) -> HttpResponse {
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
) -> HttpResponse {
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
