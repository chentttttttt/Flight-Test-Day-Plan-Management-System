pub mod sys_user;
pub mod policy_rule;
pub mod menu;
pub mod role_menu;
pub mod subject_attribute;
pub mod resource_attribute;

pub mod ws;

pub mod approval_manage;
pub mod approval_order;
pub mod notification;
pub mod minio;

pub mod milp_data;
pub(crate) mod aircraft_controller;
pub mod pilot_model_flight_time;
pub mod milp_schedule;
pub mod model_subject_controller;

// 导出所有控制器函数，便于路由注册
pub use sys_user::*;



pub use menu::*;
pub use policy_rule::*;
