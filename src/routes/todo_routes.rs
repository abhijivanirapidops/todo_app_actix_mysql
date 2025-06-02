use crate::{
    handlers::todo_handler::{
        create_todo_handler, delete_todo_handler, get_todo_handler, get_todos_handler,
        update_todo_handler,
    },
    middleware::auth_middleware::AuthMiddleware,
    utils::get_env_vars::get_env_var,
};
use actix_web::web;

pub fn configure_todo_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/todos").service(
            web::scope("")
                .wrap(AuthMiddleware::new(get_env_var("JWT_SECRET")))
                .route("", web::get().to(get_todos_handler))
                .route("", web::post().to(create_todo_handler))
                .route("/{id}", web::get().to(get_todo_handler))
                .route("/{id}", web::put().to(update_todo_handler))
                .route("/{id}", web::delete().to(delete_todo_handler)),
        ),
    );
}
