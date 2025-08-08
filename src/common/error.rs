use crate::common::result::BaseResponse;
use ntex::{http, web};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    // #[error("Failed to complete an HTTP request")]
    // Http { #[from] source: reqwest::Error },
    //
    #[error("Failed to read the cache file")]
    DiskCacheRead { source: std::io::Error },
    //
    // #[error("Failed to update the cache file")]
    // DiskCacheWrite { source: std::io::Error },
    #[error("")]
    JwtTokenError(String),

    #[error("数据库错误: {0}")]
    DbError(#[from] rbatis::Error),

    #[error("业务异常: {0}")]
    BusinessError(&'static str),
}

pub type AppResult<T> = Result<T, AppError>;
impl web::error::WebResponseError for AppError {
    fn status_code(&self) -> http::StatusCode {
        http::StatusCode::OK
    }

    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        web::HttpResponse::Ok().json(&BaseResponse::<String> {
            msg: self.to_string(),
            code: 1,
            data: Some("None".to_string()),
        })
    }
}
