use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_dict_data_model::DictData;
use crate::vo::system::sys_dict_data_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbs::value;

/*
 *添加字典数据
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictData/addDictData")]
pub async fn add_sys_dict_data(item: Json<DictDataReq>) -> AppResult<Response> {
    info!("add sys_dict_data params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    if DictData::select_by_dict_label(rb, &req.dict_type, &req.dict_label).await?.is_some() {
        return Err(AppError::BusinessError("字典标签已存在"));
    }

    if DictData::select_by_dict_value(rb, &req.dict_type, &req.dict_value).await?.is_some() {
        return Err(AppError::BusinessError("字典键值已存在"));
    }

    req.id = None;
    DictData::insert(rb, &DictData::from(req)).await.map(|_| ok_result())?
}

/*
 *删除字典数据
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictData/deleteDictData")]
pub async fn delete_sys_dict_data(item: Json<DeleteDictDataReq>) -> AppResult<Response> {
    info!("delete sys_dict_data params: {:?}", &item);
    let rb = &mut RB.clone();

    DictData::delete_by_map(rb, value! {"id": &item.ids}).await.map(|_| ok_result())?
}

/*
 *更新字典数据
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictData/updateDictData")]
pub async fn update_sys_dict_data(item: Json<DictDataReq>) -> AppResult<Response> {
    info!("update sys_dict_data params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;

    let id = req.id;

    if id.is_none() {
        return Err(AppError::BusinessError("主键不能为空"));
    }

    if DictData::select_by_id(rb, &id.unwrap_or_default()).await?.is_none() {
        return Err(AppError::BusinessError("字典数据不存在"));
    }

    if let Some(x) = DictData::select_by_dict_label(rb, &req.dict_type, &req.dict_label).await? {
        if x.id != id {
            return Err(AppError::BusinessError("字典标签已存在"));
        }
    }

    if let Some(x) = DictData::select_by_dict_value(rb, &req.dict_type, &req.dict_value).await? {
        if x.id != id {
            return Err(AppError::BusinessError("字典键值已存在"));
        }
    }

    DictData::update_by_map(rb, &DictData::from(req), value! {"id": &id}).await.map(|_| ok_result())?
}

/*
 *更新字典数据状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictData/updateDictDataStatus")]
pub async fn update_sys_dict_data_status(item: Json<UpdateDictDataStatusReq>) -> AppResult<Response> {
    info!("update sys_dict_data_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let update_sql = format!("update sys_dict_data set status = ? where id in ({})", req.ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", "));

    let mut param = vec![value!(req.status)];
    param.extend(req.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await?;
    ok_result()
}

/*
 *查询字典数据详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictData/queryDictDataDetail")]
pub async fn query_sys_dict_data_detail(item: Json<QueryDictDataDetailReq>) -> AppResult<Response> {
    info!("query sys_dict_data_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    match DictData::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("字典数据不存在")),
        Some(x) => {
            let data: DictDataResp = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询字典数据列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/dictData/queryDictDataList")]
pub async fn query_sys_dict_data_list(item: Json<QueryDictDataListReq>) -> AppResult<Response> {
    info!("query sys_dict_data_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let dict_label = item.dict_label.as_deref().unwrap_or_default(); //字典标签
    let dict_type = item.dict_type.as_deref().unwrap_or_default(); //字典类型
    let status = item.status.unwrap_or(2); //状态（0：停用，1:正常）

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = DictData::select_dict_data_list(rb, page, dict_label, dict_type, status).await?;

    let mut list: Vec<DictDataResp> = Vec::new();

    let total = d.total;

    for x in d.records {
        list.push(x.into())
    }

    ok_result_page(list, total)
}
