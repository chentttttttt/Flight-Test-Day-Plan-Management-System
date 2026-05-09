use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::utils::response::success_response;

// ---------- DTO ----------
#[derive(Debug, Deserialize)]
pub struct SetAttributeReq {
    pub key: String,
    pub value: String,      // 由于服务要求字符串，前端传入字符串
}

#[derive(Debug, Deserialize)]
pub struct SetAllAttrsReq {
    pub resource_type: String,
    pub resource_id: i64,
    pub attributes: Value,  // JSON 对象
}

// ---------- Handlers ----------

/// 设置单个属性（合并）
pub async fn set_attribute(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64, String)>, // (resource_type, resource_id, key)
    req: web::Json<SetAttributeReq>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id, key) = path.into_inner();
    services.resource_attribute
        .set_attr(resource_id, &resource_type, &key, &req.value)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}

/// 获取资源的所有属性
pub async fn get_all_attributes(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64)>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id) = path.into_inner();
    let attrs = services.resource_attribute
        .get_attrs(resource_id, &resource_type)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(attrs.unwrap_or_default()))
}

/// 获取单个属性值
pub async fn get_attribute(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64, String)>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id, key) = path.into_inner();
    let value = services.resource_attribute
        .get_attr(resource_id, &resource_type, &key)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(value))
}

/// 删除单个属性
pub async fn remove_attribute(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64, String)>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id, key) = path.into_inner();
    let deleted = services.resource_attribute
        .delete_attr(resource_id, &resource_type, &key)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(deleted))
}

/// 批量设置所有属性（替换整个 JSON）
pub async fn set_all_attributes(
    services: web::Data<ServiceContainer>,
    req: web::Json<SetAllAttrsReq>,
) -> AppResult<HttpResponse> {
    services.resource_attribute
        .set_all_attrs(req.resource_id, &req.resource_type, req.attributes.clone())
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}

/// 删除资源的所有属性
pub async fn delete_all_attributes(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64)>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id) = path.into_inner();
    let deleted = services.resource_attribute
        .delete_all_attrs(resource_id, &resource_type)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(deleted))
}

/// 删除资源
pub async fn delete_resource(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64)>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id) = path.into_inner();
    let deleted = services.resource_attribute
        .delete_resource(resource_id, &resource_type)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(deleted))
}

/// 根据资源类型获取所有记录
pub async fn list_by_type(
    services: web::Data<ServiceContainer>,
    resource_type: web::Path<String>,
) -> AppResult<HttpResponse> {
    let list = services.resource_attribute
        .list_all_attrs_by_type(&resource_type)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(list))
}

/// 获取所有未删除的记录
pub async fn list_all(
    services: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let list = services.resource_attribute
        .list_all_attrs()
        .await
        .map_err(AppError::from)?;
    Ok(success_response(list))
}

/// 根据资源ID和类型获取记录（含ID）
pub async fn find_by_resource(
    services: web::Data<ServiceContainer>,
    path: web::Path<(String, i64)>,
) -> AppResult<HttpResponse> {
    let (resource_type, resource_id) = path.into_inner();
    let record = services.resource_attribute
        .find_by_resource(resource_id, &resource_type)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(record))
}