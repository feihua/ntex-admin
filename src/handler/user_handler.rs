use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, sql_query};
use diesel::associations::HasTable;
use diesel::sql_types::Bigint;
use futures::future::ok;
use log::info;
use ntex::http::header;
use ntex::web;

use crate::model::menu::{StringColumn, SysMenu};
use crate::model::role::SysRole;
use crate::model::user::{SysUser, SysUserAdd, SysUserUpdate};
use crate::model::user_role::{SysUserRole, SysUserRoleAdd};
use crate::RB;
use crate::schema::sys_menu::api_url;
use crate::schema::sys_menu::dsl::sys_menu;
use crate::schema::sys_role::dsl::sys_role;
use crate::schema::sys_user::{id, mobile, password};
use crate::schema::sys_user::dsl::sys_user;
use crate::schema::sys_user_role::{role_id, user_id};
use crate::schema::sys_user_role::dsl::sys_user_role;
use crate::utils::error::WhoUnfollowedError;
use crate::utils::jwt_util::JWTToken;
use crate::vo::{err_result_msg, handle_result, ok_result_data, ok_result_msg, ok_result_page};
use crate::vo::user_vo::*;

// 后台用户登录
#[web::post("/login")]
pub async fn login(item: web::types::Json<UserLoginReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("user login params: {:?}", item);

    let conn_result = &mut RB.clone().get();

    if let Ok(conn) = conn_result {
        let user_result = sys_user.filter(mobile.eq(&item.mobile)).first::<SysUser>(conn);

        if let Ok(user) = user_result {
            log::info!("select_by_mobile: {:?}", user);

            if user.password.ne(&item.password) {
                return Ok(web::HttpResponse::Ok().json(&err_result_msg("密码不正确".to_string())));
            }

            let btn_menu = query_btn_menu(user.id);

            if btn_menu.len() == 0 {
                return Ok(web::HttpResponse::Ok().json(&err_result_msg("用户没有分配角色或者菜单,不能登录".to_string())));
            }

            match JWTToken::new(user.id, &user.user_name, btn_menu).create_token("123") {
                Ok(token) => {
                    Ok(web::HttpResponse::Ok().json(&ok_result_data(token)))
                }
                Err(err) => {
                    let er = match err {
                        WhoUnfollowedError::JwtTokenError(s) => { s }
                        _ => "no math error".to_string()
                    };

                    Ok(web::HttpResponse::Ok().json(&err_result_msg(er)))
                }
            }
        } else {
            Ok(web::HttpResponse::Ok().json(&err_result_msg("根据手机号查询用户异常".to_string())))
        }
    } else {
        Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接异常".to_string())))
    }
}


