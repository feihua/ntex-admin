use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_operate_log_model::{clean_operate_log, OperateLog};
use crate::vo::system::sys_operate_log_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbs::value;

/*
 *删除操作日志记录
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/operateLog/deleteOperateLog")]
pub async fn delete_sys_operate_log(item: Json<DeleteOperateLogReq>) -> AppResult<Response> {
    info!("delete sys_operate_log params: {:?}", &item);
    let rb = &mut RB.clone();

    OperateLog::delete_by_map(rb, value! {"id": &item.ids}).await?;

    ok_result()
}

/*
 *清空操作日志记录
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/operateLog/cleanOperateLog")]
pub async fn clean_sys_operate_log() -> AppResult<Response> {
    let rb = &mut RB.clone();

    clean_operate_log(rb).await?;

    ok_result()
}

/*
 *查询操作日志记录详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/operateLog/queryOperateLogDetail")]
pub async fn query_sys_operate_log_detail(item: Json<QueryOperateLogDetailReq>) -> AppResult<Response> {
    info!("query sys_operate_log_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    OperateLog::select_by_id(rb, &item.id).await?;

    match OperateLog::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("操作日志不存在")),
        Some(x) => {
            let data: OperateLogResp = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询操作日志记录列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/operateLog/queryOperateLogList")]
pub async fn query_sys_operate_log_list(item: Json<QueryOperateLogListReq>) -> AppResult<Response> {
    info!("query sys_operate_log_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let title = item.title.as_deref().unwrap_or_default(); //模块标题
    let business_type = item.business_type.unwrap_or(4); //业务类型（0其它 1新增 2修改 3删除）
    let method = item.method.as_deref().unwrap_or_default(); //方法名称
    let request_method = item.request_method.as_deref().unwrap_or_default(); //请求方式
    let operator_type = item.operator_type.unwrap_or(3); //操作类别（0其它 1后台用户 2手机端用户）
    let operate_name = item.operate_name.as_deref().unwrap_or_default(); //操作人员
    let dept_name = item.dept_name.as_deref().unwrap_or_default(); //部门名称
    let operate_url = item.operate_url.as_deref().unwrap_or_default(); //请求URL
    let operate_ip = item.operate_ip.as_deref().unwrap_or_default(); //主机地址
    let status = item.status.unwrap_or(2); //操作状态(0:异常,正常)

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = OperateLog::select_page_by_name(
        rb,
        page,
        title,
        &business_type,
        method,
        request_method,
        &operator_type,
        operate_name,
        dept_name,
        operate_url,
        operate_ip,
        &status,
    )
    .await?;

    let mut list: Vec<OperateLogResp> = Vec::new();

    let total = d.total;

    for x in d.records {
        list.push(x.into())
    }

    ok_result_page(list, total)
}
