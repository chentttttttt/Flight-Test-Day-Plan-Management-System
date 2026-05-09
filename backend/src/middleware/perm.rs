//! 完整的认证授权中间件
//! 包含：白名单、JWT 解析、路由权限映射、ABAC 授权、用户信息注入
//!
//! 提供三种 ABAC 引擎：
//! - `RoleBasedAbac`：静态内存引擎（适合少量固定角色权限）
//! - `DynamicRoleBasedAbac`：基于数据库的角色权限引擎，支持手动刷新缓存
//! - `PolicyBasedAbac`：基于 JSON 策略文件的引擎

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpMessage,
};
use futures::future::{ok, Ready};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

// 导入项目中的自定义模块
use crate::utils::jwt::*;
use crate::entity::abac_policy_rule::{self};
// 假设 JwtClaims 和 decode_token 在此
use crate::middleware::route_perm_config::get_route_permission;

// ====================================================
// 1. ABAC 引擎 trait（同步版本）
// ====================================================

pub trait AbacEngine: Send + Sync + 'static {
    fn enforce(
        &self,
        subject: &JwtClaims,
        resource_type: &str,
        action: &str,
        req: &ServiceRequest,
    ) -> Result<(), Error>;
}

// ====================================================
// 2. 基于角色的静态引擎（内存）
// ====================================================

#[derive(Debug, Clone)]
pub struct RoleBasedAbac {
    permissions: std::collections::HashMap<String, Vec<(String, String)>>,
}

impl RoleBasedAbac {
    pub fn new(permissions: std::collections::HashMap<String, Vec<(String, String)>>) -> Self {
        Self { permissions }
    }

    fn check(&self, role: &str, resource: &str, action: &str) -> bool {
        if let Some(perms) = self.permissions.get(role) {
            for (r, a) in perms {
                if (r == "*" || r == resource) && (a == "*" || a == action) {
                    return true;
                }
            }
        }
        false
    }
}

impl AbacEngine for RoleBasedAbac {
    fn enforce(
        &self,
        subject: &JwtClaims,
        resource_type: &str,
        action: &str,
        _req: &ServiceRequest,
    ) -> Result<(), Error> {
        if self.check(&subject.role, resource_type, action) {
            Ok(())
        } else {
            Err(actix_web::error::ErrorForbidden("权限不足"))
        }
    }
}

// ====================================================
// 3. 基于数据库的动态角色权限引擎（支持手动刷新）
// ====================================================

/// 动态角色权限引擎，从 `abac_policy_rule` 表加载规则并缓存
#[derive(Debug)]
pub struct DynamicRoleBasedAbac {
    cache: Arc<RwLock<std::collections::HashMap<(String, String, String),Vec<serde_json::value::Value>>>>,
    db: DatabaseConnection,
}

impl DynamicRoleBasedAbac {
    /// 创建引擎，需传入数据库连接
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            db,
        }
    }

    pub async fn new_auto_load(db: DatabaseConnection) -> Result<Self, sea_orm::DbErr> {
        let instance = Self::new(db);
        instance.load().await?; // 自动加载
        Ok(instance)
    }

    /// 从数据库加载所有启用且 effect="allow" 的角色权限规则
    pub async fn load(&self) -> Result<(), sea_orm::DbErr> {
        let rules = abac_policy_rule::Entity::find()
            .filter(abac_policy_rule::Column::Status.eq(1))          // 启用
            .filter(abac_policy_rule::Column::Deleted.eq(0))         // 未删除
            .filter(abac_policy_rule::Column::Effect.eq("ALLOW"))    // 允许规则
            .all(&self.db)
            .await?;

        let mut new_map = std::collections::HashMap::new();
        for rule in rules {

                new_map
                    .entry((rule.subject_type, rule.resource_type, rule.action))
                    .or_insert_with(Vec::new)
                    .push(rule.condition_json.unwrap());
            }
        *self.cache.write().unwrap() = new_map;
        Ok(())
    }

    /// 手动刷新缓存（例如通过管理 API 调用）
    pub async fn refresh(&self) -> Result<(), sea_orm::DbErr> {
        self.load().await
    }
}

impl AbacEngine for DynamicRoleBasedAbac {
    fn enforce(
        &self,
        subject: &JwtClaims,
        resource_type: &str,
        action: &str,
        _req: &ServiceRequest,
    ) -> Result<(), Error> {
        // // 1. 最小飞行时间
        // if let Some(min_h) = cond.get("minFlightHours").and_then(|v| v.as_i64()) {
        //     let h = context.get("flightHours").and_then(|v| v.as_i64()).unwrap_or(0);
        //     if h < min_h {
        //         return Ok(false);
        //     }
        // }
        //todo 越权
        if subject.role.contains("ADMIN") || subject.role.contains("admin") {
            return Ok(());
        }
        let cache = self.cache.read().unwrap();
        if let Some(perms) = cache.get(&(subject.role.clone(), resource_type.parse()?, action.parse()?)) {
            for c in perms {
                println!("{}",c);
                if c.eq("*") {
                    return Ok(());
                }
            }
            println!("无条件");
            return Ok(());
        }
        Err(actix_web::error::ErrorForbidden("权限不足"))
    }
}

impl AbacEngine for Arc<DynamicRoleBasedAbac> {
    fn enforce(
        &self,
        subject: &JwtClaims,
        resource_type: &str,
        action: &str,
        req: &ServiceRequest,
    ) -> Result<(), Error> {
        // 委托给内部的 DynamicRoleBasedAbac
        (**self).enforce(subject, resource_type, action, req)
    }
}

