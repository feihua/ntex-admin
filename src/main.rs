use std::env;
#[macro_use]
extern crate rbatis;
#[macro_use]
extern crate lazy_static;
use dotenvy::dotenv;
use ntex::web;
use rbatis::RBatis;

use crate::handler::{menu_handler, role_handler, user_handler};

pub mod handler;
pub mod model;
pub mod vo;
pub mod utils;
pub mod middleware;

lazy_static! {
    static ref RB: RBatis = RBatis::new();
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("src/config/log4rs.yaml", Default::default()).unwrap();
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    RB.init(rbdc_mysql::driver::MysqlDriver {}, db_url.as_str()).unwrap();

    web::HttpServer::new(|| web::App::new()
        .wrap(web::middleware::Logger::default())
        .wrap(middleware::auth::JwtAuth)
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