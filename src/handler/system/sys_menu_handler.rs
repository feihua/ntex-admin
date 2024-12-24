use log::info;
use ntex::web;
use ntex::web::types::Json;
use rbs::to_value;

use crate::common::result::BaseResponse;
use crate::model::system::sys_menu_model::Menu;
use crate::vo::system::sys_menu_vo::*;
use crate::RB;

/*
 *添加菜单信息
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/menu/addMenu")]
pub async fn add_sys_menu(item: Json<AddMenuReq>) -> Result<impl web::Responder, web::Error> {
    info!("add sys_menu params: {:?}", &item);

    let req = item.0;

    let sys_menu = Menu {
        id: None,                 //主键
        menu_name: req.menu_name, //菜单名称
        menu_type: req.menu_type, //菜单类型(1：目录   2：菜单   3：按钮)
        status: req.status, //状态(1:正常，0:禁用)
        sort: req.sort,           //排序
        parent_id: req.parent_id.unwrap_or(2), //父ID
        menu_url: req.menu_url,   //路由路径
        api_url: req.api_url,     //接口URL
        menu_icon: req.menu_icon, //菜单图标
        remark: req.remark,       //备注
        create_time: None,        //创建时间
        update_time: None,        //修改时间
    };

    let result = Menu::insert(&mut RB.clone(), &sys_menu).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *删除菜单信息
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/menu/deleteMenu")]
pub async fn delete_sys_menu(item: Json<DeleteMenuReq>) -> Result<impl web::Responder, web::Error> {
    info!("delete sys_menu params: {:?}", &item);

    //有下级的时候 不能直接删除
    let menus = Menu::select_by_column(&mut RB.clone(), "parent_id", &item.id)
        .await
        .unwrap_or_default();

    if menus.len() > 0 {
        return Ok(BaseResponse::<String>::err_result_msg(
            "有下级菜单,不能直接删除".to_string(),
        ));
    }

    let result = Menu::delete_by_column(&mut RB.clone(), "id", &item.id).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *更新菜单信息
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/menu/updateMenu")]
pub async fn update_sys_menu(item: Json<UpdateMenuReq>) -> Result<impl web::Responder, web::Error> {
    info!("update sys_menu params: {:?}", &item);

    let req = item.0;

    let sys_menu = Menu {
        id: Some(req.id),         //主键
        menu_name: req.menu_name, //菜单名称
        menu_type: req.menu_type, //菜单类型(1：目录   2：菜单   3：按钮)
        status: req.status, //状态(1:正常，0:禁用)
        sort: req.sort,           //排序
        parent_id: req.parent_id, //父ID
        menu_url: req.menu_url,   //路由路径
        api_url: req.api_url,     //接口URL
        menu_icon: req.menu_icon, //菜单图标
        remark: req.remark,       //备注
        create_time: None,        //创建时间
        update_time: None,        //修改时间
    };

    let result = Menu::update_by_column(&mut RB.clone(), &sys_menu, "id").await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *更新菜单信息状态
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/menu/updateMenuStatus")]
pub async fn update_sys_menu_status(
    item: Json<UpdateMenuStatusReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("update sys_menu_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let update_sql = format!(
        "update sys_menu set status_id = ? where id in ({})",
        req.ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut param = vec![to_value!(req.status)];
    param.extend(req.ids.iter().map(|&id| to_value!(id)));
    let result = rb.exec(&update_sql, param).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *查询菜单信息详情
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/menu/queryMenuDetail")]
pub async fn query_sys_menu_detail(
    item: Json<QueryMenuDetailReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("query sys_menu_detail params: {:?}", &item);

    let result = Menu::select_by_id(&mut RB.clone(), &item.id).await;

    match result {
        Ok(d) => {
            let x = d.unwrap();

            let sys_menu = QueryMenuDetailResp {
                id: x.id.unwrap(),                          //主键
                menu_name: x.menu_name,                     //菜单名称
                menu_type: x.menu_type,                     //菜单类型(1：目录   2：菜单   3：按钮)
                status: x.status,                     //状态(1:正常，0:禁用)
                sort: x.sort,                               //排序
                parent_id: x.parent_id,                     //父ID
                menu_url: x.menu_url.unwrap_or_default(),   //路由路径
                api_url: x.api_url.unwrap_or_default(),     //接口URL
                menu_icon: x.menu_icon.unwrap_or_default(), //菜单图标
                remark: x.remark.unwrap_or_default(),       //备注
                create_time: x.create_time.unwrap().0.to_string(),     //创建时间
                update_time: x.update_time.unwrap().0.to_string(),     //修改时间
            };

            Ok(BaseResponse::<QueryMenuDetailResp>::ok_result_data(
                sys_menu,
            ))
        }
        Err(err) => Ok(BaseResponse::<QueryMenuDetailResp>::err_result_data(
            QueryMenuDetailResp::new(),
            err.to_string(),
        )),
    }
}

/*
 *查询菜单信息列表
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/menu/queryMenuList")]
pub async fn query_sys_menu_list(
    item: Json<QueryMenuListReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("query sys_menu_list params: {:?}", &item);

    let result = Menu::select_all(&mut RB.clone()).await;

    let mut sys_menu_list_data: Vec<MenuListDataResp> = Vec::new();
    match result {
        Ok(d) => {
            for x in d {
                sys_menu_list_data.push(MenuListDataResp {
                    id: x.id.unwrap(),                                 //主键
                    menu_name: x.menu_name,                            //菜单名称
                    menu_type: x.menu_type, //菜单类型(1：目录   2：菜单   3：按钮)
                    status: x.status, //状态(1:正常，0:禁用)
                    sort: x.sort,           //排序
                    parent_id: x.parent_id, //父ID
                    menu_url: x.menu_url.unwrap_or_default(), //路由路径
                    api_url: x.api_url.unwrap_or_default(), //接口URL
                    menu_icon: x.menu_icon.unwrap_or_default(), //菜单图标
                    remark: x.remark.unwrap_or_default(), //备注
                    create_time: x.create_time.unwrap().0.to_string(), //创建时间
                    update_time: x.update_time.unwrap().0.to_string(), //修改时间
                })
            }

            Ok(BaseResponse::<Vec<MenuListDataResp>>::ok_result_page(
                sys_menu_list_data,
                0,
            ))
        }
        Err(err) => Ok(BaseResponse::<Vec<MenuListDataResp>>::err_result_page(
            MenuListDataResp::new(),
            err.to_string(),
        )),
    }
}
