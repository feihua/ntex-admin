use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_dict_data_model::{count_dict_data_by_type, update_dict_data_type};
use crate::model::system::sys_dict_type_model::DictType;
use crate::vo::system::sys_dict_type_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbs::value;

/*
 *添加字典类型
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictType/addDictType")]
pub async fn add_sys_dict_type(item: Json<DictTypeReq>) -> AppResult<Response> {
    info!("add sys_dict_type params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    if DictType::select_by_dict_type(rb, &req.dict_type).await?.is_some() {
        return Err(AppError::BusinessError("字典类型已存在"));
    }

    req.id = None;
    DictType::insert(rb, &DictType::from(req)).await.map(|_| ok_result())?
}

/*
 *删除字典类型
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictType/deleteDictType")]
pub async fn delete_sys_dict_type(item: Json<DeleteDictTypeReq>) -> AppResult<Response> {
    info!("delete sys_dict_type params: {:?}", &item);
    let rb = &mut RB.clone();

    let ids = item.ids.clone();
    for id in ids {
        let p = match DictType::select_by_id(rb, &id).await? {
            None => return Err(AppError::BusinessError("字典类型不存在,不能删除")),
            Some(p) => p,
        };

        if count_dict_data_by_type(rb, &p.dict_type).await? > 0 {
            return Err(AppError::BusinessError("已分配,不能删除"));
        }
    }

    DictType::delete_by_map(rb, value! {"id": &item.ids}).await.map(|_| ok_result())?
}

/*
 *更新字典类型
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictType/updateDictType")]
pub async fn update_sys_dict_type(item: Json<DictTypeReq>) -> AppResult<Response> {
    info!("update sys_dict_type params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;

    let id = req.id;

    if DictType::select_by_id(rb, &id.unwrap_or_default()).await?.is_none() {
        return Err(AppError::BusinessError("字典类型不存在"));
    }

    if let Some(x) = DictType::select_by_dict_type(rb, &req.dict_type).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("字典类型已存在"));
        }

        let dict_type = x.dict_type;
        update_dict_data_type(rb, &*req.dict_type, &dict_type).await?;
    }

    DictType::update_by_map(rb, &DictType::from(req), value! {"id": &id}).await.map(|_| ok_result())?
}

/*
 *更新字典类型状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictType/updateDictTypeStatus")]
pub async fn update_sys_dict_type_status(item: Json<UpdateDictTypeStatusReq>) -> AppResult<Response> {
    info!("update sys_dict_type_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let update_sql = format!("update sys_dict_type set status = ? where id in ({})", req.ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", "));

    let mut param = vec![value!(req.status)];
    param.extend(req.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await.map(|_| ok_result())?
}

/*
 *查询字典类型详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictType/queryDictTypeDetail")]
pub async fn query_sys_dict_type_detail(item: Json<QueryDictTypeDetailReq>) -> AppResult<Response> {
    info!("query sys_dict_type_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    match DictType::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("字典类型不存在")),
        Some(x) => {
            let data: DictTypeResp = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询字典类型列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictType/queryDictTypeList")]
pub async fn query_sys_dict_type_list(item: Json<QueryDictTypeListReq>) -> AppResult<Response> {
    info!("query sys_dict_type_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let dict_name = item.dict_name.as_deref().unwrap_or_default(); //字典名称
    let dict_type = item.dict_type.as_deref().unwrap_or_default(); //字典类型
    let status = item.status.unwrap_or(2); //状态（0：停用，1:正常）

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = DictType::select_dict_type_list(rb, page, dict_name, dict_type, status).await?;

    let mut list: Vec<DictTypeResp> = Vec::new();

    let total = d.total;

    for x in d.records {
        list.push(x.into())
    }

    ok_result_page(list, total)
}
