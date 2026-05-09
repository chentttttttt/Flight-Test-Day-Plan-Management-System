use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::service::ServiceContainer;
use crate::service::approval_order::{SubmitApprovalDto, ApproveDto, RejectDto};
use crate::utils::response::success_response;

// ---------- DTO ----------
#[derive(Debug, Deserialize)]
pub struct MyOrdersQuery {
    pub status: Option<String>,
}

// ---------- Handlers ----------

/// 提交审批申请
pub async fn submit_approval(
    services: web::Data<ServiceContainer>,
    auth: AuthUser,
    dto: web::Json<SubmitApprovalDto>,
) -> AppResult<HttpResponse> {
    let order = services.approval_order.submit(dto.into_inner(), auth.id).await?;
    Ok(success_response(order))
}

/// 审批通过
pub async fn approve(
    services: web::Data<ServiceContainer>,
    auth: AuthUser,
    dto: web::Json<ApproveDto>,
) -> AppResult<HttpResponse> {
    let order = services.approval_order.approve(dto.into_inner(), auth.id).await?;
    Ok(success_response(order))
}

/// 审批驳回
pub async fn reject(
    services: web::Data<ServiceContainer>,
    auth: AuthUser,
    dto: web::Json<RejectDto>,
) -> AppResult<HttpResponse> {
    let order = services.approval_order.reject(dto.into_inner(), auth.id).await?;
    Ok(success_response(order))
}

/// 获取当前用户的待办列表
pub async fn todo_list(
    services: web::Data<ServiceContainer>,
    auth: AuthUser,
) -> AppResult<HttpResponse> {
    let orders = services.approval_order.get_todo_list(auth.id).await?;
    Ok(success_response(orders))
}

/// 获取当前用户的已申请/已审批列表
pub async fn my_orders(
    services: web::Data<ServiceContainer>,
    auth: AuthUser,
    query: web::Query<MyOrdersQuery>,
) -> AppResult<HttpResponse> {
    let orders = services.approval_order.get_my_orders(auth.id, query.status.as_deref()).await?;
    Ok(success_response(orders))
}

/// 获取审批记录
pub async fn approval_records(
    services: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let order_id = path.into_inner();
    let records = services.approval_order.get_approval_records(order_id).await?;
    Ok(success_response(records))
}