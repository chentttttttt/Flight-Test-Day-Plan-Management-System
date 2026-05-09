use actix_web::{web, HttpResponse};
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::service::sys_role_menu::{SysRoleMenuCreateDto, SysRoleMenuUpdateDto};
use crate::utils::response::success_response;

// ========== 角色菜单关联管理 ==========
pub async fn create_role_menu(
    services: web::Data<ServiceContainer>,
    dto: web::Json<SysRoleMenuCreateDto>,
) -> AppResult<HttpResponse> {
    let record = services.sys_role_menu.create(dto.into_inner()).await?;
    Ok(success_response(record))
}

pub async fn update_role_menu(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<SysRoleMenuUpdateDto>,
) -> AppResult<HttpResponse> {
    let record = services.sys_role_menu.update(*id, dto.into_inner()).await?;
    Ok(success_response(record))
}

pub async fn delete_role_menu(
    services: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    services.sys_role_menu.delete(*id).await?;
    Ok(success_response(()))
}

pub async fn get_menu_ids_by_role(
    services: web::Data<ServiceContainer>,
    role: web::Path<String>,
) -> AppResult<HttpResponse> {
    let ids = services.sys_role_menu.find_menu_ids_by_role(&role).await?;
    Ok(success_response(ids))
}

pub async fn grant_menus(
    services: web::Data<ServiceContainer>,
    role: web::Path<String>,
    body: web::Json<Vec<i64>>,
) -> AppResult<HttpResponse> {
    let records = services.sys_role_menu.grant_menus_to_role(&role, &body).await?;
    Ok(success_response(records))
}

pub async fn replace_menus(
    services: web::Data<ServiceContainer>,
    role: web::Path<String>,
    body: web::Json<Vec<i64>>,
) -> AppResult<HttpResponse> {
    let records = services.sys_role_menu.replace_menus_for_role(&role, &body).await?;
    Ok(success_response(records))
}

pub async fn get_all_role_menus(
    services: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let list = services.sys_role_menu.find_all().await?;
    Ok(success_response(list))
}