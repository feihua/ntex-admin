use crate::common::error::{AppError, AppResult};
use crate::common::result::{
    err_result_msg, ok_result, ok_result_data, ok_result_page,
};
use crate::model::system::sys_dept_model::Dept;
use crate::model::system::sys_login_log_model::LoginLog;
use crate::model::system::sys_menu_model::Menu;
use crate::model::system::sys_role_model::Role;
use crate::model::system::sys_user_model::User;
use crate::model::system::sys_user_post_model::UserPost;
use crate::model::system::sys_user_role_model::{is_admin, UserRole};
use crate::utils::jwt_util::JwtToken;
use crate::utils::user_agent_util::UserAgentUtil;
use crate::vo::system::sys_dept_vo::{DeptResp};
use crate::vo::system::sys_user_vo::*;
use crate::RB;
use log::info;
use ntex::http::Response;
use ntex::web;
use ntex::web::types::Json;
use rbatis::plugin::page::PageRequest;
use rbatis::rbatis_codegen::ops::AsProxy;
use rbatis::rbdc::datetime::DateTime;
use rbs::value;
use std::collections::{HashMap, HashSet};
use crate::vo::system::sys_role_vo::RoleResp;
/*
 *添加用户信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/addUser")]
pub async fn add_sys_user(item: Json<UserReq>) -> AppResult<Response> {
    info!("add sys_user params: {:?}", &item);
    let rb = &mut RB.clone();
    let mut req = item.0;

    if User::select_by_user_name(rb, &req.user_name).await?.is_some() {
        return Err(AppError::BusinessError("登录账号已存在"));
    }

    if User::select_by_mobile(rb, &req.mobile).await?.is_some() {
        return Err(AppError::BusinessError("手机号码已存在"));
    }

    if User::select_by_email(rb, &req.email).await?.is_some() {
        return Err(AppError::BusinessError("邮箱账号已存在"));
    }

    let post_ids = req.post_ids.clone();
    req.id = None;
    let id = User::insert(rb, &User::from(req)).await?.last_insert_id;

    let mut user_post_list: Vec<UserPost> = Vec::new();
    for post_id in post_ids {
        user_post_list.push(UserPost { user_id: id.i64(), post_id })
    }

    UserPost::insert_batch(rb, &user_post_list, user_post_list.len() as u64).await.map(|_| ok_result())?
}

/*
 *删除用户信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/deleteUser")]
pub async fn delete_sys_user(
    req: web::HttpRequest,
    item: Json<DeleteUserReq>,
) -> AppResult<Response> {
    info!("delete sys_user params: {:?}", &item);
    let rb = &mut RB.clone();

    let user_id = req
        .headers()
        .get("userId")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    info!("query user menu params user_id {:?}", user_id);

    let ids = item.ids.clone();
    if ids.contains(&user_id) {
        return Err(AppError::BusinessError("当前用户不能删除"));
    }
    if ids.contains(&1) {
        return Err(AppError::BusinessError("不允许操作超级管理员用户"));
    }

    UserRole::delete_by_map(rb, value! {"user_id": &ids}).await?;
    UserPost::delete_by_map(rb, value! {"user_id": &ids}).await?;
    User::delete_by_map(rb, value! {"id": &ids}).await.map(|_| ok_result())?
}

/*
 *更新用户信息
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/updateUser")]
pub async fn update_sys_user(item: Json<UserReq>) -> AppResult<Response> {
    info!("update sys_user params: {:?}", &item);
    let rb = &mut RB.clone();
    let req = item.0;

    let id = req.id.clone();
    if id == Some (1){
        return Err(AppError::BusinessError("不允许操作超级管理员用户"));
    }

    let user = match User::select_by_id(rb, req.id.unwrap_or_default()).await? {
        None => return Err(AppError::BusinessError("用户不存在")),
        Some(x) => x,
    };

    if let Some(x) = User::select_by_user_name(rb, &req.user_name).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("登录账号已存在"));
        }
    }

    if let Some(x) = User::select_by_mobile(rb, &req.mobile).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("手机号码已存在"));
        }
    }

    if let Some(x) = User::select_by_email(rb, &req.email).await? {
        if x.id != req.id {
            return Err(AppError::BusinessError("邮箱账号已存在"));
        }
    }

    let post_ids = req.post_ids.clone();
    let mut user_post_list: Vec<UserPost> = Vec::new();
    for post_id in post_ids {
        user_post_list.push(UserPost {
            user_id: user.id.unwrap_or_default(),
            post_id,
        })
    }

    UserPost::delete_by_map(rb, value! {"user_id": &id}).await?;
    UserPost::insert_batch(rb, &user_post_list, user_post_list.len() as u64).await?;

    User::update_by_map(rb, &User::from(req), value! {"id": &id}).await.map(|_| ok_result())?
}

/*
 *更新用户信息状态
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/updateUserStatus")]
pub async fn update_sys_user_status(item: Json<UpdateUserStatusReq>) -> AppResult<Response> {
    info!("update sys_user_status params: {:?}", &item);
    let rb = &mut RB.clone();

    let req = item.0;

    let ids = req.ids.clone();
    if ids.contains(&1) {
        return Err(AppError::BusinessError("不允许操作超级管理员用户"));
    }

    let update_sql = format!(
        "update sys_user set status = ? where id in ({})",
        req.ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );

    let mut param = vec![value!(req.status)];
    param.extend(req.ids.iter().map(|&id| value!(id)));
    rb.exec(&update_sql, param).await.map(|_| ok_result())?
}

/*
 *重置用户密码
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/resetUserPassword")]
pub async fn reset_sys_user_password(item: Json<ResetUserPwdReq>) -> AppResult<Response> {
    info!("update sys_user_password params: {:?}", &item);
    let req = item.0;
    let rb = &mut RB.clone();

    let id = req.id.clone();
    if id == 1 {
        return Err(AppError::BusinessError("不允许操作超级管理员用户"));
    }

    let sys_user_result = User::select_by_id(rb, req.id).await?;

    match sys_user_result {
        None => Err(AppError::BusinessError("用户不存在")),
        Some(x) => {
            let mut user = x;
            user.password = req.password;
            User::update_by_map(rb, &user, value! {"id": &user.id}).await.map(|_| ok_result())?
        }
    }
}

/*
 *用户修改自己的密码
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/updateUserPassword")]
pub async fn update_sys_user_password(
    http_req: web::HttpRequest,
    item: Json<UpdateUserPwdReq>,
) -> AppResult<Response> {
    info!("update sys_user_password params: {:?}", &item);
    let req = item.0;
    let rb = &mut RB.clone();

    let user_id = http_req
        .headers()
        .get("userId")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    info!("query user menu params user_id {:?}", user_id);

    match User::select_by_id(rb, user_id).await? {
        None => Err(AppError::BusinessError("用户不存在")),
        Some(x) => {
            let mut user = x;
            if user.password != req.pwd {
                return Err(AppError::BusinessError("旧密码不正确"));
            }
            user.password = req.re_pwd;
            User::update_by_map(rb, &user, value! {"id": &user.id}).await.map(|_| ok_result())?
        }
    }
}

/*
 *查询用户信息详情
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/queryUserDetail")]
pub async fn query_sys_user_detail(item: Json<QueryUserDetailReq>) -> AppResult<Response> {
    info!("query sys_user_detail params: {:?}", &item);
    let rb = &mut RB.clone();

    match User::select_by_id(rb, item.id).await? {
        None => Err(AppError::BusinessError("用户不存在")),
        Some(x) => {
            let dept_result = Dept::select_by_id(rb, &x.dept_id).await?;
            let dept:DeptResp = match dept_result {
                None => {
                    return Err(AppError::BusinessError("查询用户详细信息失败,部门不存在"));
                }
                Some(x) => {
                    x.into()
                }
            };

            let post_ids = UserPost::select_by_map(rb, value! {"user_id": item.id})
                .await?
                .iter()
                .map(|x| x.post_id)
                .collect::<Vec<i64>>();

            let mut a: UserResp = x.into();
            a.dept_info = Some(dept);
            a.post_ids = Some(post_ids);

            ok_result_data(a)
        }
    }
}

/*
 *查询用户信息列表
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/queryUserList")]
pub async fn query_sys_user_list(item: Json<QueryUserListReq>) -> AppResult<Response> {
    info!("query sys_user_list params: {:?}", &item);
    let rb = &mut RB.clone();

    let mobile = item.mobile.as_deref().unwrap_or_default();
    let user_name = item.user_name.as_deref().unwrap_or_default();
    let status = item.status.unwrap_or(2);
    let dept_id = item.dept_id.unwrap_or_default();

    let page = &PageRequest::new(item.page_no, item.page_size);
    let d = User::select_sys_user_list(rb, page, mobile, user_name, status, dept_id).await?;

    let total = d.total;
    let mut sys_user_list_data: Vec<UserResp> = Vec::new();
    for x in d.records {
        sys_user_list_data.push(x.into())
    }

    ok_result_page(sys_user_list_data, total)
}

/*
 *用户登录
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/login")]
pub async fn login(item: Json<UserLoginReq>) -> AppResult<Response> {
    info!("user login params: {:?}", &item);
    let req = item.0;
    let rb = &mut RB.clone();

    // let user_agent = headers.get("User-Agent").unwrap().to_str().unwrap();
    let user_agent = "";
    info!("user agent: {:?}", user_agent);
    let agent = UserAgentUtil::new(user_agent);

    let user_result = User::select_by_account(rb, &req.account).await?;
    info!("query user by account: {:?}", user_result);

    match user_result {
        None => {
            add_login_log(req.account, 0, "用户不存在", agent).await;
            Err(AppError::BusinessError("用户不存在"))
        }
        Some(user) => {
            let mut s_user = user.clone();
            let id = user.id.unwrap();
            let username = user.user_name;
            let password = user.password;

            if password.ne(&req.password) {
                add_login_log(req.account, 0, "密码不正确", agent).await;
                return err_result_msg("密码不正确");
            }

            let btn_menu = query_btn_menu(&id).await;

            if btn_menu.len() == 0 {
                add_login_log(req.account, 0, "用户没有分配角色或者菜单,不能登录", agent).await;
                return Err(AppError::BusinessError("用户没有分配角色或者菜单,不能登录"));
            }

            let token = JwtToken::new(id, &username, btn_menu).create_token("123")?;

            add_login_log(req.account, 1, "登录成功", agent.clone()).await;
            s_user.login_os = agent.os;
            s_user.login_browser = agent.browser;
            s_user.login_date = Some(DateTime::now());
            User::update_by_map(rb, &s_user, value! {"id": &s_user.id}).await?;
            ok_result_data(token)
        }
    }
}

/*
 *添加登录日志
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
async fn add_login_log(name: String, status: i8, msg: &str, agent: UserAgentUtil) {
    let rb = &mut RB.clone();

    let sys_login_log = LoginLog {
        id: None,                             //访问ID
        login_name: name,                     //登录账号
        ipaddr: "todo".to_string(),           //登录IP地址
        login_location: "todo".to_string(),   //登录地点
        platform: agent.platform,             //平台信息
        browser: agent.browser,               //浏览器类型
        version: agent.version,               //浏览器版本
        os: agent.os,                         //操作系统
        arch: agent.arch,                     //体系结构信息
        engine: agent.engine,                 //渲染引擎信息
        engine_details: agent.engine_details, //渲染引擎详细信息
        extra: agent.extra,                   //其他信息（可选）
        status,                               //登录状态(0:失败,1:成功)
        msg: msg.to_string(),                 //提示消息
        login_time: None,                     //访问时间
    };

    match LoginLog::insert(rb, &sys_login_log).await {
        Ok(_u) => log::info!("add_login_log success: {:?}", sys_login_log),
        Err(err) => log::error!(
            "add_login_log error params: {:?}, error message: {:?}",
            sys_login_log,
            err
        ),
    }
}

/*
 *查询按钮权限
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
async fn query_btn_menu(id: &i64) -> Vec<String> {
    let rb = &mut RB.clone();

    let count = is_admin(&rb, id).await.unwrap_or_default();
    let mut btn_menu: Vec<String> = Vec::new();
    if count == 1 {
        let data = Menu::select_all(rb).await;

        for x in data.unwrap_or_default() {
            btn_menu.push(x.api_url.unwrap_or_default());
        }
        log::info!("admin login: {:?}", id);
        btn_menu
    } else {
        let btn_menu_map: Vec<HashMap<String, String>> = rb.query_decode("select distinct u.api_url from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = ?", vec![value!(id)]).await.unwrap();
        for x in btn_menu_map {
            btn_menu.push(x.get("api_url").unwrap().to_string());
        }
        log::info!("ordinary login: {:?}", id);
        btn_menu
    }
}

/*
 *查询用户角色
 *author：刘飞华
 *date：2025/01/10 09:21:35
 */
