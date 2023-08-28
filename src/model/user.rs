use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Bigint;
use ntex::web;
use serde::{Deserialize, Serialize};

use crate::RB;
use crate::schema::sys_user::dsl::sys_user;
use crate::vo::{BaseResponse, err_result_msg, handle_result};

#[derive(Insertable, Debug, PartialEq, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::sys_user)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct SysUserAdd {
    pub mobile: String,
    pub user_name: String,
    pub password: String,
    pub status_id: i32,
    pub sort: i32,
    pub remark: Option<String>,

}


#[derive(Debug, PartialEq, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::sys_user)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct SysUserUpdate {
    pub id: i64,
    pub mobile: String,
    pub user_name: String,
    pub password: String,
    pub status_id: i32,
    pub sort: i32,
    pub remark: Option<String>,

}

#[derive(Queryable, Selectable, Insertable, Debug, PartialEq, Serialize, Deserialize, QueryableByName, AsChangeset)]
#[diesel(table_name = crate::schema::sys_user)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct SysUser {
    pub id: i64,
    pub mobile: String,
    pub user_name: String,
    pub password: String,
    pub status_id: i32,
    pub sort: i32,
    pub remark: Option<String>,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,

}

impl SysUser {
    // 添加用户信息
    pub fn add_user(s_user: SysUserAdd) -> BaseResponse<String> {
        let conn_result = &mut RB.clone().get();
        if let Ok(conn) = conn_result {
            let result = diesel::insert_into(sys_user::table()).values(s_user).execute(conn);
            return handle_result(result);
        }

        err_result_msg("获取数据库连接失败".to_string())
    }
}
