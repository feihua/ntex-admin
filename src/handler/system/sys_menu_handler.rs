use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_menu_model::{select_count_menu_by_parent_id, Menu};
use crate::model::system::sys_role_menu_model::select_count_menu_by_menu_id;
use crate::vo::system::sys_menu_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::PageRequest;
use rbs::value;

/*
 *添加菜单信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/addMenu")]
pub async fn add_sys_menu(item: Json<MenuReq>) -> AppResult<Response> {
    info!("add sys_menu params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    let option = Menu::select_by_menu_name(rb, &req.menu_name).await?;
    if option.is_some() {
        return Err(AppError::BusinessError("菜单名称已存在"));
    }

    if let Some(url) = req.menu_url.clone() {
        if url != "".to_string() {
            if Menu::select_by_menu_url(rb, &url).await?.is_some() {
                return Err(AppError::BusinessError("路由路径已存在"));
            }
        }
    }

    req.id = None;
    Menu::insert(rb, &Menu::from(req))
        .await
        .map(|_| ok_result())?
}

/*
 *删除菜单信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/deleteMenu")]
pub async fn delete_sys_menu(item: Json<DeleteMenuReq>) -> AppResult<Response> {
    info!("delete sys_menu params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;
    let ids = req.ids;
    for x in ids.clone() {
        if select_count_menu_by_parent_id(rb, &x).await? > 0 {
            return Err(AppError::BusinessError("存在子菜单,不允许删除"));
        }

        if select_count_menu_by_menu_id(rb, &x).await? > 0 {
            return Err(AppError::BusinessError("菜单已分配,不允许删除"));
        }
    }

    Menu::delete_by_map(rb, value! {"id": &ids})
        .await
        .map(|_| ok_result())?
}

/*
 *更新菜单信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/updateMenu")]
pub async fn update_sys_menu(item: Json<MenuReq>) -> AppResult<Response> {
    info!("update sys_menu params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;

    let id = req.id;

    if id.is_none() {
        return Err(AppError::BusinessError("主键不能为空"));
    }

    if Menu::select_by_id(rb, &req.id.unwrap_or_default())
        .await?
        .is_none()
    {
        return Err(AppError::BusinessError("菜单信息不存在"));
    }

    if let Some(x) = Menu::select_by_menu_name(rb, &req.menu_name).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("菜单名称已存在"));
        }
    }

    if let Some(url) = req.menu_url.clone() {
        if url != "".to_string() {
            if let Some(x) = Menu::select_by_menu_url(rb, &url).await? {
                if x.id != id {
                    return Err(AppError::BusinessError("路由路径已存在"));
                }
            }
        }
    }

    Menu::update_by_map(rb, &Menu::from(req), value! {"id": &id})
        .await
        .map(|_| ok_result())?
}

/*
 *更新菜单信息状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/updateMenuStatus")]
pub async fn update_sys_menu_status(item: Json<UpdateMenuStatusReq>) -> AppResult<Response> {
    info!("update sys_menu_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let ids = req
        .ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<&str>>()
        .join(", ");
    let update_sql = format!("update sys_menu set status = ? where id in ({})", ids);

    let mut param = vec![value!(req.status)];
    param.extend(req.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await?;

    ok_result()
}

/*
 *查询菜单信息详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/queryMenuDetail")]
pub async fn query_sys_menu_detail(item: Json<QueryMenuDetailReq>) -> AppResult<Response> {
    info!("query sys_menu_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    Menu::select_by_id(rb, &item.id).await?.map_or_else(
        || Err(AppError::BusinessError("菜单信息不存在")),
        |x| {
            let data: MenuResp = x.into();
            ok_result_data(data)
        },
    )
}

/*
 *查询菜单信息列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/queryMenuList")]
pub async fn query_sys_menu_list(item: Json<QueryMenuListReq>) -> AppResult<Response> {
    info!("query sys_menu_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;
    let menu_name = req.menu_name;
    let parent_id = req.parent_id;
    let status = req.status;

    Menu::query_sys_menu_list(rb, menu_name, parent_id, status)
        .await
        .map(|x| ok_result_data(x.into_iter().map(|x| x.into()).collect::<Vec<MenuResp>>()))?
}

/*
 *查询菜单信息(排除按钮)
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/menu/queryMenuListSimple")]
pub async fn query_sys_menu_list_simple() -> AppResult<Response> {
    let rb = &mut RB.clone();

    let list = Menu::select_menu_list(rb).await?;

    let mut menu_list: Vec<MenuListSimpleDataResp> = Vec::new();
    for x in list {
        menu_list.push(MenuListSimpleDataResp {
            id: x.id,               //主键
            menu_name: x.menu_name, //菜单名称
            parent_id: x.parent_id, //父ID
        })
    }

    ok_result_data(menu_list)
}

#[web::post("/menu/queryMenuResourceList")]
pub async fn query_sys_menu_resource_list(item: Json<QueryMenuListReq>) -> AppResult<Response> {
    info!("query sys_menu_list params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;
    let menu_name = req.menu_name;
    let parent_id = req.parent_id;
    let status = req.status;

    let page = &PageRequest::new(
        req.page_no.unwrap_or_default(),
        req.page_size.unwrap_or_default(),
    );

    Menu::query_sys_menu_resource_list(rb, page, menu_name, parent_id, status)
        .await
        .map(|x| {
            ok_result_page(
                x.records
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<MenuResp>>(),
                x.total,
            )
        })?
}
