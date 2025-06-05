use actix_web::web;

use crate::{
    handlers::user_handler::{
        delete_me_handler, delete_user_handler, get_me_handler, get_user_handler,
        update_me_handler, update_user_handler,
    },
    middleware::auth_middleware::AuthMiddleware,
    utils::get_env_vars::get_env_var,
};

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .wrap(AuthMiddleware::new(get_env_var("JWT_SECRET")))
            .route("/me", web::get().to(get_me_handler))
            .route("/me", web::put().to(update_me_handler))
            .route("/me", web::delete().to(delete_me_handler))
            .route("/{id}", web::get().to(get_user_handler))
            .route("/{id}", web::put().to(update_user_handler))
            .route("/{id}", web::delete().to(delete_user_handler)), // .route("/email/{email}", web::get().to(getuserby)),
    );
}
