// handler/minio_handler.rs
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures::{TryStreamExt, StreamExt};
use bytes::BytesMut;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use actix_web::web::Query;
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::utils::response::success_response;

#[derive(Deserialize)]
pub struct UploadQuery {
    pub filename: Option<String>,
}

/// 上传文件
/// - 若提供 ?filename=xxx，则按 approval/xxx 存储，覆盖同名文件
/// - 否则使用 UUID 随机文件名
pub async fn upload_file(
    services: web::Data<ServiceContainer>,
    query: Query<UploadQuery>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    let mut field = payload.try_next().await
        .map_err(|e| AppError::InternalError(format!("读取multipart失败: {}", e)))?
        .ok_or_else(|| AppError::InvalidParameter("没有文件".into()))?;

    let content_type = field.content_disposition()
        .ok_or_else(|| AppError::InvalidParameter("缺少Content-Disposition".into()))?;
    let original_name = content_type.get_filename().unwrap_or("unknown").to_string();

    let mut bytes = BytesMut::new();
    while let Some(chunk) = field.next().await {
        let data = chunk.map_err(|e| AppError::InternalError(format!("读取文件块失败: {}", e)))?;
        bytes.extend_from_slice(&data);
    }

    let prefix = "approval"; // 所有文件统一放在此目录下

    let key = if let Some(custom_name) = &query.filename {
        // 使用自定义文件名（覆盖）
        let key = format!("{}/{}", prefix, custom_name);
        services.minio.upload_with_key(bytes.freeze(), &key).await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        key
    } else {
        // 回退到UUID随机文件名
        services.minio.upload(bytes.freeze(), &original_name, prefix).await
            .map_err(|e| AppError::InternalError(e.to_string()))?
    };

    Ok(success_response(json!({ "key": key })))
}

/// 获取文件下载URL
pub async fn get_file_url(
    services: web::Data<ServiceContainer>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let key = path.into_inner();
    let url = services.minio.get_url(&key, 3600).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(success_response(json!({ "url": url })))
}

/// 删除文件
pub async fn delete_file(
    services: web::Data<ServiceContainer>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let key = path.into_inner();
    services.minio.delete(&key).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(success_response(()))
}

/// 列出指定前缀的文件列表
pub async fn list_files(
    services: web::Data<ServiceContainer>,
    query: web::Query<HashMap<String, String>>,
) -> AppResult<HttpResponse> {
    let prefix = query.get("prefix").cloned().unwrap_or_else(|| "".to_string());
    let objects = services.minio.list_objects(&prefix).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let file_list: Vec<serde_json::Value> = objects.iter().map(|obj| {
        json!({
            "key": obj.key().unwrap_or(""),
            "size": obj.size(),
            "last_modified": obj.last_modified().map(|t| t.secs()),
        })
    }).collect();

    Ok(success_response(json!({ "files": file_list })))
}

