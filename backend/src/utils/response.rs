// src/utils/response.rs

use actix_web::{HttpResponse, HttpResponseBuilder};
use sea_orm::ColIdx;
use serde::Serialize;
use serde_json::json;
use crate::utils::db::DbError;

/// 统一成功响应
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "code": 200,
        "message": "操作成功",
        "data": data
    }))
}

/// 成功响应（自定义消息）
pub fn success_response_with_message<T: Serialize>(data: T, message: &str) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "code": 200,
        "message": message,
        "data": data
    }))
}
