use crate::error::{AppError, AppResult};
use actix_web::{HttpResponse, web, HttpRequest, HttpMessage};
use sea_orm::ExprTrait;
use serde::Serialize;
use serde_json::json;

// 从 service 导入 DTO
use crate::service::ServiceContainer;
use crate::service::sys_user::{
    SysUserCreateDto, SysUserLoginDto, SysUserQueryDto, SysUserUpdateDto,
};
use crate::utils::db::DbError;
use crate::utils::jwt::{JWT_EXPIRE_HOURS, JWT_SECRET, generate_token, JwtClaims, get_jwt_claims};

// ========== 系统用户控制器 ==========
// 提供用户登录、创建、更新、查询、删除等 HTTP 接口
// 统一响应格式、统一错误处理、统一依赖注入

/// 用户登录
///
/// 接收用户名 + 密码，校验通过后返回用户完整信息
/// 校验失败返回 401 未授权
pub async fn login(
    services: web::Data<ServiceContainer>,
    dto: web::Json<SysUserLoginDto>,
) -> AppResult<HttpResponse> {
    let user = services
        .sys_user
        .login(dto.into_inner())
        .await
        .map_err(|e| match e {
            DbError::InvalidPassword => AppError::InvalidPassword("密码错误".to_string()),
            DbError::Database(db_err) => AppError::DatabaseError(db_err),   // 直接传递 DbErr
            DbError::OperationError(s) => AppError::InternalError(s),
            other => AppError::from(other),   // 使用已有的 From 转换
        })?
        .ok_or(AppError::UserNotFound)?;   // 用户不存

    let token = generate_token(user.id, &user.role, JWT_SECRET.as_ref(), *JWT_EXPIRE_HOURS)
        .map_err(|_| AppError::AuthenticationFailed)?;

    let data = json!({
        "user": user,
        "token": token,
    });

    Ok(success_response_with_message(data, "登录成功"))
}

/// 创建用户
///
/// 管理员创建系统用户
/// 自动密码加密、用户名唯一性校验
pub async fn create_user(
    services: web::Data<ServiceContainer>,
    dto: web::Json<SysUserCreateDto>,
) -> AppResult<HttpResponse> {
    let user = services.sys_user.create_user(dto.into_inner()).await?;
    Ok(success_response_with_message(user, "创建用户成功"))
}

/// 更新用户信息
///
/// 支持部分更新：仅修改传入字段，未传字段保持原值
/// 必须传入用户 ID
pub async fn update_user(
    req: HttpRequest,
    services: web::Data<ServiceContainer>,
    dto: web::Json<SysUserUpdateDto>,
) -> AppResult<HttpResponse> {
    let clamis = get_jwt_claims(&req).unwrap_or(JwtClaims{ exp: 0, id: 0, role:String::from("USER"), });
    let is_admin = clamis.role == String::from("ADMIN");
    let by = clamis.id;
    let user = services.sys_user.update_user(dto.into_inner(), is_admin, by).await?;
    Ok(success_response(user))
}

pub async fn update_self(
    services: web::Data<ServiceContainer>,
    dto: web::Json<SysUserUpdateDto>,
) -> AppResult<HttpResponse> {
    let user = services.sys_user.update_self(dto.into_inner()).await?;
    Ok(success_response(user))
}

/// 分页条件查询用户列表
///
/// 支持：用户名模糊、真实姓名模糊、状态精确查询
/// 返回列表 + 总条数
pub async fn list_users(
    services: web::Data<ServiceContainer>,
    dto: web::Json<SysUserQueryDto>,
) -> AppResult<HttpResponse> {
    let (users, total) = services.sys_user.query_users(dto.into_inner()).await?;
    Ok(success_response(serde_json::json!({
        "list": users,
        "total": total
    })))
}

/// 删除用户
///
/// URL 参数：id = 用户ID
pub async fn delete_user(
    services: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let success = services.sys_user.delete_user(id).await?;
    Ok(success_response(success))
}

/// 根据 ID 查询单个用户信息
pub async fn get_user(
    services: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let user = services
        .sys_user
        .find_by_id(id)
        .await?
        .ok_or(AppError::RecordNotFound)?;
    Ok(success_response(user))
}

pub async fn get_current_user(
    services: web::Data<ServiceContainer>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    let clamis = get_jwt_claims(&req).unwrap_or(JwtClaims{ exp: 0, id: 0, role:String::from("USER"), });
    let id = clamis.id;
    let user = services
        .sys_user
        .find_by_id(id)
        .await?
        .ok_or(AppError::RecordNotFound)?;
    Ok(success_response(user))
}

// ========== 公共工具 ==========

/// 统一成功响应包装
///
/// 返回标准格式：
/// {
///   "code": 200,
///   "message": "操作成功",
///   "data": 业务数据
/// }
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    success_response_with_message(data, "操作成功")
}

/// 成功响应（可自定义消息）
pub fn success_response_with_message<T: Serialize>(data: T, message: &str) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "message": message,
        "data": data
    }))
}
