use log::info;
use ntex::http::header;
use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web;
use ntex::web::HttpResponse;

use crate::utils::error::WhoUnfollowedError;
use crate::utils::jwt_util::JWTToken;
use crate::vo::err_result_msg;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct JwtAuth;

impl<S> Middleware<S> for JwtAuth {
    type Service = JwtAuthMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        JwtAuthMiddleware { service }
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
}

impl<S, Err> Service<web::WebRequest<Err>> for JwtAuthMiddleware<S>
    where
        S: Service<web::WebRequest<Err>, Response=web::WebResponse, Error=web::Error>,
        Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_poll_ready!(service);

    async fn call(&self, req: web::WebRequest<Err>, ctx: ServiceCtx<'_, Self>) -> Result<Self::Response, Self::Error> {
        let path = req.path().to_string();
        info!("Hi from start. You requested path: {}", path);
        let header_value = header::HeaderValue::from_str("").unwrap();
        let authorization = req.headers().get("Authorization").unwrap_or(&header_value).to_str().unwrap().to_string();

        if path.contains("login") {
            return Ok(ctx.clone().call(&self.service, req).await?);
        }

        if authorization.len() <= 0 {
            return Ok(req.into_response(HttpResponse::Ok().json(&err_result_msg("token不能为空".to_string()))));
        }

        let jwt_token = match JWTToken::verify("123", &authorization.replace("Bearer ", "")) {
            Ok(j_token) => j_token,
            Result::Err(err) => {
                let er = match err {
                    WhoUnfollowedError::JwtTokenError(s) => { s }
                    _ => "no math error".to_string()
                };
                log::error!("You requested path: {}, token is err: {}", path, er);
                return Ok(req.into_response(HttpResponse::Ok().json(&err_result_msg(er))));
            }
        };

        return if jwt_token.permissions.contains(&path) {
            Ok(ctx.call(&self.service, req).await?)
        } else {
            log::error!("You has no permissions requested path: {:?}", &path);
            Ok(req.into_response(HttpResponse::Ok().json(&err_result_msg("无权限访问".to_string()))))
        };
    }
}