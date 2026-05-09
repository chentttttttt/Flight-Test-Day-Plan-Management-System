use std::collections::HashMap;
use actix_web::{web, HttpResponse};
use crate::error::AppResult;
use crate::service::ServiceContainer;
use crate::utils::response::success_response;

/// 创建关联
pub async fn create(
    container: web::Data<ServiceContainer>,
    path: web::Path<(i64, i64)>,
) -> AppResult<HttpResponse> {
    let (model_id, subject_id) = path.into_inner();
    let record = container.model_subject.create(model_id, subject_id).await?;
    Ok(success_response(record))
}

/// 删除单个关联
pub async fn delete(
    container: web::Data<ServiceContainer>,
    path: web::Path<(i64, i64)>,
) -> AppResult<HttpResponse> {
    let (model_id, subject_id) = path.into_inner();
    container.model_subject.delete(model_id, subject_id).await?;
    Ok(success_response(serde_json::json!({ "deleted": true })))
}

/// 查询关联（支持按 model_id、subject_id 过滤，若不传则返回全部）
pub async fn list(
    container: web::Data<ServiceContainer>,
    query: web::Query<HashMap<String, String>>,
) -> AppResult<HttpResponse> {
    let model_id = query.get("model_id").and_then(|v| v.parse::<i64>().ok());
    let subject_id = query.get("subject_id").and_then(|v| v.parse::<i64>().ok());
    let items = container.model_subject.list_all(model_id, subject_id).await?;
    Ok(success_response(serde_json::json!({ "data": items })))
}

/// 按机型批量删除
pub async fn delete_by_model(
    container: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let model_id = path.into_inner();
    let affected = container.model_subject.delete_by_model(model_id).await?;
    Ok(success_response(serde_json::json!({ "deleted": affected })))
}

/// 按科目批量删除
pub async fn delete_by_subject(
    container: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let subject_id = path.into_inner();
    let affected = container.model_subject.delete_by_subject(subject_id).await?;
    Ok(success_response(serde_json::json!({ "deleted": affected })))
}