use crate::common::result::{BaseResponse};
use crate::model::system::menu::SysMenu;
use crate::model::system::role::SysRole;
use crate::model::system::role_menu::{query_menu_by_role, SysRoleMenu};
use crate::model::system::user_role::SysUserRole;
use crate::vo::system::role_vo::*;
use crate::RB;
use log::info;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbatis::rbdc::datetime::DateTime;


// 添加角色信息
#[web::post("/role_save")]
pub async fn role_save(item: Json<RoleSaveReq>) -> Result<impl web::Responder, web::Error> {
    info!("role_save params: {:?}", &item);

    let role = item.0;

    let sys_role = SysRole {
        id: None,
        create_time: Some(DateTime::now()),
        update_time: Some(DateTime::now()),
        status_id: role.status_id,
        sort: role.sort,
        role_name: role.role_name,
        remark: role.remark,
    };

    let result = SysRole::insert(&mut RB.clone(), &sys_role).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

// 删除角色信息
#[web::post("/role_delete")]
pub async fn role_delete(item: Json<RoleDeleteReq>) -> Result<impl web::Responder, web::Error> {
    info!("role_delete params: {:?}", &item);

    let ids = item.ids.clone();
    let user_role_list = SysUserRole::select_in_column(&mut RB.clone(), "role_id", &ids)
        .await
        .unwrap_or_default();

    if user_role_list.len() > 0 {
        return Ok(BaseResponse::<String>::err_result_msg("角色已被使用,不能直接删除".to_string()));
    }

    let result = SysRole::delete_in_column(&mut RB.clone(), "id", &item.ids).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

// 更新角色信息
#[web::post("/role_update")]
pub async fn role_update(item: Json<RoleUpdateReq>) -> Result<impl web::Responder, web::Error> {
    info!("role_update params: {:?}", &item);

    let role = item.0;

    let sys_role = SysRole {
        id: Some(role.id),
        create_time: None,
        update_time: Some(DateTime::now()),
        status_id: role.status_id,
        sort: role.sort,
        role_name: role.role_name,
        remark: role.remark,
    };

    let result = SysRole::update_by_column(&mut RB.clone(), &sys_role, "id").await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}


// 查询角色列表
#[web::post("/role_list")]
pub async fn role_list(item: Json<RoleListReq>) -> Result<impl web::Responder, web::Error> {
    info!("role_list params: {:?}", &item);

    let role_name = item.role_name.clone().unwrap_or_default();
    let status_id = item.status_id.clone().unwrap_or_default();

    let page_req = &PageRequest::new(item.page_no.clone(), item.page_size.clone());
    let result =
        SysRole::select_page_by_name(&mut RB.clone(), page_req, &role_name, &status_id).await;

    let mut role_list: Vec<RoleListData> = Vec::new();
    match result {
        Ok(page) => {
            let total = page.total;

            for role in page.records {
                role_list.push(RoleListData {
                    id: role.id.unwrap(),
                    sort: role.sort,
                    status_id: role.status_id,
                    role_name: role.role_name,
                    remark: role.remark.unwrap_or_default(),
                    create_time: role.create_time.unwrap().0.to_string(),
                    update_time: role.update_time.unwrap().0.to_string(),
                })
            }
            Ok(BaseResponse::<Vec<RoleListData>>::ok_result_page(role_list, total))
        }
        Err(err) => Ok(BaseResponse::<Vec<RoleListData>>::err_result_page(role_list, err.to_string())),
    }
}

// 查询角色关联的菜单
#[web::post("/query_role_menu")]
pub async fn query_role_menu(
    item: Json<QueryRoleMenuReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("query_role_menu params: {:?}", &item);

    // 查询所有菜单
    let menu_list_all = SysMenu::select_all(&mut RB.clone())
        .await
        .unwrap_or_default();

    let mut menu_data_list: Vec<MenuDataList> = Vec::new();
    let mut role_menu_ids: Vec<i32> = Vec::new();

    for y in menu_list_all {
        let x = y.clone();
        menu_data_list.push(MenuDataList {
            id: x.id.unwrap(),
            parent_id: x.parent_id,
            title: x.menu_name,
            key: y.id.unwrap().to_string(),
            label: y.menu_name,
            is_penultimate: y.parent_id == 2,
        });
        role_menu_ids.push(x.id.unwrap())
    }

    //不是超级管理员的时候,就要查询角色和菜单的关联
    if item.role_id != 1 {
        role_menu_ids.clear();
        let role_menu_list = query_menu_by_role(&mut RB.clone(), item.role_id.clone())
            .await
            .unwrap_or_default();

        for x in role_menu_list {
            let m_id = x.get("menu_id").unwrap().clone();
            role_menu_ids.push(m_id)
        }
    }

    Ok(BaseResponse::<QueryRoleMenuData>::ok_result_data(QueryRoleMenuData {
        role_menus: role_menu_ids,
        menu_list: menu_data_list,
    }))
}

// 更新角色关联的菜单
#[web::post("/update_role_menu")]
pub async fn update_role_menu(
    item: Json<UpdateRoleMenuReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("update_role_menu params: {:?}", &item);
    let role_id = item.role_id.clone();

    let role_menu_result =
        SysRoleMenu::delete_by_column(&mut RB.clone(), "role_id", &role_id).await;

    match role_menu_result {
        Ok(_) => {
            let mut menu_role: Vec<SysRoleMenu> = Vec::new();

            for id in &item.menu_ids {
                let menu_id = id.clone();
                menu_role.push(SysRoleMenu {
                    id: None,
                    create_time: Some(DateTime::now()),
                    update_time: Some(DateTime::now()),
                    status_id: 1,
                    sort: 1,
                    menu_id,
                    role_id: role_id.clone(),
                })
            }

            let result =
                SysRoleMenu::insert_batch(&mut RB.clone(), &menu_role, item.menu_ids.len() as u64)
                    .await;

            match result {
                Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
                Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
            }
        }
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}
