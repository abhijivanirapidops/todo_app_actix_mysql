use actix_web::{HttpResponse, Responder, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::MySqlPool;

use crate::{
    middleware::auth_middleware::Claims,
    models::{
        auth_model::{AuthResponse, LoginRequest, RegisterRequest, UserInfo},
        user_model::User,
    },
    schema::user_schema::{check_email_exists, create_user, get_user_by_email},
};

pub async fn register_handler(
    pool: web::Data<MySqlPool>,
    register_data: web::Json<RegisterRequest>,
    secret_key: web::Data<String>,
) -> impl Responder {
    // Check if email already exists
    match check_email_exists(&pool, &register_data.email).await {
        Ok(true) => return HttpResponse::Conflict().json("Email already exists"),
        Ok(false) => {}
        Err(err) => {
            log::error!("Email check error: {}", err);
            return HttpResponse::InternalServerError().json("Error checking email");
        }
    }

    // Hash password
    let hashed_password = match hash(&register_data.password, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(err) => {
            log::error!("Password hashing error: {}", err);
            return HttpResponse::InternalServerError().json("Error processing password");
        }
    };

    // Create new user
    let new_user = User::new(
        register_data.name.clone(),
        register_data.email.clone(),
        hashed_password,
        None, // Default role (user)
    );

    match create_user(&pool, &new_user).await {
        Ok(_) => {
            // Generate JWT token
            let token = match generate_token(&new_user, &secret_key) {
                Ok(token) => token,
                Err(err) => {
                    log::error!("Token generation error: {}", err);
                    return HttpResponse::InternalServerError().json("Error generating token");
                }
            };

            let response = AuthResponse {
                token,
                user: UserInfo {
                    id: new_user.id,
                    name: new_user.name,
                    email: new_user.email,
                    role: format!("{:?}", new_user.role).to_lowercase(),
                },
            };

            HttpResponse::Created().json(response)
        }
        Err(err) => {
            log::error!("Create user error: {}", err);
            HttpResponse::InternalServerError().json("Error creating user")
        }
    }
}

pub async fn login_handler(
    pool: web::Data<MySqlPool>,
    login_data: web::Json<LoginRequest>,
    secret_key: web::Data<String>,
) -> impl Responder {
    // Get user by email
    let user = match get_user_by_email(&pool, &login_data.email).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().json("Invalid credentials"),
        Err(err) => {
            log::error!("Get user error: {}", err);
            return HttpResponse::InternalServerError().json("Error fetching user");
        }
    };

    // Verify password
    match verify(&login_data.password, &user.password) {
        Ok(true) => {
            // Generate JWT token
            let token = match generate_token(&user, &secret_key) {
                Ok(token) => token,
                Err(err) => {
                    log::error!("Token generation error: {}", err);
                    return HttpResponse::InternalServerError().json("Error generating token");
                }
            };

            let response = AuthResponse {
                token,
                user: UserInfo {
                    id: user.id,
                    name: user.name,
                    email: user.email,
                    role: format!("{:?}", user.role).to_lowercase(),
                },
            };

            HttpResponse::Ok().json(response)
        }
        Ok(false) => HttpResponse::Unauthorized().json("Invalid credentials"),
        Err(err) => {
            log::error!("Password verification error: {}", err);
            HttpResponse::InternalServerError().json("Error verifying password")
        }
    }
}

fn generate_token(user: &User, secret_key: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.clone(),
        email: user.email.clone(),
        role: format!("{:?}", user.role).to_lowercase(),
        exp: expiration,
    };

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret_key.as_ref());

    encode(&header, &claims, &encoding_key)
}
