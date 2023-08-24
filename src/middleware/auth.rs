// use log::info;
// use ntex::http::error::DispatchError::InternalError;
// use ntex::http::header;
// use ntex::service::{Middleware, Service, ServiceCtx};
// use ntex::util::BoxFuture;
// use ntex::web;
// use ntex::web::{error, Error, WebResponse};
// use serde_json::json;
//
// use crate::utils::error::WhoUnfollowedError;
// use crate::utils::jwt_util::JWTToken;
// use crate::vo::err_result_msg;
//
// // There are two steps in middleware processing.
// // 1. Middleware initialization, middleware factory gets called with
// //    next service in chain as parameter.
// // 2. Middleware's call method gets called with normal request.
// pub struct JwtAuth;
//
// impl<S> Middleware<S> for JwtAuth {
//     type Service = JwtAuthMiddleware<S>;
//
//     fn create(&self, service: S) -> Self::Service {
//         JwtAuthMiddleware { service }
//     }
// }
//
// pub struct JwtAuthMiddleware<S> {
//     service: S,
// }
//
// impl<S, Err> Service<web::WebRequest<Err>> for JwtAuthMiddleware<S>
//     where
//         S: Service<web::WebRequest<Err>, Response=web::WebResponse, Error=web::Error>,
//         Err: web::ErrorRenderer,
// {
//     type Response = web::WebResponse;
//     type Error = web::Error;
//     type Future<'f> = BoxFuture<'f, Result<Self::Response, Self::Error>> where Self: 'f;
//
//     ntex::forward_poll_ready!(service);
//
//     fn call<'a>(&'a self, req: web::WebRequest<Err>, ctx: ServiceCtx<'a, Self>) -> Self::Future<'_> {
//         let path = req.path().to_string();
//         info!("Hi from start. You requested path: {}", path);
//
//         let fut = ctx.call(&self.service, req);
//         Box::pin(async move {
//             let header_value = req.headers().get("Authorization").unwrap().to_str();
//             let authorization = match header_value {
//                 Ok(h_value) => {
//                     h_value.to_string().replace("Bearer ", "")
//                 }
//                 Result::Err(err) => {
//                     return web::Error(Self::Response.json(&err_result_msg("token不能为空".to_string())));
//                 }
//             };
//
//             let jwt_token_result = JWTToken::verify("123", &authorization);
//             let jwt_token = match jwt_token_result {
//                 Ok(data) => { data }
//                 Err(err) => {
//                     let er = match err {
//                         WhoUnfollowedError::JwtTokenError(s) => { s }
//                         _ => "no math error".to_string()
//                     };
//                     return web::Error(Self::Response::Ok().json(&err_result_msg("token不能为空".to_string())));
//                 }
//             };
//
//             let mut flag: bool = false;
//             for token_permission in &jwt_token.permissions {
//                 if token_permission.to_string() == path {
//                     flag = true;
//                     break;
//                 }
//             }
//             return if flag {
//                 let res = fut.await;
//                 // Ok(res)
//                 match res {
//                     Ok(r) => {
//                         Ok(r)
//                     }
//                     Result::Err(err) => {
//                         return web::Error(Self::Response::Ok().json(&err_result_msg("无权限访问".to_string())));
//                     }
//                 }
//             } else {
//                 log::error!("Hi from start. You requested path: {:?}", jwt_token.permissions);
//
//                 return web::Error(Self::Response::Ok().json(&err_result_msg("无权限访问".to_string())));
//             };
//         })
//     }
// }