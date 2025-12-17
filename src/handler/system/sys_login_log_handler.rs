use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_login_log_model::{clean_login_log, LoginLog};
use crate::vo::system::sys_login_log_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbs::value;

/*
 *删除系统访问记录
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/loginLog/deleteLoginLog")]
pub async fn delete_sys_login_log(item: Json<DeleteLoginLogReq>) -> AppResult<Response> {
    info!("delete sys_login_log params: {:?}", &item);
    let rb = &mut RB.clone();

    LoginLog::delete_by_map(rb, value! {"id": &item.ids}).await?;

    ok_result()
}

/*
 *清空系统访问记录
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/loginLog/cleanLoginLog")]
pub async fn clean_sys_login_log(item: Json<DeleteLoginLogReq>) -> AppResult<Response> {
    info!("clean sys_login_log params: {:?}", &item);
    let rb = &mut RB.clone();

    clean_login_log(rb).await?;

    ok_result()
}

/*
 *查询系统访问记录详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/loginLog/queryLoginLogDetail")]
pub async fn query_sys_login_log_detail(item: Json<QueryLoginLogDetailReq>) -> AppResult<Response> {
    info!("query sys_login_log_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    LoginLog::select_by_id(rb, &item.id).await?;

    match LoginLog::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("日志不存在")),
        Some(x) => {
            let data: LoginLogResp = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询系统访问记录列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/loginLog/queryLoginLogList")]
pub async fn query_sys_login_log_list(item: Json<QueryLoginLogListReq>) -> AppResult<Response> {
    info!("query sys_login_log_list params: {:?}", &item);
    let rb = &mut RB.clone();
    let name = item.login_name.as_deref().unwrap_or_default(); //登录账号
    let ipaddr = item.ipaddr.as_deref().unwrap_or_default(); //登录IP地址
    let browser = item.browser.as_deref().unwrap_or_default(); //浏览器类型
    let os = item.os.as_deref().unwrap_or_default(); //操作系统
    let status = item.status.unwrap_or(2); //登录状态(0:失败,1:成功)

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = LoginLog::select_login_log_list(rb, page, name, ipaddr, browser, os, &status).await?;

    let mut list: Vec<LoginLogResp> = Vec::new();

    let total = d.total;

    for x in d.records {
        list.push(x.into())
    }

    ok_result_page(list, total)
}
