use ntex::web;
use serde::Serialize;
use std::fmt::Debug;
use ntex::http::Response;

// 统一返回vo
#[derive(Serialize, Debug, Clone)]
pub struct BaseResponse<T>
where
    T: Serialize + Debug,
{
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ResponsePage<T>
where
    T: Serialize + Debug,
{
    pub code: i32,
    pub msg: String,
    pub total: u64,
    pub success: bool,
    pub data: Option<T>,
}


impl<T> BaseResponse<T>
where
    T: Serialize + Debug + Send,
{
    pub fn ok_result() -> Response {
        web::HttpResponse::Ok().json(&BaseResponse::<String> {
            msg: "操作成功".to_string(),
            code: 0,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_msg(msg: String) -> Response {
        web::HttpResponse::Ok().json(&BaseResponse::<String> {
            msg: msg.to_string(),
            code: 0,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_code(code: i32, msg: String) -> Response {
        web::HttpResponse::Ok().json(&BaseResponse::<String> {
            msg: msg.to_string(),
            code,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_data(data: T) -> Response {
        web::HttpResponse::Ok().json(&BaseResponse::<T> {
            msg: "操作成功".to_string(),
            code: 0,
            data: Some(data),
        })
    }

    pub fn err_result_msg(msg: String) -> Response {
        web::HttpResponse::Ok().json(&BaseResponse::<String> {
            msg: msg.to_string(),
            code: 1,
            data: Some("None".to_string()),
        })
    }

    pub fn err_result_code(code: i32, msg: String) -> Response {
        web::HttpResponse::Ok().json(&BaseResponse::<String> {
            msg: msg.to_string(),
            code,
            data: Some("None".to_string()),
        })
    }

    pub fn ok_result_page(data: T, total: u64)->Response  {
        web::HttpResponse::Ok().json(&ResponsePage {
            msg: "操作成功".to_string(),
            code: 0,
            success: true,
            data: Some(data),
            total,
        })
    }

    pub fn err_result_page(data: T, msg: String) ->Response {
        web::HttpResponse::Ok().json(&ResponsePage {
            msg: msg.to_string(),
            code: 1,
            success: false,
            data: Some(data),
            total: 0,
        })
    }
}