#[web::post("/user/queryUserRole")]
pub async fn query_user_role(item: Json<QueryUserRoleReq>) -> AppResult<Response> {
    info!("query user_role params: {:?}", item);
    let rb = &mut RB.clone();

    let role_list = Role::select_all(rb).await.map(|x| x.into_iter().map(|x| x.into()).collect::<Vec<RoleResp>>())?;
    let sys_role_list = role_list.clone();
    let mut user_role_ids = role_list.into_iter().map(|x| x.id.unwrap_or_default()).collect::<Vec<i64>>();

    if item.user_id != 1 {
        let vec1 = UserRole::select_by_map(rb, value! {"user_id": item.user_id}).await?;
        user_role_ids = vec1.into_iter().map(|x| x.id.unwrap_or_default()).collect::<Vec<i64>>();
    }

    ok_result_data(QueryUserRoleResp { sys_role_list, user_role_ids })
}

/*
*更新用户角色
*author：刘飞华
*date：2025/01/10 09:21:35
*/
#[web::post("/user/updateUserRole")]
pub async fn update_user_role(item: Json<UpdateUserRoleReq>) -> AppResult<Response> {
    info!("update_user_role params: {:?}", item);
    let rb = &mut RB.clone();

    let user_id = item.user_id;
    let role_ids = &item.role_ids;
    let len = item.role_ids.len();

    if user_id == 1 {
        return Err(AppError::BusinessError("不能修改超级管理员的角色"));
    }

    UserRole::delete_by_map(rb, value! {"user_id": user_id}).await?;

    let mut list: Vec<UserRole> = Vec::new();
    for role_id in role_ids {
        let r_id = role_id.clone();
        list.push(UserRole {
            id: None,
            create_time: Some(DateTime::now()),
            role_id: r_id,
            user_id: user_id.clone(),
        })
    }

    UserRole::insert_batch(rb, &list, len as u64).await?;

    ok_result()
}

