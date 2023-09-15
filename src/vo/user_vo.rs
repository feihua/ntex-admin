use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginReq {
    pub mobile: String,
    pub password: String,
}


#[derive(Debug, Deserialize)]
pub struct QueryUserRoleReq {
    pub user_id: i32,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct QueryUserRoleData {
    pub sys_role_list: Vec<UserRoleList>,
    pub user_role_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleList {
    pub id: i64,
    pub status_id: i8,
    pub sort: i32,
    pub role_name: String,
    pub remark: String,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleReq {
    pub user_id: i64,
    pub role_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryUserMenuReq {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryUserMenuResp {
    pub msg: String,
    pub code: i32,
    pub data: QueryUserMenuData,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryUserMenuData {
    pub sys_menu: HashSet<MenuUserList>,
    pub btn_menu: HashSet<String>,
    pub avatar: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct MenuUserList {
    pub id: i64,
    pub parent_id: i64,
    pub name: String,
    pub path: String,
    pub api_url: String,
    pub menu_type: i8,
    pub icon: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserListReq {
    #[serde(rename = "current")]
    pub page_no: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
    pub mobile: Option<String>,
    pub status_id: Option<i8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResp {
    pub msg: String,
    pub code: i32,
    pub success: bool,
    pub total: u64,
    pub data: Option<Vec<UserListData>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListData {
    pub id: i64,
    pub sort: i32,
    pub status_id: i32,
    pub mobile: String,
    pub user_name: String,
    pub remark: String,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSaveReq {
    pub mobile: String,
    pub user_name: String,
    pub status_id: i32,
    pub sort: i32,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateReq {
    pub id: i64,
    pub sort: i32,
    pub status_id: i32,
    pub mobile: String,
    pub user_name: String,
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeleteReq {
    pub ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserPwdReq {
    pub id: i64,
    pub pwd: String,
    pub re_pwd: String,
}