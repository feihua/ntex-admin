#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rbatis;

use std::env;

use crate::handler::system::{
    sys_dept_handler, sys_dict_data_handler, sys_dict_type_handler, sys_login_log_handler, sys_menu_handler, sys_notice_handler, sys_operate_log_handler, sys_post_handler, sys_role_handler,
    sys_user_handler,
};
use dotenvy::dotenv;
use ntex::web;
use rbatis::rbdc::pool::{ConnectionManager, Pool};
use rbatis::RBatis;
use rbdc_mysql::MysqlDriver;
use rbdc_pool_fast::FastPool;

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
    let manager = ConnectionManager::new(MysqlDriver {}, db_url.as_str()).expect("create connection manager error");
    let pool = FastPool::new(manager).expect("create db pool error");

    RB.init_pool(pool).expect("init db pool error");

    web::HttpServer::new(|| {
        web::App::new().wrap(web::middleware::Logger::default()).wrap(middleware::auth::JwtAuth).service(
            (web::scope("/api/system"))
                .service(sys_user_handler::add_sys_user)
                .service(sys_user_handler::delete_sys_user)
                .service(sys_user_handler::update_sys_user)
                .service(sys_user_handler::update_sys_user_status)
                .service(sys_user_handler::update_sys_user_password)
                .service(sys_user_handler::reset_sys_user_password)
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
                .service(sys_role_handler::query_allocated_list)
                .service(sys_role_handler::query_unallocated_list)
                .service(sys_role_handler::cancel_auth_user)
                .service(sys_role_handler::batch_cancel_auth_user)
                .service(sys_role_handler::batch_auth_user)
                .service(sys_menu_handler::add_sys_menu)
                .service(sys_menu_handler::delete_sys_menu)
                .service(sys_menu_handler::update_sys_menu)
                .service(sys_menu_handler::update_sys_menu_status)
                .service(sys_menu_handler::query_sys_menu_detail)
                .service(sys_menu_handler::query_sys_menu_list)
                .service(sys_menu_handler::query_sys_menu_list_simple)
                .service(sys_menu_handler::query_sys_menu_resource_list)
                .service(sys_post_handler::add_sys_post)
                .service(sys_post_handler::delete_sys_post)
                .service(sys_post_handler::update_sys_post)
                .service(sys_post_handler::update_sys_post_status)
                .service(sys_post_handler::query_sys_post_detail)
                .service(sys_post_handler::query_sys_post_list)
                .service(sys_operate_log_handler::delete_sys_operate_log)
                .service(sys_operate_log_handler::query_sys_operate_log_detail)
                .service(sys_operate_log_handler::query_sys_operate_log_list)
                .service(sys_notice_handler::add_sys_notice)
                .service(sys_notice_handler::delete_sys_notice)
                .service(sys_notice_handler::update_sys_notice)
                .service(sys_notice_handler::update_sys_notice_status)
                .service(sys_notice_handler::query_sys_notice_detail)
                .service(sys_notice_handler::query_sys_notice_list)
                .service(sys_login_log_handler::delete_sys_login_log)
                .service(sys_login_log_handler::query_sys_login_log_detail)
                .service(sys_login_log_handler::query_sys_login_log_list)
                .service(sys_dict_type_handler::add_sys_dict_type)
                .service(sys_dict_type_handler::delete_sys_dict_type)
                .service(sys_dict_type_handler::update_sys_dict_type)
                .service(sys_dict_type_handler::update_sys_dict_type_status)
                .service(sys_dict_type_handler::query_sys_dict_type_detail)
                .service(sys_dict_type_handler::query_sys_dict_type_list)
                .service(sys_dict_data_handler::add_sys_dict_data)
                .service(sys_dict_data_handler::delete_sys_dict_data)
                .service(sys_dict_data_handler::update_sys_dict_data)
                .service(sys_dict_data_handler::update_sys_dict_data_status)
                .service(sys_dict_data_handler::query_sys_dict_data_detail)
                .service(sys_dict_data_handler::query_sys_dict_data_list)
                .service(sys_dept_handler::add_sys_dept)
                .service(sys_dept_handler::delete_sys_dept)
                .service(sys_dept_handler::update_sys_dept)
                .service(sys_dept_handler::update_sys_dept_status)
                .service(sys_dept_handler::query_sys_dept_detail)
                .service(sys_dept_handler::query_sys_dept_list),
        )
    })
    .bind(("127.0.0.1", 8101))?
    .run()
    .await
}
