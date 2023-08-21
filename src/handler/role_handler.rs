use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::associations::HasTable;
use ntex::web;


use crate::{RB, schema};
use crate::model::menu::SysMenu;
use crate::model::role::{SysRole, SysRoleAdd, SysRoleUpdate};
use crate::model::role_menu::{SysRoleMenuAdd};
use crate::schema::sys_menu::dsl::sys_menu;
use crate::schema::sys_role::dsl::sys_role;
use crate::schema::sys_role::id;
use crate::schema::sys_role_menu::{menu_id, role_id};
use crate::schema::sys_role_menu::dsl::sys_role_menu;
use crate::schema::sys_user_role::dsl::sys_user_role;
use crate::vo::{err_result_msg, handle_result, ok_result_data, ok_result_msg, ok_result_page};
use crate::vo::role_vo::*;

// 查询角色列表
#[web::post("/role_list")]
pub async fn role_list(item: web::types::Json<RoleListReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("role_list params: {:?}", &item);

    // let role_name = item.role_name.clone().unwrap_or_default();
    // let status_id = item.status_id.clone().unwrap_or_default();

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let sys_role_result = sys_role.load::<SysRole>(conn);

        let mut role_list: Vec<RoleListData> = Vec::new();

        if let Ok(sys_role_list) = sys_role_result {
            for role in sys_role_list {
                role_list.push(RoleListData {
                    id: role.id,
                    sort: role.sort,
                    status_id: role.status_id,
                    role_name: role.role_name,
                    remark: role.remark,
                    create_time: role.create_time.to_string(),
                    update_time: role.update_time.to_string(),
                })
            }
        }


        return Ok(web::HttpResponse::Ok().json(&ok_result_page(role_list, 10)));
    }

    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 添加角色信息
#[web::post("/role_save")]
pub async fn role_save(item: web::types::Json<RoleSaveReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("role_save params: {:?}", &item);

    let role = item.0;

    let s_role = SysRoleAdd {
        status_id: role.status_id,
        sort: role.sort,
        role_name: role.role_name,
        remark: role.remark.unwrap(),
    };

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let result = diesel::insert_into(sys_role::table()).values(s_role).execute(conn);

        return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 更新角色信息
#[web::post("/role_update")]
pub async fn role_update(item: web::types::Json<RoleUpdateReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("role_update params: {:?}", &item);

    let role = item.0;

    let s_role = SysRoleUpdate {
        id: role.id,
        status_id: role.status_id,
        sort: role.sort,
        role_name: role.role_name,
        remark: role.remark.unwrap_or_default(),
    };

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let result = diesel::update(sys_role).filter(id.eq(&role.id)).set(s_role).execute(conn);

        return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 删除角色信息
#[web::post("/role_delete")]
pub async fn role_delete(item: web::types::Json<RoleDeleteReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("role_delete params: {:?}", &item);


    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let ids = item.ids.clone();
        let count_result = sys_user_role.filter(schema::sys_user_role::role_id.eq_any(ids)).count().get_result::<i64>(conn);

        if let Ok(count) = count_result {
            if count == 0 {
                let result = diesel::delete(sys_role.filter(id.eq(3))).execute(conn);
                return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
            }
        }
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 查询角色关联的菜单
#[web::post("/query_role_menu")]
pub async fn query_role_menu(item: web::types::Json<QueryRoleMenuReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("query_role_menu params: {:?}", &item);

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let mut menu_data_list: Vec<MenuDataList> = Vec::new();
        let mut role_menu_ids: Vec<i64> = Vec::new();
        // 查询所有菜单
        let menu_list_result = sys_menu.load::<SysMenu>(conn);

        if let Ok(menu_list) = menu_list_result {
            for x in menu_list {
                menu_data_list.push(MenuDataList {
                    id: x.id.clone(),
                    parent_id: x.parent_id,
                    title: x.menu_name.clone(),
                    key: x.id.to_string(),
                    label: x.menu_name,
                    is_penultimate: x.parent_id == 2,
                });
                role_menu_ids.push(x.id)
            }
        }

        //不是超级管理员的时候,就要查询角色和菜单的关联
        if item.role_id != 1 {
            role_menu_ids.clear();
            let sys_user_result = sys_role_menu.filter(role_id.eq(item.role_id.clone())).select(menu_id).load::<i64>(conn);
            // let role_menu_sql = sql_query("SELECT menu_id FROM sys_role_menu where role_id = ? ");
            // let sys_user_result = role_menu_sql.bind::<Bigint, _>(item.role_id.clone()).get_results::<i64>(conn);

            match sys_user_result {
                Ok(menu_ids) => {
                    role_menu_ids = menu_ids
                }
                Err(err) => {
                    return Ok(web::HttpResponse::Ok().json(&ok_result_msg(err.to_string())));
                }
            }
        }

        return Ok(web::HttpResponse::Ok().json(&ok_result_data(QueryRoleMenuData {
            role_menus: role_menu_ids,
            menu_list: menu_data_list,
        })));
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 更新角色关联的菜单
#[web::post("/update_role_menu")]
pub async fn update_role_menu(item: web::types::Json<UpdateRoleMenuReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("update_role_menu params: {:?}", &item);
    let r_id = item.role_id.clone();
    let menu_ids = item.menu_ids.clone();
    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let result = diesel::delete(sys_role_menu.filter(role_id.eq(r_id))).execute(conn);

        match result {
            Ok(_) => {
                let mut role_menu: Vec<SysRoleMenuAdd> = Vec::new();

                for m_id in menu_ids {
                    role_menu.push(SysRoleMenuAdd {
                        status_id: 1,
                        sort: 1,
                        menu_id: m_id.clone(),
                        role_id: r_id.clone(),
                    })
                }

                let result = diesel::insert_into(sys_role_menu::table()).values(role_menu).execute(conn);

                return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
            }
            Err(err) => {
                return Ok(web::HttpResponse::Ok().json(&err_result_msg(err.to_string())));
            }
        }
    }
    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}
