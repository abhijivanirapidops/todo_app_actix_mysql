use actix_web::web;

use crate::{
    handlers::user_handler::{
        delete_me_handler, delete_user_handler, get_me_handler, get_user_handler,
        get_users_handler, update_me_handler, update_user_handler,
    },
    middleware::{
        auth_middleware::AuthMiddleware, authorization_middleware::AuthorizationMiddleware,
    },
    utils::get_env_vars::get_env_var,
};

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    let jwt_secret = get_env_var("JWT_SECRET");

    // Public routes - no authentication
    cfg.service(web::scope("/users/public").route("/health", web::get().to(|| async { "OK" })));

    // Admin-only routes
    cfg.service(
        web::scope("/users/admin")
            .wrap(AuthorizationMiddleware::new(vec!["admin".to_string()]))
            .wrap(AuthMiddleware::new(jwt_secret.clone()))
            .route("", web::get().to(get_users_handler))
            .route("/{id}", web::get().to(get_user_handler))
            .route("/{id}", web::put().to(update_user_handler))
            .route("/{id}", web::delete().to(delete_user_handler)),
    );

    // Authenticated user-only routes (no role check)
    cfg.service(
        web::scope("/users")
            .wrap(AuthMiddleware::new(jwt_secret.clone()))
            .route("/me", web::get().to(get_me_handler))
            .route("/me", web::put().to(update_me_handler))
            .route("/me", web::delete().to(delete_me_handler)),
    );
}
