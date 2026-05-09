//! 路由注册统一管理
//! 所有 API 路由集中在此配置，便于维护

use actix_web::web::ServiceConfig;
use actix_web::{Scope, web};
use chrono;
use serde_json;

// 导入所有控制器函数
use crate::controller;
use crate::controller::ws::websocket_handler;
use crate::middleware::perm::*;
use crate::controller::aircraft_controller;


/// 注册所有路由（核心入口）
pub fn configure_routes(cfg: &mut ServiceConfig) {
    // ========== 系统用户模块 ==========
    cfg.service(
        web::scope("/api/user")
            .route("/info", web::get().to(controller::sys_user::get_current_user))
            .route("/{id}", web::get().to(controller::sys_user::get_user))
            .route("/login", web::post().to(controller::sys_user::login))
            .route("/create", web::post().to(controller::sys_user::create_user))
            .route("/update", web::put().to(controller::sys_user::update_user))
            .route("/update-self", web::put().to(controller::sys_user::update_self))
            .route("/list", web::post().to(controller::sys_user::list_users))
            .route("/delete/{id}", web::delete().to(controller::sys_user::delete_user))
    );

    cfg.service(
        web::scope("/api/menu")
            .route("", web::post().to(controller::menu::create_menu))
            .route("/{id}", web::put().to(controller::menu::update_menu))
            .route("/{id}", web::delete().to(controller::menu::delete_menu))
            .route("/tree", web::get().to(controller::menu::menu_tree))
            .route("/list", web::get().to(controller::menu::menu_list))
            .route("/role/tree/{role_code}", web::get().to(controller::menu_tree_by_role))
            .route("/role/list/{role_code}", web::get().to(controller::menu_list_by_role))
    );

    // ========== 角色菜单关联路由 ==========
    cfg.service(
        web::scope("/api/role")
            // 关联表 CRUD
            .route("", web::post().to(controller::role_menu::create_role_menu))
            // 查询所有
            .route("/list", web::get().to(controller::role_menu::get_all_role_menus))
            .route("/{id}", web::put().to(controller::role_menu::update_role_menu))
            .route("/{id}", web::delete().to(controller::role_menu::delete_role_menu))
            // 查询角色拥有的菜单ID
            .route("/role/menus/{role}", web::get().to(controller::role_menu::get_menu_ids_by_role))
            // 批量授权菜单给角色（添加）
            .route("/role/grant/{role}", web::post().to(controller::role_menu::grant_menus))
            // 替换角色的菜单权限（先删除再添加）
            .route("/role/replace/{role}", web::post().to(controller::role_menu::replace_menus))
    );

    cfg.service(
        web::scope("/api/rule")
            .route("", web::post().to(controller::policy_rule::create_rule)) // 创建规则
            .route("/list", web::get().to(controller::policy_rule::list_all_rules)) // 查询所有规则
            .route("/query", web::get().to(controller::policy_rule::query_rules)) // 条件查询规则
            .route("/{id}", web::put().to(controller::policy_rule::update_rule)) // 更新规则
            .route("/{id}", web::delete().to(controller::policy_rule::delete_rule)) // 删除规则
            .route("/find", web::post().to(controller::policy_rule::find_rule))
    );

    cfg.service(
        web::scope("/api/subject-attribute")
            .route("", web::post().to(controller::subject_attribute::set_attributes))
            .route("", web::get().to(controller::subject_attribute::get_all))
            .route("/{subject_type}", web::get().to(controller::subject_attribute::get_all_by_subject_type))
            .route("/{subject_type}/{id}", web::get().to(controller::subject_attribute::get_all_attributes))
            .route("/{id}", web::delete().to(controller::subject_attribute::delete_all_attributes))
            .route("/attr/{id}", web::delete().to(controller::subject_attribute::delete_subject))
            .route("/{subject_type}/{id}/{key}", web::get().to(controller::subject_attribute::get_attribute))
            .route("/{subject_type}/{id}/{key}", web::delete().to(controller::subject_attribute::remove_attribute))
    );


    cfg.service(
        web::scope("/api/resource-attribute")
            // 批量设置（替换整个 JSON）
            .route("", web::post().to(controller::resource_attribute::set_all_attributes))
            // 列出所有记录
            .route("", web::get().to(controller::resource_attribute::list_all))
            // 根据类型列出所有记录
            .route("/type/{resource_type}", web::get().to(controller::resource_attribute::list_by_type))
            // 获取全部属性
            .route("/{resource_type}/{resource_id}", web::get().to(controller::resource_attribute::get_all_attributes))
            // 删除资源
            .route("/res/{resource_type}/{resource_id}", web::delete().to(controller::resource_attribute::delete_resource))
            // 删除全部属性
            .route("/{resource_type}/{resource_id}", web::delete().to(controller::resource_attribute::delete_all_attributes))
            // 获取单个属性
            .route("/{resource_type}/{resource_id}/{key}", web::get().to(controller::resource_attribute::get_attribute))
            // 设置单个属性
            .route("/{resource_type}/{resource_id}/{key}", web::post().to(controller::resource_attribute::set_attribute))
            // 删除单个属性
            .route("/{resource_type}/{resource_id}/{key}", web::delete().to(controller::resource_attribute::remove_attribute))
            // 根据资源ID和类型获取记录（含ID）
            .route("/record/{resource_type}/{resource_id}", web::get().to(controller::resource_attribute::find_by_resource))
    );

    cfg.service(
        web::scope("/api/approval-manage")
            .route("/active", web::get().to(controller::approval_manage::get_active_flow))
            // 树形数据
            .route("/tree", web::get().to(controller::approval_manage::get_flow_tree))
            .route("/tree/{id}", web::get().to(controller::approval_manage::get_flow_tree_by_id))
            // 流程 CRUD
            .route("/flow", web::post().to(controller::approval_manage::create_flow))
            .route("/flow/{id}", web::put().to(controller::approval_manage::update_flow))
            .route("/flow/{id}", web::delete().to(controller::approval_manage::delete_flow))
            .route("/flow/{id}", web::get().to(controller::approval_manage::get_flow))
            // 节点 CRUD
            .route("/node", web::post().to(controller::approval_manage::create_node))
            .route("/node/{id}", web::put().to(controller::approval_manage::update_node))
            .route("/node/{id}", web::delete().to(controller::approval_manage::delete_node))
            .route("/node/{id}", web::get().to(controller::approval_manage::get_node))
    );
    cfg.service(
        web::scope("/api/approval")
            .route("/submit", web::post().to(controller::approval_order::submit_approval))
            .route("/approve", web::post().to(controller::approval_order::approve))
            .route("/reject", web::post().to(controller::approval_order::reject))
            .route("/todo", web::get().to(controller::approval_order::todo_list))
            .route("/my-orders", web::get().to(controller::approval_order::my_orders))
            .route("/records/{order_id}", web::get().to(controller::approval_order::approval_records))
    );


    cfg.service(
        web::scope("/api/notification")
            // 创建通知（需管理员权限）
            .route("", web::post().to(controller::notification::create))
            // 当前用户的通知列表
            .route("/list", web::post().to(controller::notification::list_current_user))
            // 管理员查看指定用户通知列表
            .route("/list/{user_id}", web::post().to(controller::notification::list_by_user_id))
            // 批量标记已读
            .route("/mark-read", web::post().to(controller::notification::mark_read))
            // 删除通知
            .route("/{id}", web::delete().to(controller::notification::delete))
            // 未读数量
            .route("/unread-count", web::get().to(controller::notification::unread_count))
        );
    cfg.service(
        web::scope("/api/minio")
            .route("/list", web::get().to(controller::minio::list_files))
            .route("/upload", web::post().to(controller::minio::upload_file))
            .route("/url/{key}", web::get().to(controller::minio::get_file_url))
            .route("/delete/{key}", web::delete().to(controller::minio::delete_file))

    );

    cfg.service(
        web::scope("/api/milp")
            .route("/schedule", web::post().to(controller::milp_schedule::schedule))
        // ... 其他路由
    );

    // cfg.service(
    //     web::scope("/api")
    //         // WebSocket 路由，需要认证
    //         .route("/ws", web::get().to(websocket_handler))
    // );

    cfg.service(
        web::scope("/api/aircraft")
            .route("/pilots", web::get().to(aircraft_controller::list_pilots))
            .route("/pilot", web::post().to(aircraft_controller::create_pilot))
            .route("/pilot/{id}/models", web::get().to(aircraft_controller::get_pilot_models))
            .route("/pilot/{id}/models", web::post().to(aircraft_controller::set_pilot_models))
            .route("/pilot/{id}", web::get().to(aircraft_controller::get_pilot))
            .route("/pilot/{id}", web::put().to(aircraft_controller::update_pilot))
            .route("/pilot/{id}", web::delete().to(aircraft_controller::delete_pilot))

            // 机型
            .route("/model", web::post().to(aircraft_controller::create_model))
            .route("/model/{id}", web::put().to(aircraft_controller::update_model))
            .route("/model/{id}", web::delete().to(aircraft_controller::delete_model))
            .route("/models", web::get().to(aircraft_controller::list_models))

            // 科目
            .route("/subject", web::post().to(aircraft_controller::create_subject))
            .route("/subject/{id}", web::put().to(aircraft_controller::update_subject))
            .route("/subject/{id}", web::delete().to(aircraft_controller::delete_subject))
            .route("/subjects", web::get().to(aircraft_controller::list_subjects))

            // 飞机
            .route("/finish", web::post().to(aircraft_controller::finish_flight))
            .route("/list", web::get().to(aircraft_controller::list_aircraft))
            .route("/detail/{id}", web::get().to(aircraft_controller::aircraft_detail))
            .route("/incomplete/{aircraft_id}", web::get().to(aircraft_controller::incomplete_subjects))

            // 飞机 - 泛动态路径 放在最后！防止覆盖前面路由
            .route("", web::post().to(aircraft_controller::create_aircraft))
            .route("/{id}", web::put().to(aircraft_controller::update_aircraft))
            .route("/{id}", web::delete().to(aircraft_controller::delete_aircraft))
    );

    cfg.service(
        web::scope("/api/pilot-model-flight-time")
            .route("", web::post().to(controller::pilot_model_flight_time::create))
            .route("/list", web::get().to(controller::pilot_model_flight_time::list))
            .route("/upsert", web::post().to(controller::pilot_model_flight_time::upsert))
            .route("/{id}", web::get().to(controller::pilot_model_flight_time::get_by_id))
            .route("/{id}", web::put().to(controller::pilot_model_flight_time::update))
            .route("/{id}", web::delete().to(controller::pilot_model_flight_time::delete))
            .route("/pilot/{pilot_id}/model/{model_id}", web::get().to(controller::pilot_model_flight_time::get_by_pilot_model))
            .route("/pilot/{pilot_id}", web::delete().to(controller::pilot_model_flight_time::delete_by_pilot))
    );


    cfg.service(
        web::scope("/api/model-subject")
            .route("/{model_id}/{subject_id}", web::post().to(controller::model_subject_controller::create))
            .route("/{model_id}/{subject_id}", web::delete().to(controller::model_subject_controller::delete))
            .route("", web::get().to(controller::model_subject_controller::list))
            .route("/model/{model_id}", web::delete().to(controller::model_subject_controller::delete_by_model))
            .route("/subject/{subject_id}", web::delete().to(controller::model_subject_controller::delete_by_subject))
    );



    // 健康检查接口
    cfg.route("/health", web::get().to(health_check));
}

/// 健康检查接口
async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "flight-system",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
