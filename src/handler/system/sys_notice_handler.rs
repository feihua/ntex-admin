use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_notice_model::Notice;
use crate::vo::system::sys_notice_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbs::value;

/*
 *添加通知公告
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/notice/addNotice")]
pub async fn add_sys_notice(item: Json<NoticeReq>) -> AppResult<Response> {
    info!("add sys_notice params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    let res = Notice::select_by_title(rb, &req.notice_title).await?;
    if res.is_some() {
        return Err(AppError::BusinessError("公告标题已存在"));
    }

    req.id = None;
    Notice::insert(rb, &Notice::from(req))
        .await
        .map(|_| ok_result())?
}

/*
 *删除通知公告
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/notice/deleteNotice")]
pub async fn delete_sys_notice(item: Json<DeleteNoticeReq>) -> AppResult<Response> {
    info!("delete sys_notice params: {:?}", &item);
    let rb = &mut RB.clone();

    Notice::delete_by_map(rb, value! {"id": &item.ids}).await?;

    ok_result()
}

/*
 *更新通知公告
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/notice/updateNotice")]
pub async fn update_sys_notice(item: Json<NoticeReq>) -> AppResult<Response> {
    info!("update sys_notice params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;

    let id = req.id;
    let result = Notice::select_by_id(rb, &id.unwrap_or_default()).await?;

    if result.is_none() {
        return Err(AppError::BusinessError("通知公告不存在"));
    }

    let res = Notice::select_by_title(rb, &req.notice_title).await?;

    if let Some(x) = res {
        if x.id != id {
            return Err(AppError::BusinessError("公告标题已存在"));
        }
    }

    Notice::update_by_map(rb, &Notice::from(req), value! {"id": &id})
        .await
        .map(|_| ok_result())?
}

/*
 *更新通知公告状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/notice/updateNoticeStatus")]
pub async fn update_sys_notice_status(item: Json<UpdateNoticeStatusReq>) -> AppResult<Response> {
    info!("update sys_notice_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let update_sql = format!(
        "update sys_notice set status = ? where id in ({})",
        req.ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut param = vec![value!(req.status)];
    param.extend(req.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await?;
    ok_result()
}

/*
 *查询通知公告详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/notice/queryNoticeDetail")]
pub async fn query_sys_notice_detail(item: Json<QueryNoticeDetailReq>) -> AppResult<Response> {
    info!("query sys_notice_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    match Notice::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("通知公告不存在")),
        Some(x) => {
            let data: NoticeResp = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询通知公告列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/notice/queryNoticeList")]
pub async fn query_sys_notice_list(item: Json<QueryNoticeListReq>) -> AppResult<Response> {
    info!("query sys_notice_list params: {:?}", &item);
    let rb = &mut RB.clone();
    let notice_title = item.notice_title.as_deref().unwrap_or_default();
    let notice_type = item.notice_type.unwrap_or(0); //公告类型（1:通知,2:公告）
    let status = item.status.unwrap_or(2); //公告状态（0:关闭,1:正常 ）

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = Notice::select_sys_notice_list(rb, page, notice_title, notice_type, status).await?;

    let mut data: Vec<NoticeResp> = Vec::new();
    let total = d.total;

    for x in d.records {
        data.push(x.into())
    }

    ok_result_page(data, total)
}
