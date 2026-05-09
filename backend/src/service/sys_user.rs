use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Order,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::entity::sys_user;
use crate::utils::db::{DbError, DbResult};

// ========== DTO 定义 ==========

/// 创建系统用户请求 DTO
///
/// 包含创建用户所需的所有字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysUserCreateDto {
    pub username: String,
    pub password: String,
    pub real_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i16>,
    pub role: Option<String>,
    pub create_by: Option<i64>,
}

/// 更新系统用户请求 DTO
///
/// 用于修改用户基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysUserUpdateDto {
    pub id: i64,
    pub old_password: Option<String>,
    pub password: Option<String>,
    pub real_name: Option<String>,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<i16>,
}

/// 用户登录请求 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysUserLoginDto {
    pub username: String,
    pub password: String,
}

/// 系统用户分页查询 DTO
///
/// 支持按用户名、真实姓名、状态模糊/精确查询，并支持分页
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysUserQueryDto {
    pub username: Option<String>,
    pub real_name: Option<String>,
    pub status: Option<i16>,
    pub role: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

// ========== 系统用户服务 ==========

/// 系统用户服务
///
/// 提供用户创建、登录、更新、查询、删除等核心业务逻辑
#[derive(Clone)]
pub struct SysUserService {
    db: DatabaseConnection,
}

impl SysUserService {
    /// 构造系统用户服务实例
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 创建系统用户
    ///
    /// # 业务规则
    /// - 用户名不能为空
    /// - 密码长度 ≥ 6 位
    /// - 用户名必须唯一
    /// - 密码自动 bcrypt 加密
    /// - 默认状态为启用（1）
    pub async fn create_user(&self, dto: SysUserCreateDto) -> DbResult<sys_user::Model> {
        // 业务校验
        if dto.username.is_empty() {
            return Err(DbError::InvalidParameter("用户名不能为空".to_string()));
        }
        if dto.password.len() < 6 {
            return Err(DbError::InvalidParameter("密码长度不能小于6位".to_string()));
        }

        // 检查用户名唯一性
        let exists = sys_user::Entity::find()
            .filter(sys_user::Column::Username.eq(&dto.username))
            .count(&self.db)
            .await?
            > 0;
        if exists {
            return Err(DbError::InvalidParameter("用户名已存在".to_string()));
        }

        // 密码加密
        let hashed_pwd = hash(dto.password.as_bytes(), DEFAULT_COST)
            .map_err(|e| DbError::OperationError(format!("密码加密失败: {}", e)))?;

        // 数据库插入
        let mut model = sys_user::ActiveModel {
            id: Default::default(),
            username: Set(dto.username),
            password: Set(hashed_pwd),
            real_name: Set(dto.real_name),
            phone: Set(dto.phone),
            email: Set(dto.email),
            status: Set(dto.status.unwrap_or(1)),
            deleted: Set(0),
            create_by: Set(dto.create_by),
            create_time: Set(Utc::now().naive_utc()),
            update_by: Set(None),
            update_time: Set(None),
            role: Set(dto.role.unwrap_or(String::from("USER"))),
        }
        .insert(&self.db)
        .await?;

        model.password = String::new();

        Ok(model)
    }

    /// 用户登录验证
    ///
    /// # 验证规则
    /// - 用户必须存在
    /// - 账户状态必须为启用（1）
    /// - 密码 bcrypt 校验匹配
    ///
    /// # 返回
    /// - 验证通过：返回用户信息
    /// - 验证失败：返回 None（防枚举攻击）
    pub async fn login(&self, dto: SysUserLoginDto) -> DbResult<Option<sys_user::Model>> {
        // 查询用户（未删除+正常状态）
        let user = sys_user::Entity::find()
            .filter(sys_user::Column::Username.eq(&dto.username))
            .filter(sys_user::Column::Status.eq(1))
            .one(&self.db)
            .await?;

        if let Some(mut user) = user {
            // 密码验证
            let valid = verify(&dto.password, &user.password)
                .map_err(|e| DbError::OperationError(format!("密码验证失败: {}", e)))?;

            if valid {
                user.password = String::new();
                return Ok(Some(user));
            } else {
                return Err(DbError::InvalidPassword);
            }
        }

        Ok(None)
    }

    /// 更新用户信息（通用）
    ///
    /// - 当 `is_admin` 为 `true` 时，按管理员模式更新（可直接修改密码和状态，无需旧密码）
    /// - 当 `is_admin` 为 `false` 时，按用户自助模式更新（需验证旧密码，不可修改状态，不可修改用户名）
    ///
    /// # 参数
    /// - `dto`: 更新数据，所有字段为 `Option`
    /// - `is_admin`: 是否为管理员操作
    ///
    /// # 返回
    /// 更新后的用户模型（密码已清空）
    pub async fn update_user(&self, dto: SysUserUpdateDto, is_admin: bool, by: i64) -> DbResult<sys_user::Model> {
        // 1. 查询用户
        let user = sys_user::Entity::find_by_id(dto.id)
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active_model = user.into_active_model();

        // 2. 密码更新（仅当有旧密码或管理员直接修改时）
        if is_admin {
            if let Some(new_password) = dto.password {
                if new_password.len() < 6 {
                    return Err(DbError::InvalidParameter("密码长度不能小于6位".to_string()));
                }
                let hashed_pwd = hash(new_password.as_bytes(), DEFAULT_COST)
                    .map_err(|e| DbError::OperationError(format!("密码加密失败: {}", e)))?;
                active_model.password = Set(hashed_pwd);
            }
        } else {
            // 自助模式：需验证旧密码，且只有同时提供旧密码和新密码时才更新
            if let Some(old_password) = dto.old_password {
                let valid = verify(&old_password, &active_model.password.as_ref())
                    .map_err(|e| DbError::OperationError(format!("密码验证失败: {}", e)))?;
                if !valid {
                    return Err(DbError::InvalidParameter("旧密码错误".to_string()));
                }
                if let Some(new_password) = dto.password {
                    if new_password.len() < 6 {
                        return Err(DbError::InvalidParameter("密码长度不能小于6位".to_string()));
                    }
                    let hashed_pwd = hash(new_password.as_bytes(), DEFAULT_COST)
                        .map_err(|e| DbError::OperationError(format!("密码加密失败: {}", e)))?;
                    active_model.password = Set(hashed_pwd);
                }
            }
            // 自助模式下不允许更新状态、角色
        }

        // 3. 更新基本信息（通用）
        if let Some(real_name) = dto.real_name {
            active_model.real_name = Set(Option::from(real_name));
        }
        if let Some(phone) = dto.phone {
            active_model.phone = Set(Option::from(phone));
        }
        if let Some(email) = dto.email {
            active_model.email = Set(Option::from(email));
        }

        // 4. 管理员专属字段更新
        if is_admin {
            if let Some(status) = dto.status {
                active_model.status = Set(status);
            }
            // 管理员可更新角色
            if let Some(role) = dto.role {
                active_model.role = Set(role);
            }
        }

        // 5. 更新操作人及时间
        active_model.update_by = Set(Some(by));
        active_model.update_time = Set(Some(Utc::now().naive_utc()));

        // 6. 执行更新
        let mut updated_user = active_model.update(&self.db).await?;
        updated_user.password = String::new(); // 清空密码，避免泄露
        Ok(updated_user)
    }

    pub async fn update_self(&self, dto: SysUserUpdateDto) -> DbResult<sys_user::Model> {
        // 1. 查询用户
        let user = sys_user::Entity::find_by_id(dto.id)
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active_model = user.into_active_model();


        // 需验证旧密码，且只有同时提供旧密码和新密码时才更新
        if let Some(old_password) = dto.old_password {
            let valid = verify(&old_password, &active_model.password.as_ref())
                .map_err(|e| DbError::OperationError(format!("密码验证失败: {}", e)))?;
            if !valid {
                return Err(DbError::InvalidParameter("旧密码错误".to_string()));
            }
            if let Some(new_password) = dto.password {
                if new_password.len() < 6 {
                    return Err(DbError::InvalidParameter("密码长度不能小于6位".to_string()));
                }
                let hashed_pwd = hash(new_password.as_bytes(), DEFAULT_COST)
                    .map_err(|e| DbError::OperationError(format!("密码加密失败: {}", e)))?;
                active_model.password = Set(hashed_pwd);
            }
        }

        // 3. 更新基本信息（通用）
        if let Some(real_name) = dto.real_name {
            active_model.real_name = Set(Option::from(real_name));
        }
        if let Some(phone) = dto.phone {
            active_model.phone = Set(Option::from(phone));
        }
        if let Some(email) = dto.email {
            active_model.email = Set(Option::from(email));
        }

        // 5. 更新操作人及时间
        active_model.update_by = Set(Some(dto.id));
        active_model.update_time = Set(Some(Utc::now().naive_utc()));

        // 6. 执行更新
        let mut updated_user = active_model.update(&self.db).await?;
        updated_user.password = String::new(); // 清空密码，避免泄露
        Ok(updated_user)
    }

    /// 分页条件查询用户
    ///
    /// 支持：
    /// - 用户名模糊查询
    /// - 真实姓名模糊查询
    /// - 状态精确查询
    /// - 按创建时间倒序
    ///
    /// # 返回
    /// (用户列表, 总条数)
    pub async fn query_users(&self, dto: SysUserQueryDto) -> DbResult<(Vec<sys_user::Model>, u64)> {
        let mut query = sys_user::Entity::find();

        // 构建查询条件（空字符串自动跳过）
        // 用户名：非空才查询
        if let Some(username) = dto.username.filter(|s| !s.is_empty()) {
            query = query.filter(sys_user::Column::Username.like(format!("%{}%", username)));
        }
        // 真实姓名：非空才查询
        if let Some(real_name) = dto.real_name.filter(|s| !s.is_empty()) {
            query = query.filter(sys_user::Column::RealName.like(format!("%{}%", real_name)));
        }
        // 状态
        if let Some(status) = dto.status {
            query = query.filter(sys_user::Column::Status.eq(status));
        }
        // 角色：非空才查询
        if let Some(role) = dto.role.filter(|s| !s.is_empty()) {
            query = query.filter(sys_user::Column::Role.eq(role));
        }

        // 排序+分页
        query = query.order_by(sys_user::Column::CreateTime, Order::Desc);
        let total = query.clone().count(&self.db).await?;
        let mut users = query
            .offset(((dto.page - 1) * dto.page_size) as u64)
            .limit(dto.page_size as u64)
            .all(&self.db)
            .await?;

        // 清空所有用户的密码字段
        for user in &mut users {
            user.password = String::new();
        }

        Ok((users, total))
    }

    /// 删除用户
    ///
    ///
    pub async fn delete_user(&self, id: i64) -> DbResult<bool> {
        let mut user = sys_user::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active_model = user.into_active_model();
        let d = active_model.delete(&self.db).await?;

        Ok(d.rows_affected == 1)
    }

    /// 根据用户ID查询单个用户
    pub async fn find_by_id(&self, id: i64) -> DbResult<Option<sys_user::Model>> {
        let user_opt = sys_user::Entity::find_by_id(id)
            .one(&self.db)
            .await?;

        if let Some(mut user) = user_opt {
            user.password = String::new();
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sea_orm::{ConnectionTrait, Database, DbErr};
    use serial_test::serial;
    use std::env;
    use std::sync::Once;

    // 全局日志初始化，确保每个测试只初始化一次
    static INIT_LOGGER: Once = Once::new();

    fn init_logger() {
        INIT_LOGGER.call_once(|| {
            // 设置环境变量，让 sea-orm 输出 SQL 日志
            if std::env::var("RUST_LOG").is_err() {
                unsafe {
                    std::env::set_var("RUST_LOG", "sea_orm=info,debug");
                }
            }
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });
    }

    /// 获取测试数据库连接，并清空测试数据
    async fn get_test_db() -> Result<DatabaseConnection, DbErr> {
        dotenv().ok();
        let database_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL 未配置");

        let db = Database::connect(&database_url).await?;

        // 清空测试数据（确保测试独立）
        db.execute_unprepared("TRUNCATE TABLE sys_user RESTART IDENTITY CASCADE;")
            .await?;

        Ok(db)
    }

    // ========== 辅助函数：创建测试用户 ==========
    async fn create_test_user(service: &SysUserService, username: &str) -> sys_user::Model {
        let dto = SysUserCreateDto {
            username: username.to_string(),
            password: "123456".to_string(),
            real_name: Some(format!("测试用户-{}", username)),
            phone: Some("13800138000".to_string()),
            email: Some(format!("{}@example.com", username)),
            status: Some(1),
            create_by: Some(1),
        };
        service.create_user(dto).await.unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_create_user_success() {
        init_logger();
        println!("[TEST] 开始测试：创建用户成功");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        let dto = SysUserCreateDto {
            username: "testadmin".to_string(),
            password: "123456".to_string(),
            real_name: Some("测试管理员".to_string()),
            phone: Some("13800138000".to_string()),
            email: Some("test@example.com".to_string()),
            status: Some(1),
            create_by: Some(1),
        };

        println!("[INFO] 创建用户请求: {:?}", dto);
        let user = service.create_user(dto).await.unwrap();
        println!("[INFO] 创建成功，用户信息: {:?}", user);

        assert_eq!(user.username, "testadmin");
        assert_eq!(user.real_name, Some("测试管理员".to_string()));
        assert_eq!(user.status, 1);
        assert_eq!(user.deleted, 0);
        println!("[TEST] 完成测试：创建用户成功");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_user_duplicate_username() {
        init_logger();
        println!("[TEST] 开始测试：创建用户重复用户名失败");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        // 先创建一个用户
        let _ = create_test_user(&service, "duplicate").await;

        let dto = SysUserCreateDto {
            username: "duplicate".to_string(),
            password: "123456".to_string(),
            real_name: None,
            phone: None,
            email: None,
            status: Some(1),
            create_by: Some(1),
        };

        println!("[INFO] 尝试创建重复用户名: {}", dto.username);
        let result = service.create_user(dto).await;
        assert!(result.is_err());
        if let Err(DbError::InvalidParameter(msg)) = result {
            assert_eq!(msg, "用户名已存在");
            println!("[INFO] 捕获预期错误: {}", msg);
        } else {
            panic!("期望返回 InvalidParameter 错误");
        }
        println!("[TEST] 完成测试：创建用户重复用户名失败");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_user_password_too_short() {
        init_logger();
        println!("[TEST] 开始测试：创建用户密码过短失败");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        let dto = SysUserCreateDto {
            username: "shortpwd".to_string(),
            password: "123".to_string(),
            real_name: None,
            phone: None,
            email: None,
            status: Some(1),
            create_by: Some(1),
        };

        println!("[INFO] 尝试创建密码过短的用户: {}", dto.username);
        let result = service.create_user(dto).await;
        assert!(result.is_err());
        if let Err(DbError::InvalidParameter(msg)) = result {
            assert_eq!(msg, "密码长度不能小于6位");
            println!("[INFO] 捕获预期错误: {}", msg);
        } else {
            panic!("期望返回 InvalidParameter 错误");
        }
        println!("[TEST] 完成测试：创建用户密码过短失败");
    }

    #[tokio::test]
    #[serial]
    async fn test_login_success() {
        init_logger();
        println!("[TEST] 开始测试：用户登录成功");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        // 创建测试用户
        let user = create_test_user(&service, "logintest").await;

        let dto = SysUserLoginDto {
            username: "logintest".to_string(),
            password: "123456".to_string(),
        };

        println!("[INFO] 登录请求: {:?}", dto);
        let result = service.login(dto).await.unwrap();
        assert!(result.is_some());
        let logged_user = result.unwrap();
        assert_eq!(logged_user.id, user.id);
        assert_eq!(logged_user.username, "logintest");
        println!("[INFO] 登录成功，用户信息: {:?}", logged_user);
        println!("[TEST] 完成测试：用户登录成功");
    }

    #[tokio::test]
    #[serial]
    async fn test_login_failure_wrong_password() {
        init_logger();
        println!("[TEST] 开始测试：用户登录密码错误");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        let _ = create_test_user(&service, "logintest").await;

        let dto = SysUserLoginDto {
            username: "logintest".to_string(),
            password: "wrongpassword".to_string(),
        };

        println!("[INFO] 尝试错误密码登录: username={}", dto.username);
        let result = service.login(dto).await.unwrap();
        assert!(result.is_none());
        println!("[INFO] 登录失败，返回 None");
        println!("[TEST] 完成测试：用户登录密码错误");
    }

    #[tokio::test]
    #[serial]
    async fn test_update_user_self() {
        init_logger();
        println!("[TEST] 开始测试：用户更新自己的信息");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        // 创建用户
        let user = create_test_user(&service, "selfupdate").await;

        let update_dto = SysUserUpdateDto {
            id: user.id,
            old_password: Some("123456".to_string()),
            password: Some("newpassword".to_string()),
            real_name: Some("更新后的姓名".to_string()),
            phone: Some("13999999999".to_string()),
            email: Some("newemail@example.com".to_string()),
            status: None,       // 普通用户不能修改状态
            update_by: user.id, // 自己更新自己
        };

        println!("[INFO] 更新请求: {:?}", update_dto);
        let updated = service.update_user_self(update_dto).await.unwrap();
        println!("[INFO] 更新后的用户: {:?}", updated);

        assert_eq!(updated.real_name, Some("更新后的姓名".to_string()));
        assert_eq!(updated.phone, Some("13999999999".to_string()));
        assert_eq!(updated.email, Some("newemail@example.com".to_string()));

        // 验证密码已更新（通过登录测试）
        let login_dto = SysUserLoginDto {
            username: "selfupdate".to_string(),
            password: "newpassword".to_string(),
        };
        let login_result = service.login(login_dto).await.unwrap();
        assert!(login_result.is_some());
        println!("[INFO] 使用新密码登录成功");
        println!("[TEST] 完成测试：用户更新自己的信息");
    }

    #[tokio::test]
    #[serial]
    async fn test_update_user_by_admin() {
        init_logger();
        println!("[TEST] 开始测试：管理员更新用户信息（包括状态）");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        // 创建用户
        let user = create_test_user(&service, "adminupdate").await;

        let update_dto = SysUserUpdateDto {
            id: user.id,
            old_password: None,
            password: Some("adminpwd".to_string()),
            real_name: Some("管理员更新的姓名".to_string()),
            phone: Some("18888888888".to_string()),
            email: Some("adminupdate@example.com".to_string()),
            status: Some(0), // 禁用用户
            update_by: 999,  // 管理员ID
        };

        println!("[INFO] 管理员更新请求: {:?}", update_dto);
        let updated = service.update_user_by_admin(update_dto).await.unwrap();
        println!("[INFO] 更新后的用户: {:?}", updated);

        assert_eq!(updated.real_name, Some("管理员更新的姓名".to_string()));
        assert_eq!(updated.phone, Some("18888888888".to_string()));
        assert_eq!(updated.email, Some("adminupdate@example.com".to_string()));
        assert_eq!(updated.status, 0); // 状态被禁用

        // 验证登录失败（因为状态为0）
        let login_dto = SysUserLoginDto {
            username: "adminupdate".to_string(),
            password: "adminpwd".to_string(),
        };
        let login_result = service.login(login_dto).await.unwrap();
        assert!(login_result.is_none());
        println!("[INFO] 用户被禁用后登录失败");
        println!("[TEST] 完成测试：管理员更新用户信息");
    }

    #[tokio::test]
    #[serial]
    async fn test_query_users_pagination() {
        init_logger();
        println!("[TEST] 开始测试：分页查询用户");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        // 创建多个用户
        for i in 1..=5 {
            let username = format!("user{}", i);
            create_test_user(&service, &username).await;
        }

        // 查询第一页，每页2条
        let query = SysUserQueryDto {
            username: None,
            real_name: None,
            status: None,
            page: 1,
            page_size: 2,
        };
        println!("[INFO] 查询条件: page=1, page_size=2");
        let (users, total) = service.query_users(query).await.unwrap();
        println!(
            "[INFO] 查询结果: 总记录数={}, 本页记录数={}",
            total,
            users.len()
        );
        assert_eq!(total, 5);
        assert_eq!(users.len(), 2);

        // 验证排序：按创建时间倒序，所以最后创建的排在前面
        // 由于创建顺序是 user1, user2, user3, user4, user5，所以第一页应该是 user5, user4
        assert_eq!(users[0].username, "user5");
        assert_eq!(users[1].username, "user4");

        // 查询第二页
        let query = SysUserQueryDto {
            username: None,
            real_name: None,
            status: None,
            page: 2,
            page_size: 2,
        };
        println!("[INFO] 查询条件: page=2, page_size=2");
        let (users, total) = service.query_users(query).await.unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "user3");
        assert_eq!(users[1].username, "user2");

        // 带条件查询：用户名模糊
        let query = SysUserQueryDto {
            username: Some("user".to_string()),
            real_name: None,
            status: None,
            page: 1,
            page_size: 10,
        };
        println!("[INFO] 模糊查询 username like '%user%'");
        let (users, total) = service.query_users(query).await.unwrap();
        assert_eq!(total, 5);
        assert_eq!(users.len(), 5);

        // 按状态查询（所有用户状态为1）
        let query = SysUserQueryDto {
            username: None,
            real_name: None,
            status: Some(1),
            page: 1,
            page_size: 10,
        };
        println!("[INFO] 按状态 status=1 查询");
        let (users, total) = service.query_users(query).await.unwrap();
        assert_eq!(total, 5);
        assert_eq!(users.len(), 5);

        // 按状态查询状态为0（无）
        let query = SysUserQueryDto {
            username: None,
            real_name: None,
            status: Some(0),
            page: 1,
            page_size: 10,
        };
        println!("[INFO] 按状态 status=0 查询");
        let (users, total) = service.query_users(query).await.unwrap();
        assert_eq!(total, 0);
        assert_eq!(users.len(), 0);

        println!("[TEST] 完成测试：分页查询用户");
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_user() {
        init_logger();
        println!("[TEST] 开始测试：删除用户（硬删除）");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        // 创建用户
        let user = create_test_user(&service, "todelete").await;

        // 删除前查询存在
        let found_before = service.find_by_id(user.id).await.unwrap();
        assert!(found_before.is_some());
        println!("[INFO] 删除前用户存在: id={}", user.id);

        // 执行删除
        println!("[INFO] 删除用户 id={}, update_by=1", user.id);
        let result = service.delete_user(user.id).await.unwrap();
        assert!(result); // 返回 true 表示删除成功
        println!("[INFO] 删除操作影响行数: {}", result);

        // 删除后查询不存在
        let found_after = service.find_by_id(user.id).await.unwrap();
        assert!(found_after.is_none());
        println!("[INFO] 删除后用户已不存在");

        println!("[TEST] 完成测试：删除用户");
    }

    #[tokio::test]
    #[serial]
    async fn test_find_by_id() {
        init_logger();
        println!("[TEST] 开始测试：根据ID查询用户");
        let db = get_test_db().await.unwrap();
        let service = SysUserService::new(db);

        let user = create_test_user(&service, "findbyid").await;

        // 查询存在的用户
        let found = service.find_by_id(user.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);
        println!("[INFO] 查询存在的用户 id={} 成功", user.id);

        // 查询不存在的用户
        let not_found = service.find_by_id(99999).await.unwrap();
        assert!(not_found.is_none());
        println!("[INFO] 查询不存在的用户返回 None");

        println!("[TEST] 完成测试：根据ID查询用户");
    }
}
