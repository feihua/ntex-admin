use std::env;

use ntex::web;
use sea_orm::{Database, DatabaseConnection};
use crate::handler::system::{sys_menu_handler, sys_role_handler, sys_user_handler};

pub mod common;
pub mod handler;
pub mod middleware;
pub mod model;
pub mod utils;
pub mod vo;

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("src/config/log4rs.yaml", Default::default()).unwrap();
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    let state = AppState { conn };

    web::HttpServer::new(move || {
        web::App::new()
            .state(state.clone())
            .wrap(web::middleware::Logger::default())
            .wrap(middleware::auth::JwtAuth)
            .service(
                (web::scope("/api"))
                    .service(sys_user_handler::add_sys_user)
                    .service(sys_user_handler::delete_sys_user)
                    .service(sys_user_handler::update_sys_user)
                    .service(sys_user_handler::update_sys_user_status)
                    .service(sys_user_handler::update_user_password)
                    .service(sys_user_handler::query_sys_user_detail)
                    .service(sys_user_handler::query_sys_user_list)
                    .service(sys_user_handler::query_user_role)
                    .service(sys_user_handler::update_user_role)
                    .service(sys_user_handler::query_user_menu)
                    .service(sys_user_handler::login)
                    .service(sys_role_handler::add_sys_role)
                    .service(sys_role_handler::delete_sys_role)
                    .service(sys_role_handler::update_sys_role)
                    .service(sys_role_handler::update_sys_role_status)
                    .service(sys_role_handler::query_sys_role_detail)
                    .service(sys_role_handler::query_sys_role_list)
                    .service(sys_role_handler::query_role_menu)
                    .service(sys_role_handler::update_role_menu)
                    .service(sys_menu_handler::add_sys_menu)
                    .service(sys_menu_handler::delete_sys_menu)
                    .service(sys_menu_handler::update_sys_menu)
                    .service(sys_menu_handler::update_sys_menu_status)
                    .service(sys_menu_handler::query_sys_menu_detail)
                    .service(sys_menu_handler::query_sys_menu_list),
            )
    })
    .bind(("127.0.0.1", 8101))?
    .run()
    .await
}
