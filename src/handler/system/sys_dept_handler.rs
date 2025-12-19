use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data};
use crate::model::system::sys_dept_model::{
    check_dept_exist_user, select_children_dept_by_id, select_dept_count,
    select_normal_children_dept_by_id, Dept,
};
use crate::vo::system::sys_dept_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::rbatis_codegen::ops::AsProxy;
use rbatis::rbdc::DateTime;
use rbs::value;

/*
 *添加部门表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dept/addDept")]
pub async fn add_sys_dept(item: Json<DeptReq>) -> AppResult<Response> {
    info!("add sys_dept params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    let res = Dept::select_by_dept_name(rb, &req.dept_name, req.parent_id).await?;

    if res.is_some() {
        return Err(AppError::BusinessError("部门名称已存在"));
    }

    let ancestors = match Dept::select_by_id(rb, &req.parent_id).await? {
        None => return Err(AppError::BusinessError("上级部门不存在")),
        Some(dept) => {
            if dept.status == 0 {
                return Err(AppError::BusinessError("部门停用，不允许添加"));
            }
            format!("{},{}", dept.ancestors.unwrap_or_default(), req.parent_id)
        }
    };

    req.id = None;
    req.ancestors = Some(ancestors);
    Dept::insert(rb, &Dept::from(req))
        .await
        .map(|_| ok_result())?
}

/*
 *删除部门表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dept/deleteDept")]
pub async fn delete_sys_dept(item: Json<DeleteDeptReq>) -> AppResult<Response> {
    info!("delete sys_dept params: {:?}", &item);
    let rb = &mut RB.clone();

    if select_dept_count(rb, &item.id).await? > 0 {
        return Err(AppError::BusinessError("存在下级部门,不允许删除"));
    }

    if check_dept_exist_user(rb, &item.id).await? > 0 {
        return Err(AppError::BusinessError("部门存在用户,不允许删除"));
    }

    Dept::delete_by_map(rb, value! {"id": &item.id}).await?;
    ok_result()
}

/*
 *更新部门表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dept/updateDept")]
pub async fn update_sys_dept(item: Json<DeptReq>) -> AppResult<Response> {
    info!("update sys_dept params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    let id = req.id;

    if Some(req.parent_id) == id {
        return Err(AppError::BusinessError("上级部门不能是自己"));
    }

    let old_ancestors = match Dept::select_by_id(rb, &id.unwrap_or_default()).await? {
        None => return Err(AppError::BusinessError("部门不存在")),
        Some(dept) => dept.ancestors.unwrap_or_default(),
    };

    let ancestors = match Dept::select_by_id(rb, &req.parent_id).await? {
        None => return Err(AppError::BusinessError("上级部门不存在")),
        Some(dept) => {
            format!("{},{}", dept.ancestors.unwrap_or_default(), &req.parent_id)
        }
    };

    if let Some(x) = Dept::select_by_dept_name(rb, &req.dept_name, req.parent_id).await? {
        if x.id != id {
            return Err(AppError::BusinessError("部门名称已存在"));
        }
    }

    if select_normal_children_dept_by_id(rb, &id.unwrap_or_default()).await? > 0 && req.status == 0
    {
        return Err(AppError::BusinessError("该部门包含未停用的子部门"));
    }

    for mut x in select_children_dept_by_id(rb, &id.unwrap_or_default()).await? {
        x.ancestors = Some(
            x.ancestors
                .unwrap_or_default()
                .replace(old_ancestors.as_str(), ancestors.as_str()),
        );
        Dept::update_by_map(rb, &x, value! {"id": &x.id}).await?;
    }

    if req.status == 1 && ancestors != "0" {
        let ids = ancestors.split(",").map(|s| s.i64()).collect::<Vec<i64>>();

        let update_sql = format!(
            "update sys_dept set status = ? ,update_time = ? where id in ({})",
            ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
        );

        let mut param = vec![value!(req.status), value!(DateTime::now())];
        param.extend(ids.iter().map(|&id| value!(id)));

        rb.exec(&update_sql, param).await?;
    }
    req.ancestors = Some(ancestors.clone());

    let data = Dept::from(req);

    Dept::update_by_map(rb, &data, value! {"id":  &id})
        .await
        .map(|_| ok_result())?
}

/*
 *更新部门表状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dept/updateDeptStatus")]
pub async fn update_sys_dept_status(item: Json<UpdateDeptStatusReq>) -> AppResult<Response> {
    info!("update sys_dept_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    if req.status == 1 {
        for id in req.ids.clone() {
            if let Some(x) = Dept::select_by_id(rb, &id).await? {
                let ancestors = x.ancestors;
                let ids = ancestors
                    .unwrap_or_default()
                    .split(",")
                    .map(|s| s.i64())
                    .collect::<Vec<i64>>();

                let update_sql = format!(
                    "update sys_dept set status = ? where id in ({})",
                    ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
                );

                let mut param = vec![value!(req.status)];
                param.extend(ids.iter().map(|&id| value!(id)));
                rb.exec(&update_sql, param).await?;
            }
        }
    }
    let update_sql = format!(
        "update sys_dept set status = ? where id in ({})",
        req.ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut param = vec![value!(req.status)];
    param.extend(req.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await.map(|_| ok_result())?
}

/*
 *查询部门详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dept/queryDeptDetail")]
pub async fn query_sys_dept_detail(item: Json<QueryDeptDetailReq>) -> AppResult<Response> {
    info!("query sys_dept_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    Dept::select_by_id(rb, &item.id).await?;

    match Dept::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("部门不存在")),
        Some(x) => {
            let data: DeptResp = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询部门列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dept/queryDeptList")]
pub async fn query_sys_dept_list(item: Json<QueryDeptListReq>) -> AppResult<Response> {
    info!("query sys_dept_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let dept_name = item.dept_name.as_deref().unwrap_or_default(); //部门名称
    let status = item.status.unwrap_or(2); //部状态（0：停用，1:正常）

    let result = Dept::select_page_dept_list(rb, dept_name, status).await?;

    let mut list: Vec<DeptResp> = Vec::new();
    for x in result {
        list.push(x.into())
    }

    ok_result_data(list)
}