fn query_btn_menu(u_id: i64) -> Vec<String> {
    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let user_role_sql = sql_query("SELECT * FROM sys_user_role where user_id = ? and role_id = 1");
        let user_role_result = user_role_sql.bind::<Bigint, _>(&u_id).get_result::<SysUserRole>(conn);

        match user_role_result {
            Ok(_) => {
                log::info!("admin login: {:?}",u_id);

                // let sys_menu_sql = sql_query("SELECT api_url FROM sys_menu");
                // let sys_menu_result = sys_menu_sql.load::<String>(conn);

                let sys_menu_result = sys_menu.select(api_url).load::<String>(conn);
                log::info!("admin login: {:?}",sys_menu_result);
                if let Ok(btn) = sys_menu_result {
                    return btn;
                }
                Vec::new()
            }
            Err(_) => {
                log::info!("ordinary login: {:?}",u_id);
                let result = sql_query("select u.api_url from sys_user_role t \
            left join sys_role usr on t.role_id = usr.id \
            left join sys_role_menu srm on usr.id = srm.role_id \
            left join sys_menu u on srm.menu_id = u.id \
            where t.user_id = ?")
                    .bind::<Bigint, _>(&u_id)
                    .load::<StringColumn>(conn);

                if let Ok(btn) = result {
                    let mut btn_list: Vec<String> = Vec::new();
                    for x in btn {
                        btn_list.push(x.api_url)
                    }
                    return btn_list;
                }
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}

#[web::post("/query_user_role")]
pub async fn query_user_role(item: web::types::Json<QueryUserRoleReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("query_user_role params: {:?}", item);

    let conn_result = &mut RB.clone().get();

    if let Ok(conn) = conn_result {
        let user_role_result = sys_user_role.filter(user_id.eq(&item.user_id)).select(role_id).load::<i64>(conn);
        let mut user_role_ids: Vec<i64> = Vec::new();

        if let Ok(ids) = user_role_result {
            user_role_ids = ids
        }

        let sys_role_result = sys_role.load::<SysRole>(conn);
        let mut role_list: Vec<UserRoleList> = Vec::new();

        if let Ok(sys_role_list) = sys_role_result {
            for x in sys_role_list {
                role_list.push(UserRoleList {
                    id: x.id,
                    status_id: x.status_id,
                    sort: x.sort,
                    role_name: x.role_name,
                    remark: x.remark,
                    create_time: x.create_time.to_string(),
                    update_time: x.update_time.to_string(),
                });
            }
        }

        Ok(web::HttpResponse::Ok().json(&ok_result_data(QueryUserRoleData {
            sys_role_list: role_list,
            user_role_ids,
        })))
    } else {
        Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接异常".to_string())))
    }
}

#[web::post("/update_user_role")]
pub async fn update_user_role(item: web::types::Json<UpdateUserRoleReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("update_user_role params: {:?}", item);

    let user_role = item.0;
    let u_id = user_role.user_id;
    let role_ids = user_role.role_ids;

    if u_id == 1 {
        return Ok(web::HttpResponse::Ok().json(&err_result_msg("不能修改超级管理员的角色".to_string())));
    }

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let sys_result = diesel::delete(sys_user_role.filter(user_id.eq(u_id))).execute(conn);

        return match sys_result {
            Ok(_) => {
                let mut sys_role_user_list: Vec<SysUserRoleAdd> = Vec::new();
                for r_id in role_ids {
                    sys_role_user_list.push(SysUserRoleAdd {
                        status_id: 1,
                        sort: 1,
                        role_id: r_id,
                        user_id: u_id.clone(),
                    })
                }
                let result = diesel::insert_into(sys_user_role::table()).values(&sys_role_user_list).execute(conn);
                Ok(web::HttpResponse::Ok().json(&handle_result(result)))
            }
            Err(err) => {
                Ok(web::HttpResponse::Ok().json(&err_result_msg(err.to_string())))
            }
        };
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}


#[web::get("/query_user_menu")]
pub async fn query_user_menu(req: web::HttpRequest) -> Result<impl web::Responder, web::Error> {
    let def = header::HeaderValue::from_str("").unwrap();
    let token = req
        .headers()
        .get("Authorization")
        .unwrap_or(&def)
        .to_str()
        .ok()
        .unwrap();

    let split_vec = token.split_whitespace().collect::<Vec<_>>();
    if split_vec.len() != 2 || split_vec[0] != "Bearer" {
        return Ok(web::HttpResponse::Ok().json(&err_result_msg("the token format wrong".to_string())));
    }
    let token = split_vec[1];
    let jwt_token_e = JWTToken::verify("123", &token);
    let jwt_token = match jwt_token_e {
        Ok(data) => { data }
        Err(err) => {
            return Ok(web::HttpResponse::Ok().json(&err_result_msg(err.to_string())));
        }
    };

    log::info!("query user menu params {:?}",jwt_token);

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        //根据id查询用户
        let result = sql_query("select * from sys_user where id = ?")
            .bind::<Bigint, _>(jwt_token.id)
            .get_result::<SysUser>(conn);

        match result {
            Ok(user) => {
                let user_role_sql = sql_query("SELECT * FROM sys_user_role where user_id = ? and role_id = 1");
                let user_role_result = user_role_sql.bind::<Bigint, _>(&user.id).get_result::<SysUserRole>(conn);

                let mut sys_menu_list: Vec<SysMenu> = Vec::new();
                match user_role_result {
                    Ok(_) => {
                        match sys_menu.load::<SysMenu>(conn) {
                            Ok(s_menus) => {
                                sys_menu_list = s_menus;
                            }
                            Err(_) => {
                                // sys_menu_list
                            }
                        }
                    }
                    Err(_) => {
                        let result = sql_query("select u.* from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = ? order by u.id asc")
                            .bind::<Bigint, _>(&jwt_token.id)
                            .load::<SysMenu>(conn);
                        match result {
                            Ok(s_menus) => {
                                sys_menu_list = s_menus;
                            }
                            Err(err) => {
                                info!("菜单不存在：{:?}",&err)
                            }
                        }
                    }
                }


                let mut sys_menu_map: HashMap<i64, MenuUserList> = HashMap::new();
                let mut sys_user_menu_list: Vec<MenuUserList> = Vec::new();
                let mut btn_menu: Vec<String> = Vec::new();
                let mut sys_menu_parent_ids: Vec<i64> = Vec::new();

                for x in sys_menu_list {
                    if x.menu_type != 3 {
                        sys_menu_map.insert(x.id.clone(), MenuUserList {
                            id: x.id,
                            parent_id: x.parent_id,
                            name: x.menu_name,
                            icon: x.menu_icon.unwrap_or_default(),
                            api_url: x.api_url.clone(),
                            menu_type: x.menu_type,
                            path: x.menu_url,
                        });
                        sys_menu_parent_ids.push(x.parent_id.clone())
                    }

                    btn_menu.push(x.api_url);
                }

                for menu_id in sys_menu_parent_ids {
                    let s_result = sql_query("select * from sys_menu where id = ?")
                        .bind::<Bigint, _>(&menu_id)
                        .get_result::<SysMenu>(conn);
                    match s_result {
                        Ok(menu) => {
                            sys_menu_map.insert(menu.id.clone(), MenuUserList {
                                id: menu.id,
                                parent_id: menu.parent_id,
                                name: menu.menu_name,
                                icon: menu.menu_icon.unwrap_or_default(),
                                api_url: menu.api_url,
                                menu_type: menu.menu_type,
                                path: menu.menu_url,
                            });
                        }
                        Err(err) => {
                            info!("菜单不存在：{:?}",&err)
                        }
                    }
                }

                let mut sys_menu_ids: Vec<i64> = Vec::new();
                for menu in &sys_menu_map {
                    sys_menu_ids.push(menu.0.abs())
                }

                sys_menu_ids.sort();

                for m_id in sys_menu_ids {
                    let menu = sys_menu_map.get(&m_id).cloned().unwrap();
                    sys_user_menu_list.push(menu)
                }


                return Ok(web::HttpResponse::Ok().json(&ok_result_data(QueryUserMenuData {
                    sys_menu: sys_user_menu_list,
                    btn_menu,
                    avatar: "https://gw.alipayobjects.com/zos/antfincdn/XAosXuNZyF/BiazfanxmamNRoxxVxka.png".to_string(),
                    name: user.user_name,
                })));
            }

            // 查询用户数据库异常
            Err(err) => {
                return Ok(web::HttpResponse::Ok().json(&err_result_msg(err.to_string())));
            }
        }
    }

    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 查询用户列表
#[web::post("/user_list")]
pub async fn user_list(item: web::types::Json<UserListReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("query user_list params: {:?}", &item);

    // let mobile = item.mobile.as_deref().unwrap_or_default();
    // let status_id = &item.status_id;

    let mut list_data: Vec<UserListData> = Vec::new();

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let sys_user_result = sys_user.load::<SysUser>(conn);

        if let Ok(sys_user_list) = sys_user_result {
            for user in sys_user_list {
                list_data.push(UserListData {
                    id: user.id,
                    sort: user.sort,
                    status_id: user.status_id,
                    mobile: user.mobile,
                    user_name: user.user_name,
                    remark: user.remark.unwrap_or_default(),
                    create_time: user.create_time.to_string(),
                    update_time: user.update_time.to_string(),
                })
            }
        }
    }
    Ok(web::HttpResponse::Ok().json(&ok_result_page(list_data, 10)))
}

// 添加用户信息
#[web::post("/user_save")]
pub async fn user_save(item: web::types::Json<UserSaveReq>) -> Result<impl web::Responder, web::Error> {
    info!("user_save params: {:?}", &item);

    let user = item.0;

    let s_user = SysUserAdd {
        status_id: user.status_id,
        sort: user.sort,
        mobile: user.mobile,
        user_name: user.user_name,
        remark: user.remark,
        password: "123456".to_string(),//默认密码为123456,暂时不加密
    };

    Ok(web::HttpResponse::Ok().json(&SysUser::add_user(s_user)))
}

// 更新用户信息
#[web::post("/user_update")]
pub async fn user_update(item: web::types::Json<UserUpdateReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("user_update params: {:?}", &item);

    let user = item.0;

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let user_sql = sql_query("SELECT * FROM sys_user where id = ? ");
        let result = user_sql.bind::<Bigint, _>(user.id).get_result::<SysUser>(conn);

        return match result {
            Ok(s_user) => {
                let s_user = SysUserUpdate {
                    id: user.id.clone(),
                    status_id: user.status_id,
                    sort: user.sort,
                    mobile: user.mobile,
                    user_name: user.user_name,
                    remark: user.remark,
                    password: s_user.password.clone(),
                };

                let result = diesel::update(sys_user.filter(id.eq(user.id.clone())))
                    .set(s_user)
                    .execute(conn);
                Ok(web::HttpResponse::Ok().json(&handle_result(result)))
            }
            Err(err) => {
                Ok(web::HttpResponse::Ok().json(&err_result_msg(err.to_string())))
            }
        };
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 删除用户信息
#[web::post("/user_delete")]
pub async fn user_delete(item: web::types::Json<UserDeleteReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("user_delete params: {:?}", &item);

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let mut ids = item.ids.clone();
        //id为1的用户为系统预留用户,不能删除
        if ids.contains(&1) {
            ids.remove(1);
        }

        let result = diesel::delete(sys_user.filter(id.eq_any(ids))).execute(conn);
        return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
    }

    Ok(web::HttpResponse::Ok().json(&ok_result_msg("删除用户信息成功".to_string())))
}

// 更新用户密码
#[web::post("/update_user_password")]
pub async fn update_user_password(item: web::types::Json<UpdateUserPwdReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("update_user_pwd params: {:?}", &item);

    let user_pwd = item.0;

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let user_sql = sql_query("SELECT * FROM sys_use where user_id = ? ");
        let sys_user_result = user_sql.bind::<Bigint, _>(user_pwd.id).get_result::<SysUser>(conn);

        return match sys_user_result {
            Ok(user) => {
                if user.password == user_pwd.pwd {
                    let result = diesel::update(sys_user.filter(id.eq(user_pwd.id.clone()))).set(password.eq(&user_pwd.re_pwd)).execute(conn);
                    Ok(web::HttpResponse::Ok().json(&handle_result(result)))
                } else {
                    Ok(web::HttpResponse::Ok().json(&err_result_msg("旧密码不正确".to_string())))
                }
            }
            Err(err) => {
                Ok(web::HttpResponse::Ok().json(&err_result_msg(err.to_string())))
            }
        };
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}