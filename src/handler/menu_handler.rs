use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::associations::HasTable;
use ntex::web;

use crate::model::menu::{SysMenu, SysMenuAdd, SysMenuUpdate};
use crate::RB;
use crate::schema::sys_menu::{id, parent_id};
use crate::schema::sys_menu::dsl::sys_menu;
use crate::vo::{err_result_msg, handle_result, ok_result_data};
use crate::vo::menu_vo::{*};

// 查询菜单
#[web::post("/menu_list")]
pub async fn menu_list(item: web::types::Json<MenuListReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("menu_list params: {:?}", &item);

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let mut menu_list: Vec<MenuListData> = Vec::new();

        let menu_result = sys_menu.load::<SysMenu>(conn);
        if let Ok(menus) = menu_result {
            for menu in menus {
                menu_list.push(MenuListData {
                    id: menu.id,
                    sort: menu.sort,
                    status_id: menu.status_id,
                    parent_id: menu.parent_id,
                    menu_name: menu.menu_name.clone(),
                    label: menu.menu_name,
                    menu_url: menu.menu_url,
                    icon: menu.menu_icon.unwrap_or_default(),
                    api_url: menu.api_url,
                    remark: menu.remark.unwrap_or_default(),
                    menu_type: menu.menu_type,
                    create_time: menu.create_time.to_string(),
                    update_time: menu.update_time.to_string(),
                })
            }
        }


        return Ok(web::HttpResponse::Ok().json(&ok_result_data(menu_list)));
    }

    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 添加菜单
#[web::post("/menu_save")]
pub async fn menu_save(item: web::types::Json<MenuSaveReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("menu_save params: {:?}", &item);
    let menu = item.0;

    let s_menu = SysMenuAdd {
        status_id: menu.status_id,
        sort: menu.sort,
        parent_id: menu.parent_id.unwrap_or(0),
        menu_name: menu.menu_name,
        menu_url: menu.menu_url,
        api_url: menu.api_url,
        menu_icon: menu.icon,
        remark: menu.remark,
        menu_type: menu.menu_type,
    };

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let result = diesel::insert_into(sys_menu::table()).values(s_menu).execute(conn);

        return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
    }

    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 更新菜单
#[web::post("/menu_update")]
pub async fn menu_update(item: web::types::Json<MenuUpdateReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("menu_update params: {:?}", &item);
    let menu = item.0;

    let s_menu = SysMenuUpdate {
        id: menu.id,
        status_id: menu.status_id,
        sort: menu.sort,
        parent_id: menu.parent_id,
        menu_name: menu.menu_name,
        menu_url: menu.menu_url,
        api_url: menu.api_url,
        menu_icon: menu.icon,
        remark: menu.remark,
        menu_type: menu.menu_type,
    };

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        let result = diesel::update(sys_menu).filter(id.eq(&menu.id)).set(s_menu).execute(conn);

        return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
    }

    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}

// 删除菜单信息
#[web::post("/menu_delete")]
pub async fn menu_delete(item: web::types::Json<MenuDeleteReq>) -> Result<impl web::Responder, web::Error> {
    log::info!("menu_delete params: {:?}", &item);

    let conn_result = &mut RB.clone().get();
    if let Ok(conn) = conn_result {
        //有下级的时候 不能直接删除
        let count_result = sys_menu.filter(parent_id.eq(&item.id)).count().get_result::<i64>(conn);

        if let Ok(count) = count_result {
            if count > 0 {
                return Ok(web::HttpResponse::Ok().json(&err_result_msg("有下级菜单,不能直接删除".to_string())));
            }
        }

        let result = diesel::delete(sys_menu.filter(id.eq(3))).execute(conn);

        return Ok(web::HttpResponse::Ok().json(&handle_result(result)));
    }

    Ok(web::HttpResponse::Ok().json(&err_result_msg("获取数据库连接失败".to_string())))
}