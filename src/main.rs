mod config;
mod handlers;
mod middleware;
mod models;
mod routes;
mod schema;
mod utils;

use crate::routes::config_routes;
use actix_web::{App, HttpServer, middleware::Logger, web};
use config::database::create_connection_pool;
use dotenv::dotenv;
use env_logger;
use utils::get_env_vars::get_env_var;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = get_env_var("DATABASE_URL");
    let host = get_env_var("SERVER_HOST");
    let port: u16 = get_env_var("SERVER_PORT").parse().unwrap();
    let jwt_secret = get_env_var("JWT_SECRET");

    let pool = create_connection_pool(&database_url)
        .await
        .expect("Failed to create database connection pool");

    println!("🚀 Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .wrap(Logger::default())
            .configure(config_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
