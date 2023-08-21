use std::env;

use diesel::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use ntex::web;
use ntex::web::middleware;
use once_cell::sync::Lazy;

use crate::handler::{menu_handler, role_handler, user_handler};

pub mod handler;
pub mod model;
pub mod vo;
pub mod utils;
pub mod schema;

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub static RB: Lazy<DbPool> = Lazy::new(|| {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
});

#[ntex::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("src/config/log4rs.yaml", Default::default()).unwrap();
    dotenv().ok();
    web::HttpServer::new(|| web::App::new()
        .wrap(middleware::Logger::default())
        .service(web::scope("/api")
            .service(user_handler::login)
            .service(user_handler::query_user_role)
            .service(user_handler::update_user_role)
            .service(user_handler::query_user_menu)
            .service(user_handler::user_list)
            .service(user_handler::user_save)
            .service(user_handler::user_delete)
            .service(user_handler::user_update)
            .service(user_handler::update_user_password)
            .service(role_handler::query_role_menu)
            .service(role_handler::update_role_menu)
            .service(role_handler::role_list)
            .service(role_handler::role_save)
            .service(role_handler::role_delete)
            .service(role_handler::role_update)
            .service(menu_handler::menu_list)
            .service(menu_handler::menu_save)
            .service(menu_handler::menu_delete)
            .service(menu_handler::menu_update))
    )
        .bind(("127.0.0.1", 8101))?
        .run()
        .await
}