// ====================================================
// 4. 基于策略文件的引擎（可选）
// ====================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyRuleFile {
    effect: String,
    conditions: PolicyCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyCondition {
    #[serde(default)]
    roles: Option<Vec<String>>,
    #[serde(default)]
    resources: Option<Vec<String>>,
    #[serde(default)]
    actions: Option<Vec<String>>,
}

pub struct PolicyBasedAbac {
    rules: Vec<PolicyRuleFile>,
}

impl PolicyBasedAbac {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let rules: Vec<PolicyRuleFile> = serde_json::from_str(json)?;
        Ok(Self { rules })
    }

    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn evaluate(&self, subject: &JwtClaims, resource: &str, action: &str, _req: &ServiceRequest) -> bool {
        for rule in &self.rules {
            if self.match_condition(&rule.conditions, subject, resource, action) {
                return rule.effect == "allow";
            }
        }
        false
    }

    fn match_condition(
        &self,
        cond: &PolicyCondition,
        subject: &JwtClaims,
        resource: &str,
        action: &str,
    ) -> bool {
        if let Some(roles) = &cond.roles {
            if !roles.contains(&subject.role) && !roles.contains(&"*".to_string()) {
                return false;
            }
        }
        if let Some(resources) = &cond.resources {
            if !resources.contains(&resource.to_string()) && !resources.contains(&"*".to_string()) {
                return false;
            }
        }
        if let Some(actions) = &cond.actions {
            if !actions.contains(&action.to_string()) && !actions.contains(&"*".to_string()) {
                return false;
            }
        }
        true
    }
}

impl AbacEngine for PolicyBasedAbac {
    fn enforce(
        &self,
        subject: &JwtClaims,
        resource_type: &str,
        action: &str,
        req: &ServiceRequest,
    ) -> Result<(), Error> {
        if self.evaluate(subject, resource_type, action, req) {
            Ok(())
        } else {
            Err(actix_web::error::ErrorForbidden("策略拒绝访问"))
        }
    }
}

// ====================================================
// 5. 通用辅助函数：提取 Token
// ====================================================

/// 从请求头提取 Bearer Token
fn extract_token(req: &ServiceRequest) -> Result<String, Error> {
    println!("{:?}",req.headers().get("Authorization"));
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("缺少 Authorization 头"))?;
    let auth_str = auth_header
        .to_str()
        .map_err(|_| actix_web::error::ErrorUnauthorized("无效的 Authorization 头"))?;
    if !auth_str.starts_with("Bearer ") {
        return Err(actix_web::error::ErrorUnauthorized(
            "Authorization 格式错误，应为 Bearer <token>",
        ));
    }
    Ok(auth_str[7..].to_string())
}

// ====================================================
// 6. 中间件主体（整合 JWT 解析、路由权限映射、ABAC）
// ====================================================

/// 认证授权中间件构造器
#[derive(Debug, Clone)]
pub struct AuthMiddleware<E: AbacEngine> {
    engine: E,
    whitelist: Vec<String>,
    jwt_secret:  &'static [u8],
}

impl<E: AbacEngine> AuthMiddleware<E> {
    pub fn new(engine: E, whitelist: Vec<String>, jwt_secret:  &'static [u8]) -> Self {
        Self {
            engine,
            whitelist,
            jwt_secret,
        }
    }

    fn is_whitelisted(&self, req: &ServiceRequest) -> bool {
        let path = req.path();
        self.whitelist.iter().any(|prefix| path.starts_with(prefix))
    }
}

// Transform 实现
impl<S, B, E> Transform<S, ServiceRequest> for AuthMiddleware<E>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    E: AbacEngine + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S, E>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service,
            engine: self.engine.clone(),
            whitelist: self.whitelist.clone(),
            jwt_secret: self.jwt_secret,
        })
    }
}

// 实际处理请求的服务
pub struct AuthMiddlewareService<S, E> {
    service: S,
    engine: E,
    whitelist: Vec<String>,
    jwt_secret: &'static [u8],
}

impl<S, B, E> Service<ServiceRequest> for AuthMiddlewareService<S, E>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
    E: AbacEngine + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let u = &req;
        println!("{:?}",u.headers());
        println!("{:#?}",u.path());
        let whitelisted = self.whitelist.iter().any(|prefix| req.path().starts_with(prefix));
        if whitelisted {
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

        // 1. 提取 Token
        let token_result = extract_token(&req);

        // 2. 解析 JWT
        let claims = match token_result.and_then(|token| decode_token(&token, self.jwt_secret)) {
            Ok(claims) => claims,
            Err(e) => return Box::pin(async { Err(e) }),
        };

        println!("{:?}",claims);
        // 3. 路由权限映射
        let (resource_type, action) = get_route_permission(req.method(), req.path());
        println!("{:#?}, {:#?}",resource_type, action);
        if resource_type == "UNKNOWN" || action == "UNKNOWN" {
            let err = actix_web::error::ErrorForbidden("路由未配置权限");
            //先注释了 todo
            //return Box::pin(async { Err(err) });
        }

        // 4. ABAC 校验
        if let Err(e) = self.engine.enforce(&claims, &resource_type, &action, &req) {
            return Box::pin(async { Err(e) });
        }

        // 5. 注入用户信息，放行
        req.extensions_mut().insert(claims);
        let fut = self.service.call(req);
        Box::pin(fut)
    }
}