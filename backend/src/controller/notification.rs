//! 通知控制器（C层）
//! 提供通知的 REST API 接口

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use crate::service::notification::{CreateNotificationReq, QueryNotificationReq, MarkReadReq, NotificationListResp, NotificationResp};
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::utils::response::success_response;
use crate::utils::jwt::{get_jwt_claims, JwtClaims};

// ========== Handlers ==========

/// 创建通知（管理员或系统内部调用）
/// POST /api/notification
pub async fn create(
    services: web::Data<ServiceContainer>,
    req: web::Json<CreateNotificationReq>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_jwt_claims(&http_req)?;
    if claims.role != "ADMIN" {
        return Err(AppError::PermissionDenied);
    }
    let notifications = services.notification.create(req.into_inner()).await?;
    Ok(success_response(notifications))
}

/// 获取当前用户的通知列表（推荐：从 JWT 中获取 user_id）
/// GET /api/notification/list
pub async fn list_current_user(
    services: web::Data<ServiceContainer>,
    query: web::Json<QueryNotificationReq>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_jwt_claims(&http_req)?; //todo 更改抛错
    let user_id = claims.id;
    let (list, total) = services.notification.list_by_user(user_id, query.into_inner()).await?;
    let resp = NotificationListResp {
        list: list.into_iter().map(|n| n.into()).collect(),
        total,
    };
    Ok(success_response(resp))
}

/// 获取指定用户的通知列表（仅管理员可用）
/// GET /api/notification/list/{user_id}
pub async fn list_by_user_id(
    services: web::Data<ServiceContainer>,
    path: web::Path<i64>,
    query: web::Json<QueryNotificationReq>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_jwt_claims(&http_req)?;
    if claims.role != "admin" {
        return Err(AppError::PermissionDenied);
    }
    let user_id = path.into_inner();
    let (list, total) = services.notification.list_by_user(user_id, query.into_inner()).await?;
    let resp = NotificationListResp {
        list: list.into_iter().map(|n| n.into()).collect(),
        total,
    };
    Ok(success_response(resp))
}

/// 批量标记通知为已读（仅限本人）
/// POST /api/notification/mark-read
pub async fn mark_read(
    services: web::Data<ServiceContainer>,
    req: web::Json<MarkReadReq>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_jwt_claims(&http_req)?;
    let user_id = claims.id;
    services.notification.mark_as_read(user_id, req.notification_ids.clone()).await?;
    Ok(success_response(()))
}

/// 删除通知（仅限本人）
/// DELETE /api/notification/{id}
pub async fn delete(
    services: web::Data<ServiceContainer>,
    path: web::Path<i64>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_jwt_claims(&http_req)?;
    let user_id = claims.id;
    let notif_id = path.into_inner();
    let deleted = services.notification.delete(user_id, notif_id).await?;
    if !deleted {
        return Err(AppError::RecordNotFound);
    }
    Ok(success_response(()))
}

/// 获取当前用户未读通知数量
/// GET /api/notification/unread-count
pub async fn unread_count(
    services: web::Data<ServiceContainer>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    let claims = get_jwt_claims(&http_req)?;
    let user_id = claims.id;
    let count = services.notification.unread_count(user_id).await?;
    Ok(success_response(count))
}