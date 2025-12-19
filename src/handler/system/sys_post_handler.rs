use crate::common::error::{AppError, AppResult};
use crate::common::result::{ok_result, ok_result_data, ok_result_page};
use crate::model::system::sys_post_model::Post;
use crate::model::system::sys_user_post_model::count_user_post_by_id;
use crate::vo::system::sys_post_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbs::value;

/*
 *添加岗位信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/post/addPost")]
pub async fn add_sys_post(item: Json<PostReq>) -> AppResult<Response> {
    info!("add sys_post params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    if Post::select_by_name(rb, &req.post_name).await?.is_some() {
        return Err(AppError::BusinessError("岗位名称已存在"));
    }

    if Post::select_by_code(rb, &req.post_code).await?.is_some() {
        return Err(AppError::BusinessError("岗位编码已存在"));
    }

    req.id = None;
    Post::insert(rb, &Post::from(req)).await.map(|_| ok_result())?
}

/*
 *删除岗位信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/post/deletePost")]
pub async fn delete_sys_post(item: Json<DeletePostReq>) -> AppResult<Response> {
    info!("delete sys_post params: {:?}", &item);
    let rb = &mut RB.clone();

    let ids = item.ids.clone();
    for id in ids {
        let post_by_id = Post::select_by_id(rb, &id).await?;
        let _ = match post_by_id {
            None => {
                return Err(AppError::BusinessError("岗位不存在,不能删除"));
            }
            Some(p) => p,
        };

        if count_user_post_by_id(rb, id).await? > 0 {
            return Err(AppError::BusinessError("已分配,不能删除"));
        }
    }

    Post::delete_by_map(rb, value! {"id": &item.ids}).await?;

    ok_result()
}

/*
 *更新岗位信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/post/updatePost")]
pub async fn update_sys_post(item: Json<PostReq>) -> AppResult<Response> {
    info!("update sys_post params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;

    let id = req.id;
    if Post::select_by_id(rb, &id.unwrap_or_default()).await?.is_none() {
        return Err(AppError::BusinessError("岗位不存在"));
    }

    if let Some(x) = Post::select_by_name(rb, &req.post_name).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("岗位名称已存在"));
        }
    }

    if let Some(x) = Post::select_by_code(rb, &req.post_code).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("岗位编码已存在"));
        }
    }

    Post::update_by_map(rb, &Post::from(req), value! {"id": &id}).await.map(|_| ok_result())?
}

/*
 *更新岗位信息状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/post/updatePostStatus")]
pub async fn update_sys_post_status(item: Json<UpdatePostStatusReq>) -> AppResult<Response> {
    info!("update sys_post_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let update_sql = format!("update sys_post set status = ? where id in ({})", item.ids.iter().map(|_| "?").collect::<Vec<&str>>().join(", "));

    let mut param = vec![value!(item.status)];
    param.extend(item.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await?;

    ok_result()
}

/*
 *查询岗位信息详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/post/queryPostDetail")]
pub async fn query_sys_post_detail(item: Json<QueryPostDetailReq>) -> AppResult<Response> {
    info!("query sys_post_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    match Post::select_by_id(rb, &item.id).await? {
        None => Err(AppError::BusinessError("岗位不存在")),
        Some(x) => {
            let data: Post = x.into();
            ok_result_data(data)
        }
    }
}

/*
 *查询岗位信息列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/post/queryPostList")]
pub async fn query_sys_post_list(item: Json<QueryPostListReq>) -> AppResult<Response> {
    info!("query sys_post_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let post_code = item.post_code.as_deref().unwrap_or_default(); //岗位编码
    let post_name = item.post_name.as_deref().unwrap_or_default(); //岗位名称
    let status = item.status.unwrap_or(2); //部状态（0：停用，1:正常）

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = Post::select_post_list(rb, page, post_code, post_name, status).await?;

    let mut list: Vec<PostResp> = Vec::new();

    let total = d.total;

    for x in d.records {
        list.push(x.into())
    }

    ok_result_page(list, total)
}
