// src/middleware/auth.rs
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{Ready, ready, err};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

/// 从 JWT 中解析出的用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: i64,
    pub role: String,
    // 可以扩展其他字段，如 username 等
}

/// 实现 FromRequest 以便在 handler 中直接使用 `auth: AuthUser`
impl FromRequest for AuthUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // 从请求头中提取 token
        let auth_header = req.headers().get("Authorization");
        let token = match auth_header {
            Some(header) => header.to_str().unwrap_or("").trim_start_matches("Bearer "),
            None => return err(actix_web::error::ErrorUnauthorized("Missing token")),
        };

        // 解析 token（这里假设 JWT_SECRET 已定义）
        let secret = std::env::var("JWT_SECRET").unwrap_or("your-secret".to_string());
        let key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        match decode::<AuthUser>(token, &key, &validation) {
            Ok(data) => ready(Ok(data.claims)),
            Err(_) => err(actix_web::error::ErrorUnauthorized("Invalid token")),
        }
    }
}