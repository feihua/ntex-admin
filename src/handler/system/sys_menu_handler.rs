use log::info;
use ntex::web;
use ntex::web::types::Json;
use ntex::web::Responder;

use diesel::associations::HasTable;
use diesel::sql_types::*;
use diesel::{sql_query, ExpressionMethods, QueryDsl, RunQueryDsl};
use log::{debug, error};

use crate::RB;

use crate::common::result::BaseResponse;
use crate::model::system::sys_menu_model::*;
use crate::schema::sys_menu::dsl::sys_menu;
use crate::schema::sys_menu::*;
use crate::vo::system::sys_menu_vo::*;

/*
 *添加菜单信息
 *author：刘飞华
 *date：2024/12/20 10:04:30
 */
#[web::post("/menu/addMenu")]
pub async fn add_sys_menu(item: Json<AddMenuReq>) -> impl Responder {
    info!("add sys_menu params: {:?}", &item);

    let req = item.0;

    let add_sys_menu_param = AddSysMenu {
        menu_name: req.menu_name,        //菜单名称
        menu_type: req.menu_type,        //菜单类型(1：目录   2：菜单   3：按钮)
        status: req.status,              //状态(1:正常，0:禁用)
        sort: req.sort,                  //排序
        parent_id: req.parent_id,        //父ID
        menu_url: req.menu_url,          //路由路径
        api_url: req.api_url,            //接口URL
        menu_icon: req.menu_icon,        //菜单图标
        remark: req.remark,              //备注
        create_time: Default::default(), //创建时间
        update_time: Default::default(), //修改时间
    };

    match &mut RB.clone().get() {
        Ok(conn) => {
            let result = diesel::insert_into(sys_menu::table())
                .values(add_sys_menu_param)
                .execute(conn);
            match result {
                Ok(_u) => BaseResponse::<String>::ok_result(),
                Err(err) => BaseResponse::<String>::err_result_msg(err.to_string()),
            }
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            BaseResponse::<String>::err_result_msg(err.to_string())
        }
    }
}

/*
 *删除菜单信息
 *author：刘飞华
 *date：2024/12/20 10:04:30
 */
#[web::post("/menu/deleteMenu")]
pub async fn delete_sys_menu(item: Json<DeleteMenuReq>) -> impl Responder {
    info!("delete sys_menu params: {:?}", &item);
    let req = item.0;

    match &mut RB.clone().get() {
        Ok(conn) => {
            match sys_menu
                .filter(parent_id.eq(&req.id))
                .count()
                .get_result::<i64>(conn)
            {
                Ok(count) => {
                    if count > 0 {
                        error!("err:{}", "有下级菜单,不能直接删除".to_string());
                        return BaseResponse::<String>::err_result_msg(
                            "有下级菜单,不能直接删除".to_string(),
                        );
                    }
                    let result = diesel::delete(sys_menu.filter(id.eq(&req.id))).execute(conn);
                    match result {
                        Ok(_u) => BaseResponse::<String>::ok_result(),
                        Err(err) => BaseResponse::<String>::err_result_msg(err.to_string()),
                    }
                }
                Err(err) => {
                    error!("err:{}", err.to_string());
                    BaseResponse::<String>::err_result_msg(err.to_string())
                }
            }
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            BaseResponse::<String>::err_result_msg(err.to_string())
        }
    }
}

/*
 *更新菜单信息
 *author：刘飞华
 *date：2024/12/20 10:04:30
 */
#[web::post("/menu/updateMenu")]
pub async fn update_sys_menu(item: Json<UpdateMenuReq>) -> impl Responder {
    info!("update sys_menu params: {:?}", &item);

    let req = item.0;

    let update_sys_menu_param = UpdateSysMenu {
        id: req.id,                      //主键
        menu_name: req.menu_name,        //菜单名称
        menu_type: req.menu_type,        //菜单类型(1：目录   2：菜单   3：按钮)
        status: req.status,              //状态(1:正常，0:禁用)
        sort: req.sort,                  //排序
        parent_id: req.parent_id,        //父ID
        menu_url: req.menu_url,          //路由路径
        api_url: req.api_url,            //接口URL
        menu_icon: req.menu_icon,        //菜单图标
        remark: req.remark,              //备注, //创建时间
        create_time: Default::default(), //修改时间
        update_time: Default::default(),
    };

    match &mut RB.clone().get() {
        Ok(conn) => {
            let result = diesel::update(sys_menu)
                .filter(id.eq(&req.id))
                .set(update_sys_menu_param)
                .execute(conn);
            match result {
                Ok(_u) => BaseResponse::<String>::ok_result(),
                Err(err) => BaseResponse::<String>::err_result_msg(err.to_string()),
            }
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            BaseResponse::<String>::err_result_msg(err.to_string())
        }
    }
}