/*
*查询用户菜单
*author：刘飞华
*date：2025/01/10 09:21:35
*/
#[web::get("/user/queryUserMenu")]
pub async fn query_user_menu(req: web::HttpRequest) -> AppResult<Response> {
    let rb = &mut RB.clone();

    let user_id = req
        .headers()
        .get("userId")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    info!("query user menu params user_id {:?}", user_id);

    //根据id查询用户
    //根据id查询用户
    match User::select_by_id(rb, user_id).await? {
        None => Err(AppError::BusinessError("用户不存在")),
        Some(user) => {
            //role_id为1是超级管理员--判断是不是超级管理员
            let sql_str = "select count(id) from sys_user_role where role_id = 1 and user_id = ?";
            let count = rb
                .query_decode::<i32>(sql_str, vec![value!(user.id)])
                .await?;

            let sys_menu_list: Vec<Menu>;

            if count > 0 {
                log::info!("The current user is a super administrator");
                sys_menu_list = Menu::select_all(rb).await?;
            } else {
                log::info!("The current user is not a super administrator");
                let sql_str = "select u.* from sys_user_role t left join sys_role usr on t.role_id = usr.id left join sys_role_menu srm on usr.id = srm.role_id left join sys_menu u on srm.menu_id = u.id where t.user_id = ?";
                sys_menu_list = rb.query_decode(sql_str, vec![value!(user.id)]).await?;
            }

            let mut sys_menu: Vec<MenuList> = Vec::new();
            let mut btn_menu: Vec<String> = Vec::new();
            let mut sys_menu_ids: HashSet<i64> = HashSet::new();

            for x in sys_menu_list {
                if x.visible == 0 {
                    continue;
                }
                if x.menu_type != 3 {
                    sys_menu_ids.insert(x.id.unwrap_or_default().clone());
                    sys_menu_ids.insert(x.parent_id.unwrap_or_default().clone());
                }

                if x.api_url.clone().unwrap_or_default().len() > 0 {
                    btn_menu.push(x.api_url.unwrap_or_default());
                }
            }

            let mut menu_ids = Vec::new();
            for id in sys_menu_ids {
                menu_ids.push(id)
            }
            let vec1 = Menu::select_by_ids(rb, &menu_ids).await?;
            for menu in vec1 {
                sys_menu.push(MenuList {
                    id: menu.id.unwrap_or_default(),
                    parent_id: menu.parent_id,
                    name: menu.menu_name,
                    icon: menu.menu_icon.unwrap_or_default(),
                    api_url: menu
                        .api_url
                        .as_ref()
                        .map_or_else(|| "".to_string(), |url| url.to_string()),
                    menu_type: menu.menu_type,
                    path: menu.menu_url.unwrap_or_default(),
                });
            }

            let resp = QueryUserMenuResp {
                sys_menu,
                btn_menu,
                avatar:
                    "https://gw.alipayobjects.com/zos/antfincdn/XAosXuNZyF/BiazfanxmamNRoxxVxka.png"
                        .to_string(),
                name: user.user_name,
            };

            ok_result_data(resp)
        }
    }
}
