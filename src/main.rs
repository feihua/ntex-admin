#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rbatis;

use std::env;

use dotenvy::dotenv;
use ntex::web;
use rbatis::RBatis;
use crate::handler::system::{sys_menu_handler, sys_role_handler, sys_user_handler};

pub mod common;
pub mod handler;
pub mod middleware;
pub mod model;
pub mod utils;
pub mod vo;

lazy_static! {
    static ref RB: RBatis = RBatis::new();
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("src/config/log4rs.yaml", Default::default()).unwrap();
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    RB.init(rbdc_mysql::driver::MysqlDriver {}, db_url.as_str())
        .unwrap();

    web::HttpServer::new(|| {
        web::App::new()
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