/*
 *更新菜单信息状态
 *author：刘飞华
 *date：2024/12/20 10:04:30
 */
#[web::post("/menu/updateMenuStatus")]
pub async fn update_sys_menu_status(item: Json<UpdateMenuStatusReq>) -> impl Responder {
    info!("update sys_menu_status params: {:?}", &item);
    let req = item.0;

    match &mut RB.clone().get() {
        Ok(conn) => {
            let result = diesel::update(sys_menu)
                .filter(id.eq_any(&req.ids))
                .set(status.eq(req.status))
                .execute(conn);
            match result {
                Ok(_u) => BaseResponse::<String>::ok_result(),
                Err(err) => BaseResponse::<String>::err_result_msg(err.to_string()),
            }
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            BaseResponse::<String>::err_result_msg(err.to_string())
        }
    }
}

/*
 *查询菜单信息详情
 *author：刘飞华
 *date：2024/12/20 10:04:30
 */
#[web::post("/menu/queryMenuDetail")]
pub async fn query_sys_menu_detail(item: Json<QueryMenuDetailReq>) -> impl Responder {
    info!("query sys_menu_detail params: {:?}", &item);
    let req = item.0;

    match &mut RB.clone().get() {
        Ok(conn) => {
            let sys_menu_sql = sql_query("SELECT * FROM sys_menu WHERE id = ?");
            let result = sys_menu_sql
                .bind::<Bigint, _>(&req.id)
                .get_result::<SysMenu>(conn);
            match result {
                Ok(x) => {
                    let data = QueryMenuDetailResp {
                        id: x.id,                               //主键
                        menu_name: x.menu_name,                 //菜单名称
                        menu_type: x.menu_type, //菜单类型(1：目录   2：菜单   3：按钮)
                        status: x.status,       //状态(1:正常，0:禁用)
                        sort: x.sort,           //排序
                        parent_id: x.parent_id, //父ID
                        menu_url: x.menu_url,   //路由路径
                        api_url: x.api_url,     //接口URL
                        menu_icon: x.menu_icon, //菜单图标
                        remark: x.remark.unwrap_or_default(), //备注
                        create_time: x.create_time.to_string(), //创建时间
                        update_time: x.update_time.to_string(), //修改时间
                    };

                    BaseResponse::<QueryMenuDetailResp>::ok_result_data(data)
                }
                Err(err) => {
                    error!("err:{}", err.to_string());
                    BaseResponse::<QueryMenuDetailResp>::err_result_data(
                        QueryMenuDetailResp::new(),
                        err.to_string(),
                    )
                }
            }
        }

        Err(err) => {
            error!("err:{}", err.to_string());
            BaseResponse::<QueryMenuDetailResp>::err_result_data(
                QueryMenuDetailResp::new(),
                err.to_string(),
            )
        }
    }
}

/*
 *查询菜单信息列表
 *author：刘飞华
 *date：2024/12/20 10:04:30
 */
#[web::post("/menu/queryMenuList")]
pub async fn query_sys_menu_list(item: Json<QueryMenuListReq>) -> impl Responder {
    info!("query sys_menu_list params: {:?}", &item);

    let query = sys_menu::table().into_boxed();

    debug!(
        "SQL:{}",
        diesel::debug_query::<diesel::mysql::Mysql, _>(&query).to_string()
    );

    match &mut RB.clone().get() {
        Ok(conn) => {
            let mut sys_menu_list_data: Vec<MenuListDataResp> = Vec::new();
            if let Ok(list) = query.load::<SysMenu>(conn) {
                for x in list {
                    sys_menu_list_data.push(MenuListDataResp {
                        id: x.id,                               //主键
                        menu_name: x.menu_name,                 //菜单名称
                        menu_type: x.menu_type, //菜单类型(1：目录   2：菜单   3：按钮)
                        status: x.status,       //状态(1:正常，0:禁用)
                        sort: x.sort,           //排序
                        parent_id: x.parent_id, //父ID
                        menu_url: x.menu_url,   //路由路径
                        api_url: x.api_url,     //接口URL
                        menu_icon: x.menu_icon, //菜单图标
                        remark: x.remark.unwrap_or_default(), //备注
                        create_time: x.create_time.to_string(), //创建时间
                        update_time: x.update_time.to_string(), //修改时间
                    })
                }
            }
            let total = 0;
            BaseResponse::<Vec<MenuListDataResp>>::ok_result_page(sys_menu_list_data, total)
        }
        Err(err) => {
            error!("err:{}", err.to_string());
            BaseResponse::err_result_page(MenuListDataResp::new(), err.to_string())
        }
    }
}
