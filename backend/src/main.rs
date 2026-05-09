#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]
extern crate core;

use actix_web::{web, App, HttpServer, middleware::Logger};
use dotenv::dotenv;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, prelude::*};

// ========== 模块导入 ==========
mod controller;
mod error;
mod service;
mod utils;
mod entity;
mod macros;
mod middleware;
mod milp_client;

use error::{AppError, ErrorResponse};
use service::ServiceContainer;
use utils::routes::configure_routes;
use utils::db::*;
use utils::jwt::*;
use crate::error::AppResult;
use crate::middleware::perm::*;
use crate::utils::conf;
use crate::utils::minio::MinioConfig;

// ========== 主函数 ==========
#[actix_web::main]
async fn main() -> AppResult<()> {
    // 1. 加载环境变量
    dotenvy::dotenv().ok();

    // 2. 初始化日志系统（同时支持 actix 日志 + SeaORM 完整 debug 日志）
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env()
                  .add_directive("actix_web=debug".parse().unwrap())
                  .add_directive("flight_system=debug".parse().unwrap())
                  .add_directive("sea_orm=debug".parse().unwrap()) // SeaORM 完整 SQL 日志
            .add_directive("sqlx::query=debug".parse().unwrap())
        )
        .init();

    // 3. 初始化数据库连接
    println!("正在连接数据库...");
    let db_conn = connect_db().await?;
    println!("数据库连接成功 ✅");

    // ======================
    // 从 utils 初始化 MinIO
    // ======================
    // 初始化MinIO
    let minio_config = MinioConfig::from_env();
    let minio = service::minio::MinioService::new(&minio_config)
        .await
        .map_err(|e| AppError::InternalError(format!("MinIO初始化失败: {}", e)))?;

    // 4. 初始化服务容器（依赖注入）
    let services = Arc::new(ServiceContainer::new(db_conn.clone(), minio));


    // 创建引擎并注入
    let abac_engine = DynamicRoleBasedAbac::new_auto_load(db_conn.clone()).await?;
    let abac_engine = Arc::new(abac_engine);
    let engine_data = web::Data::from(abac_engine.clone());


    // 2. JWT 密钥（从环境变量读取）
    println!("JWT_SECRET: {:?}", *JWT_SECRET);
    println!("JWT_EXPIRE_HOURS: {}", *JWT_EXPIRE_HOURS);


    // 3. 白名单路径（无需认证的公共端点） //todo Arc::new
    let whitelist = vec![
        "/api/user/login".to_string(),
        "/register".to_string(),   // 你原有的白名单
        "/health".to_string(),     // 如果有健康检查
        "/api/menu/role".to_string(),
        "/api/minio".to_string()
        //"/api/notification".to_string(),
    ];

    // 创建中间件（传递 Arc，而不是克隆整个引擎）
    let auth_middleware = AuthMiddleware::new(
        abac_engine.clone(),   // 类型为 Arc<DynamicRoleBasedAbac>
        whitelist.clone(),
        JWT_SECRET.as_slice(), // 注意：JWT_SECRET 类型为 &'static [u8]
    );



    // 5. 启动 HTTP 服务器
    println!("服务器启动: http://127.0.0.1:8080");

    HttpServer::new(move || {
        let jwt_secret: &'static [u8] = JWT_SECRET.as_slice();
        App::new()
            // 全局共享服务
            .app_data(web::Data::from(Arc::clone(&services)))
            .app_data(engine_data.clone())
            // 跨域配置
            .wrap(
                actix_cors::Cors::permissive()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .expose_headers(["Authorization", "Content-Type"])  // 可选，调试用
                    .max_age(3600)
            )
            // .wrap(
            //     AbacMiddleware::new()
            //         .public_path("/api/user/login")
            //         .public_path("/register"),
            // )
            //.wrap(auth_middleware.clone())

            .wrap(auth_middleware.clone())
            // .wrap(AuthMiddleware::new(
            //     engine_data.get_ref().clone(),
            //     whitelist.clone(),
            //     jwt_secret,
            // ))
            // 日志中间件
            .wrap(Logger::default())
            // JSON 解析错误统一处理
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                AppError::InvalidParameter(format!("JSON 解析失败: {}", err)).into()
            }))
            // 注册 ABAC 中间件，并配置跳过哪些路径

            // 路由
            .configure(configure_routes)
    })
        .bind(("127.0.0.1", 8080))
        .map_err(|e| AppError::InternalError(format!("服务器绑定失败: {}", e)))?
        .run()
        .await
        .map_err(|e| AppError::InternalError(format!("服务器运行失败: {}", e)))?;

    Ok(())
}