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
    pub value: Value,
}

#[derive(Debug, Deserialize)]
pub struct SetBatchAttributeReq {
    pub subject_type: String,
    pub subject_id: i64,
    pub attributes: Value, // JSON object
}

// ---------- Handlers ----------

/// 设置单个属性
pub async fn set_attribute(
    services: web::Data<ServiceContainer>,
    subject_type: web::Path<String>,
    subject_id: web::Path<i64>,
    req: web::Json<SetAttributeReq>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_subject_attribute;
    s.set_attr(*subject_id, &subject_type, &req.key, req.value.clone())
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}

/// 批量设置属性（修改后，从请求体获取主体类型和 ID）
pub async fn set_attributes(
    services: web::Data<ServiceContainer>,
    req: web::Json<SetBatchAttributeReq>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_subject_attribute;
    s.set_attr_batch(req.subject_id, &req.subject_type, req.attributes.clone())
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}

/// 获取单个属性
pub async fn get_attribute(
    services: web::Data<ServiceContainer>,
    params: web::Path<(String, i64, String)>,
) -> AppResult<HttpResponse> {
    let (subject_type, subject_id, key) = params.into_inner();
    let s = &services.abac_subject_attribute;
    let value = s.get_attr(subject_id, &subject_type, &key)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(value))
}

/// 获取全部属性
pub async fn get_all_attributes(
    services: web::Data<ServiceContainer>,
    params: web::Path<(String, i64)>,
) -> AppResult<HttpResponse> {
    let (subject_type, subject_id) = params.into_inner();
    let s = &services.abac_subject_attribute;
    let attrs = s.get_all_attr(subject_id, &subject_type)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(attrs))
}

/// 获取指定类型的所有主体属性列表
pub async fn get_all_by_subject_type(
    services: web::Data<ServiceContainer>,
    subject_type: web::Path<String>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_subject_attribute;
    let list = s.get_all_by_subject_type(&subject_type).await.map_err(AppError::from)?;
    Ok(success_response(list))
}

/// 获取指定类型的所有主体属性列表
pub async fn get_all(
    services: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_subject_attribute;
    let list = s.get_all().await.map_err(AppError::from)?;
    Ok(success_response(list))
}

/// 删除单个属性
pub async fn remove_attribute(
    services: web::Data<ServiceContainer>,
    params: web::Path<(String, i64, String)>,
) -> AppResult<HttpResponse> {
    let (subject_type, subject_id, key) = params.into_inner();
    let s = &services.abac_subject_attribute;
    s.remove_attr(subject_id, &subject_type, &key)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}

/// 删除主体所有属性
pub async fn delete_all_attributes(
    services: web::Data<ServiceContainer>,
    params: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let id  = params.into_inner();
    let s = &services.abac_subject_attribute;
    s.delete_all_attr(id)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}

/// 删除主体
pub async fn delete_subject(
    services: web::Data<ServiceContainer>,
    params: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let id  = params.into_inner();
    let s = &services.abac_subject_attribute;
    s.delete_attr(id)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(()))
}