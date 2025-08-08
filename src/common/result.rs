use crate::common::error::AppResult;
use ntex::http::Response;
use ntex::web;
use serde::Serialize;
use std::fmt::Debug;

// 统一返回vo
#[derive(Serialize, Debug, Clone)]
pub struct BaseResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponsePage<T> {
    pub code: i32,
    pub msg: String,
    pub total: u64,
    pub success: bool,
    pub data: Option<T>,
}

pub fn ok_result() -> AppResult<Response> {
    ok_result_msg("操作成功")
}

pub fn ok_result_msg(msg: &str) -> AppResult<Response> {
    let x = &BaseResponse::<String> {
        msg: msg.to_string(),
        code: 0,
        data: Some("None".to_string()),
    };
    Ok(web::HttpResponse::Ok().json(x))
}

pub fn ok_result_data<T>(data: T) -> AppResult<Response>
where
    T: Serialize,
{
    let x = &BaseResponse {
        msg: "操作成功".to_string(),
        code: 0,
        data: Some(data),
    };
    Ok(web::HttpResponse::Ok().json(x))
}

pub fn err_result_msg(msg: &str) -> AppResult<Response> {
    let x = &BaseResponse::<String> {
        msg: msg.to_string(),
        code: 1,
        data: Some("None".to_string()),
    };
    Ok(web::HttpResponse::Ok().json(x))
}

pub fn ok_result_page<T>(data: T, total: u64) -> AppResult<Response>
where
    T: Serialize,
{
    let page = &ResponsePage {
        msg: "操作成功".to_string(),
        code: 0,
        success: true,
        data: Some(data),
        total,
    };
    Ok(web::HttpResponse::Ok().json(page))
}
