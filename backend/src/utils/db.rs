// src/db/mod.rs
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("数据库错误: {0}")]
    Database(#[from] DbErr),
    #[error("配置读取失败: {0}")]
    ConfigError(String),

    #[error("失败: {0}")]
    BusinessError(String),
    #[error("失败: {0}")]
    InvalidParameter(String),
    #[error("记录不存在")]
    RecordNotFound,

    // #[error("数据库操作失败: {0}")]
    // DbErr(#[from] DbErr),

    #[error("失败: {0}")]
    OperationError(String),
    #[error("密码错误")]
    InvalidPassword,
}


//实现 From<DbErr> 以便自动转换
// impl From<DbErr> for DbError {
//     fn from(err: DbErr) -> Self {
//         DbError::DbErr(err)
//     }
// }

pub type DbResult<T> = Result<T, DbError>;

/// 创建数据库连接池
pub async fn connect_db() -> DbResult<DatabaseConnection> {
    // 实际项目中建议从配置文件读取，此处为示例
    let database_url = "postgres://postgres:root@localhost:5432/flight_plan";

    let mut opt = ConnectOptions::new(database_url.to_string());
    // 连接池配置优化
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(60 * 60))
        .sqlx_logging(false)
    ;

    let conn = Database::connect(opt).await?;
    Ok(conn)
}