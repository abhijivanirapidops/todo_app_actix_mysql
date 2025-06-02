use actix_web::web;

use crate::handlers::user_handler::{
    create_user_handler, delete_user_handler, get_user_handler, get_users_handler,
    update_user_handler,
};

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::post().to(create_user_handler))
            .route("", web::get().to(get_users_handler))
            .route("/{id}", web::get().to(get_user_handler))
            .route("/{id}", web::put().to(update_user_handler))
            .route("/{id}", web::delete().to(delete_user_handler)), // .route("/email/{email}", web::get().to(get_user_by_email)),
    );
}
