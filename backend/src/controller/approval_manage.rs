use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::service::approval_manage::{
    ApprovalFlowCreateDto, ApprovalFlowUpdateDto,
    ApprovalNodeCreateDto, ApprovalNodeUpdateDto,
};
use crate::utils::response::success_response;

// ---------- 流程 DTO ----------
#[derive(Debug, Deserialize)]
pub struct CreateFlowReq {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlowReq {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
}

// ---------- 节点 DTO ----------
#[derive(Debug, Deserialize)]
pub struct CreateNodeReq {
    pub flow_id: i64,
    pub node_order: i32,
    pub node_name: String,
    pub approver_type: String,
    pub approver_value: Option<String>,
    pub approve_strategy: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeReq {
    pub node_name: Option<String>,
    pub approver_type: Option<String>,
    pub approver_value: Option<String>,
    pub approve_strategy: Option<String>,
    pub status: Option<i16>,
}

// ========== 流程操作 ==========
pub async fn create_flow(
    services: web::Data<ServiceContainer>,
    req: web::Json<CreateFlowReq>,
) -> AppResult<HttpResponse> {
    let dto = ApprovalFlowCreateDto {
        code: req.code.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        status: req.status,
    };
    let flow = services.approval_manage.create_flow(dto).await?;
    Ok(success_response(flow))
}

pub async fn update_flow(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    req: web::Json<UpdateFlowReq>,
) -> AppResult<HttpResponse> {
    let dto = ApprovalFlowUpdateDto {
        name: req.name.clone(),
        description: req.description.clone(),
        status: req.status,
    };
    let flow = services.approval_manage.update_flow(*id, dto).await?;
    Ok(success_response(flow))
}

pub async fn delete_flow(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    services.approval_manage.delete_flow(*id).await?;
    Ok(success_response(()))
}

pub async fn get_flow(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let flow = services.approval_manage.get_flow(*id).await?
        .ok_or_else(|| AppError::BusinessError("流程不存在".into()))?;
    Ok(success_response(flow))
}

pub async fn get_active_flow(
    services: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let flow = services.approval_manage.get_active_flow().await?;
    Ok(success_response(flow))
}

// ========== 节点操作 ==========
pub async fn create_node(
    services: web::Data<ServiceContainer>,
    req: web::Json<CreateNodeReq>,
) -> AppResult<HttpResponse> {
    let dto = ApprovalNodeCreateDto {
        flow_id: req.flow_id,
        node_order: req.node_order,
        node_name: req.node_name.clone(),
        approver_type: req.approver_type.clone(),
        approver_value: req.approver_value.clone(),
        approve_strategy: req.approve_strategy.clone(),
        status: req.status,
    };
    let node = services.approval_manage.create_node(dto).await?;
    Ok(success_response(node))
}

pub async fn update_node(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    req: web::Json<UpdateNodeReq>,
) -> AppResult<HttpResponse> {
    let dto = ApprovalNodeUpdateDto {
        node_name: req.node_name.clone(),
        approver_type: req.approver_type.clone(),
        approver_value: req.approver_value.clone(),
        approve_strategy: req.approve_strategy.clone(),
        status: req.status,
    };
    let node = services.approval_manage.update_node(*id, dto).await?;
    Ok(success_response(node))
}

pub async fn delete_node(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    services.approval_manage.delete_node(*id).await?;
    Ok(success_response(()))
}

pub async fn get_node(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let node = services.approval_manage.get_node(*id).await?
        .ok_or_else(|| AppError::BusinessError("节点不存在".into()))?;
    Ok(success_response(node))
}

// ========== 树形数据 ==========
pub async fn get_flow_tree(
    services: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let tree = services.approval_manage.get_flow_tree().await?;
    Ok(success_response(tree))
}

pub async fn get_flow_tree_by_id(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let tree = services.approval_manage.get_flow_tree_by_id(*id).await?;
    Ok(success_response(tree))
}