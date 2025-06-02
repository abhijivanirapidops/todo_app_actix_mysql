use actix_web::web;

use crate::handlers::auth_handler::{login_handler, register_handler};

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login_handler))
            .route("/register", web::post().to(register_handler)),
    );
}
