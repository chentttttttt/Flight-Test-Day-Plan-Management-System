//! 所有业务服务层统一入口
//! 提供服务初始化、模块导出，便于全局依赖注入

pub mod sys_user;

pub mod subject_type;

pub mod task_type;

pub mod abac_subject_attribute;
pub mod abac_resource_attribute;
pub mod abac_policy_rule;
pub mod abac_access_log;
pub mod sys_menu;
pub mod sys_role_menu;

pub mod approval_manage;
pub(crate) mod approval_order;
pub(crate) mod notification;
pub(crate) mod minio;
pub(crate) mod aircraft_service;
pub(crate) mod pilot_model_flight_time;
mod model_subject_service;

// 导出核心结构体（方便外部调用）
pub use sys_user::SysUserService;

pub use subject_type::SubjectTypeService;

pub use task_type::TaskTypeService;

pub use abac_subject_attribute::AbacSubjectAttributeService;
pub use abac_resource_attribute::AbacResourceAttributeService;
pub use abac_access_log::AbacAccessLogService;
pub use sys_menu::SysMenuService;
pub use sys_role_menu::SysRoleMenuService;

use sea_orm::DatabaseConnection;
use crate::service::abac_policy_rule::AbacPolicyRuleService;
use crate::service::aircraft_service::AircraftService;
use crate::service::approval_manage::ApprovalManageService;
use crate::service::approval_order::ApprovalOrderService;
use crate::service::minio::MinioService;
use crate::service::model_subject_service::ModelSubjectService;
use crate::service::notification::NotificationService;
use crate::service::pilot_model_flight_time::PilotModelFlightTimeService;
use crate::utils::minio::MinioConfig;
use crate::utils::ws_manager::WsManager;

/// 服务容器（统一管理所有服务实例）
/// 用于 Actix Web 依赖注入，一次性初始化所有服务
#[derive(Clone)]
pub struct ServiceContainer {
    pub sys_user: SysUserService,


    pub subject_type: SubjectTypeService,

    pub task_type: TaskTypeService,

    pub abac_subject_attribute: AbacSubjectAttributeService,
    pub abac_resource_attr: AbacResourceAttributeService,
    pub abac_access_log: AbacAccessLogService,
    pub resource_attribute:AbacResourceAttributeService,
    pub abac_policy_rule: AbacPolicyRuleService,
    pub sys_menu: SysMenuService,
    pub sys_role_menu: SysRoleMenuService,
    pub ws_manager: WsManager,
    pub approval_manage: ApprovalManageService,
    pub approval_order: ApprovalOrderService,
    pub notification:NotificationService,
    pub minio: MinioService,

    pub aircraft: AircraftService,
    pub pilot_model_flight_time_service: PilotModelFlightTimeService,
    pub model_subject:ModelSubjectService,
}
impl ServiceContainer {
    /// 初始化所有服务（核心入口）
    /// 只需传入数据库连接，自动创建所有服务实例
    pub fn new(db: DatabaseConnection, minio: MinioService) -> Self {
        let ws_manager = WsManager::new();

        Self {
            sys_user: SysUserService::new(db.clone()),

            subject_type: SubjectTypeService::new(db.clone()),

            task_type: TaskTypeService::new(db.clone()),

            abac_subject_attribute: AbacSubjectAttributeService(db.clone()),
            abac_resource_attr: AbacResourceAttributeService(db.clone()),
            abac_access_log: AbacAccessLogService(db.clone()),
            abac_policy_rule: AbacPolicyRuleService(db.clone()),
            resource_attribute: AbacResourceAttributeService(db.clone()),
            sys_menu: SysMenuService(db.clone()),
            sys_role_menu: SysRoleMenuService(db.clone()),
            approval_manage: ApprovalManageService::new(db.clone()),
            approval_order: ApprovalOrderService::new(db.clone(), ws_manager.clone(), minio.clone()),
            notification: NotificationService(db.clone()),
            aircraft: AircraftService::new(db.clone()),
            pilot_model_flight_time_service: PilotModelFlightTimeService::new(db.clone()),
            model_subject:ModelSubjectService::new(db.clone()),
            ws_manager,
            minio,

        }
    }
}

// 导出通用 DTO 类型（可选，若需要跨模块复用）
pub type DbResult<T> = Result<T, crate::utils::db::DbError>;