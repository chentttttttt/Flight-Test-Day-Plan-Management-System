//! JWT 工具函数（无中间件版本）

use std::env;
use std::sync::OnceLock;
use actix_web::error::Error;
use actix_web::{HttpMessage, HttpRequest};
use actix_web::web::to;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Validation, TokenData};
use once_cell::sync::Lazy;
use crate::error::AppError;
use crate::utils::conf;

// ==============================
// JWT 载荷固定格式（与ABAC配套）
// ==============================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub exp: i64,       // 过期时间
    pub id: i64,        // 用户ID
    pub role: String, // 角色类型(admin/user)
}

// ==============================
// 【工具函数】解析并验证 Token
// 给 ABAC 中间件直接调用
// ==============================
pub fn decode_token(token: &str, secret: &[u8]) -> Result<JwtClaims, Error> {
    let decoding_key = DecodingKey::from_secret(secret);
    let validation = Validation::new(Algorithm::HS256);
    let token_data: TokenData<JwtClaims> = decode(token, &decoding_key, &validation)
        .map_err(|_| actix_web::error::ErrorUnauthorized("无效或已过期的token"))?;

    Ok(token_data.claims)
}

// ==============================
// 【工具函数】生成 Token（登录/测试用）
// ==============================
pub fn generate_token(
    id: i64,
    role: &str,
    secret: &[u8],
    expire_hours: i64,
) -> Result<String, Error> {
    use chrono::{Utc, Duration};

    let claims = JwtClaims {
        exp: (Utc::now() + Duration::hours(expire_hours)).timestamp(),
        id,
        role: role.to_string(),
    };

    let encoding_key = EncodingKey::from_secret(secret);

    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &encoding_key,
    )
        .map_err(|_| actix_web::error::ErrorInternalServerError("生成token失败"))
}


pub fn get_jwt_claims(req: &HttpRequest) -> Result<JwtClaims, AppError> {
    // 关键：先保存 extensions 到变量，延长生命周期
    let extensions = req.extensions();
    extensions
        .get::<JwtClaims>()
        .cloned()
        .ok_or(AppError::BusinessError("未找到用户信息，请登录".into()))
}


// ==============================
// 默认密钥（可改为从配置读取）
// ==============================
/// JWT 签名密钥（字节数组，供 EncodingKey/DecodingKey 使用）
// 全局 JWT 密钥（字节数组，供加密/解密使用）
pub static JWT_SECRET: Lazy<Vec<u8>> = Lazy::new(|| {
    // 读取环境变量，若不存在则使用默认值
    let secret_str = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your_jwt_secret_key_123456".to_string());
    secret_str.into_bytes()
});

// 全局 JWT 过期小时数（i64）
pub static JWT_EXPIRE_HOURS: Lazy<i64> = Lazy::new(|| {
    env::var("JWT_EXPIRE_HOURS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12)
});

pub fn load_env() {
    dotenvy::dotenv().ok();
}