use actix_web::web::{self, ServiceConfig};

use crate::routes::auth_routes::configure_auth_routes;
use crate::routes::todo_routes::configure_todo_routes;
use crate::routes::user_routes::configure_user_routes;

pub mod database;

pub fn config_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(configure_todo_routes)
            .configure(configure_user_routes)
            .configure(configure_auth_routes),
    );
}
