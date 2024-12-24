use log::info;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbatis::rbdc::datetime::DateTime;
use rbs::to_value;

use crate::common::result::BaseResponse;
use crate::model::system::sys_menu_model::Menu;
use crate::model::system::sys_role_menu_model::{query_menu_by_role, RoleMenu};
use crate::model::system::sys_role_model::Role;
use crate::model::system::sys_user_role_model::UserRole;
use crate::vo::system::sys_role_vo::*;
use crate::RB;

/*
 *添加角色信息
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/addRole")]
pub async fn add_sys_role(item: Json<AddRoleReq>) -> Result<impl web::Responder, web::Error> {
    info!("add sys_role params: {:?}", &item);

    let req = item.0;

    let sys_role = Role {
        id: None,                 //主键
        role_name: req.role_name, //名称
        status_id: req.status_id, //状态(1:正常，0:禁用)
        sort: req.sort,           //排序
        remark: req.remark,       //备注
        create_time: None,        //创建时间
        update_time: None,        //修改时间
    };

    let result = Role::insert(&mut RB.clone(), &sys_role).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *删除角色信息
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/deleteRole")]
pub async fn delete_sys_role(item: Json<DeleteRoleReq>) -> Result<impl web::Responder, web::Error> {
    info!("delete sys_role params: {:?}", &item);

    let ids = item.ids.clone();
    let user_role_list = UserRole::select_in_column(&mut RB.clone(), "role_id", &ids)
        .await
        .unwrap_or_default();

    if user_role_list.len() > 0 {
        return Ok(BaseResponse::<String>::err_result_msg(
            "角色已被使用,不能直接删除".to_string(),
        ));
    }

    let result = Role::delete_in_column(&mut RB.clone(), "id", &item.ids).await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *更新角色信息
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/updateRole")]
pub async fn update_sys_role(item: Json<UpdateRoleReq>) -> Result<impl web::Responder, web::Error> {
    info!("update sys_role params: {:?}", &item);

    let req = item.0;

    let sys_role = Role {
        id: Some(req.id),         //主键
        role_name: req.role_name, //名称
        status_id: req.status_id, //状态(1:正常，0:禁用)
        sort: req.sort,           //排序
        remark: req.remark,       //备注
        create_time: None,        //创建时间
        update_time: None,        //修改时间
    };

    let result = Role::update_by_column(&mut RB.clone(), &sys_role, "id").await;

    match result {
        Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}

/*
 *更新角色信息状态
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/updateRoleStatus")]
pub async fn update_sys_role_status(
    item: Json<UpdateRoleStatusReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("update sys_role_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let update_sql = format!(
        "update sys_role set status_id = ? where id in ({})",
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
 *查询角色信息详情
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/queryRoleDetail")]
pub async fn query_sys_role_detail(
    item: Json<QueryRoleDetailReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("query sys_role_detail params: {:?}", &item);

    let result = Role::select_by_id(&mut RB.clone(), &item.id).await;

    match result {
        Ok(d) => {
            let x = d.unwrap();

            let sys_role = QueryRoleDetailResp {
                id: x.id.unwrap(),                                 //主键
                role_name: x.role_name,                            //名称
                status_id: x.status_id,                            //状态(1:正常，0:禁用)
                sort: x.sort,                                      //排序
                remark: x.remark,                                  //备注
                create_time: x.create_time.unwrap().0.to_string(), //创建时间
                update_time: x.update_time.unwrap().0.to_string(), //修改时间
            };

            Ok(BaseResponse::<QueryRoleDetailResp>::ok_result_data(
                sys_role,
            ))
        }
        Err(err) => Ok(BaseResponse::<String>::ok_result_code(1, err.to_string())),
    }
}

/*
 *查询角色信息列表
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/queryRoleList")]
pub async fn query_sys_role_list(
    item: Json<QueryRoleListReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("query sys_role_list params: {:?}", &item);

    let role_name = item.role_name.clone().unwrap_or_default();
    let status_id = item.status_id.clone().unwrap_or(2);

    let page = &PageRequest::new(item.page_no.clone(), item.page_size.clone());
    let result = Role::select_page_by_name(&mut RB.clone(), page, &role_name, status_id).await;

    let mut sys_role_list_data: Vec<RoleListDataResp> = Vec::new();
    match result {
        Ok(d) => {
            let total = d.total;

            for x in d.records {
                sys_role_list_data.push(RoleListDataResp {
                    id: x.id.unwrap(),                                 //主键
                    role_name: x.role_name,                            //名称
                    status_id: x.status_id,                            //状态(1:正常，0:禁用)
                    sort: x.sort,                                      //排序
                    remark: x.remark,                                  //备注
                    create_time: x.create_time.unwrap().0.to_string(), //创建时间
                    update_time: x.update_time.unwrap().0.to_string(), //修改时间
                })
            }

            Ok(BaseResponse::<Vec<RoleListDataResp>>::ok_result_page(
                sys_role_list_data,
                total,
            ))
        }
        Err(err) => Ok(BaseResponse::<Vec<RoleListDataResp>>::err_result_page(
            RoleListDataResp::new(),
            err.to_string(),
        )),
    }
}

/*
 *查询角色关联的菜单
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/queryRoleMenu")]
pub async fn query_role_menu(
    item: Json<QueryRoleMenuReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("query role_menu params: {:?}", &item);

    // 查询所有菜单
    let menu_list_all = Menu::select_all(&mut RB.clone()).await.unwrap_or_default();

    let mut menu_data_list: Vec<MenuDataList> = Vec::new();
    let mut role_menu_ids: Vec<i64> = Vec::new();

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

    Ok(BaseResponse::<QueryRoleMenuData>::ok_result_data(
        QueryRoleMenuData {
            menu_ids: role_menu_ids,
            menu_list: menu_data_list,
        },
    ))
}

/*
 *更新角色关联的菜单
 *author：刘飞华
 *date：2024/12/16 14:19:49
 */
#[web::post("/role/updateRoleMenu")]
pub async fn update_role_menu(
    item: Json<UpdateRoleMenuReq>,
) -> Result<impl web::Responder, web::Error> {
    info!("update role_menu params: {:?}", &item);
    let role_id = item.role_id.clone();

    let role_menu_result = RoleMenu::delete_by_column(&mut RB.clone(), "role_id", &role_id).await;

    match role_menu_result {
        Ok(_) => {
            let mut menu_role: Vec<RoleMenu> = Vec::new();

            for id in &item.menu_ids {
                let menu_id = id.clone();
                menu_role.push(RoleMenu {
                    id: None,
                    create_time: Some(DateTime::now()),
                    menu_id,
                    role_id: role_id.clone(),
                })
            }

            let result =
                RoleMenu::insert_batch(&mut RB.clone(), &menu_role, item.menu_ids.len() as u64)
                    .await;

            match result {
                Ok(_u) => Ok(BaseResponse::<String>::ok_result()),
                Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
            }
        }
        Err(err) => Ok(BaseResponse::<String>::err_result_msg(err.to_string())),
    }
}
