use actix_web::{
    http::StatusCode,
    HttpResponse, ResponseError,
};
use sea_orm::DbErr;
use serde::Serialize;
use std::fmt;
pub(crate) use crate::utils::db::DbError;

/// 全局错误类型
#[derive(Debug)]
pub enum AppError {
    // 数据库错误
    DatabaseError(DbErr),
    // 参数错误
    InvalidParameter(String),
    // 记录未找到
    RecordNotFound,
    // 权限不足
    PermissionDenied,
    // 认证失败
    AuthenticationFailed,
    // 业务逻辑错误
    BusinessError(String),
    // 内部服务器错误
    InternalError(String),
    InvalidPassword(String),
    UserNotFound,
}


/// 统一响应结构体
#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "数据库错误: {}", e),
            AppError::InvalidParameter(msg) => write!(f, "参数错误: {}", msg),
            AppError::RecordNotFound => write!(f, "记录不存在"),
            AppError::PermissionDenied => write!(f, "权限不足"),
            AppError::AuthenticationFailed => write!(f, "认证失败"),
            AppError::BusinessError(msg) => write!(f, "业务错误: {}", msg),
            AppError::InternalError(msg) => write!(f, "内部错误: {}", msg),
            AppError::InvalidPassword(msg) => write!(f, "{}", msg),
            AppError::UserNotFound => write!(f, "用户不存在"),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, code, message) = match self {
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                50001,
                "数据库操作失败".to_string(),
            ),
            AppError::InvalidParameter(msg) => (
                StatusCode::BAD_REQUEST,
                40001,
                msg.clone(),
            ),
            AppError::RecordNotFound => (
                StatusCode::NOT_FOUND,
                40401,
                "请求的记录不存在".to_string(),
            ),
            AppError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                40301,
                "没有操作权限".to_string(),
            ),
            AppError::AuthenticationFailed => (
                StatusCode::UNAUTHORIZED,
                40101,
                "认证失败，请重新登录".to_string(),
            ),
            AppError::BusinessError(msg) => (
                StatusCode::BAD_REQUEST,
                40002,
                msg.clone(),
            ),
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                50002,
                msg.clone(),
            ),
            AppError::InvalidPassword(msg) => (
                StatusCode::BAD_REQUEST,
                4000,
                msg.clone()
            ),
            AppError::UserNotFound => (
                StatusCode::BAD_REQUEST,
                40001,
                "用户不存在".to_string(),
                )
        };

        HttpResponse::build(status_code).json(ErrorResponse {
            code,
            message,
            data: None,
        })
    }
}

/// 转换 SeaORM 错误到 AppError
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::DatabaseError(err)
    }
}

// src/error.rs
impl From<aws_sdk_s3::Error> for AppError {
    fn from(err: aws_sdk_s3::Error) -> Self {
        AppError::InternalError(format!("存储服务错误: {}", err))
    }
}
/// 统一成功响应
pub fn success_response<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "message": "操作成功",
        "data": data
    }))
}

/// 简化 Result 类型
pub type AppResult<T> = Result<T, AppError>;


// ========== 错误类型转换（关键适配） ==========
impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        match err {
            // 数据库底层错误
            DbError::Database(e) => AppError::DatabaseError(e),

            // 配置错误
            DbError::ConfigError(msg) => AppError::InternalError(format!("数据库配置错误: {}", msg)),

            // 业务错误
            DbError::BusinessError(msg) => AppError::BusinessError(msg),

            // 参数错误
            DbError::InvalidParameter(msg) => AppError::InvalidParameter(msg),

            // 记录不存在
            DbError::RecordNotFound => AppError::RecordNotFound,

            // 操作失败 → 内部错误
            DbError::OperationError(msg) => AppError::InternalError(msg),
            DbError::InvalidPassword => AppError::InvalidPassword("密码错误".to_string()),
        }
    }
}

// 简化 Result 类型（全局复用）
// pub type AppResult<T> = Result<T, AppError>;