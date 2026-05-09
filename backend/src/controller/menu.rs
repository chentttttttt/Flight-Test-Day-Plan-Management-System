use actix_web::{web, HttpResponse};
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::service::sys_menu::{SysMenuCreateDto, SysMenuUpdateDto};
use crate::service::sys_role_menu::{SysRoleMenuCreateDto, SysRoleMenuUpdateDto};
use crate::utils::response::{success_response};

// ========== 菜单管理 ==========
pub async fn create_menu(
    service: web::Data<ServiceContainer>,
    dto: web::Json<SysMenuCreateDto>,
) -> AppResult<HttpResponse> {
    let menu = service.sys_menu.create(dto.into_inner())
        .await
        .map_err(AppError::from)?;
    Ok(success_response(menu))
}

pub async fn update_menu(
    service: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<SysMenuUpdateDto>,
) -> AppResult<HttpResponse> {
    let menu = service.sys_menu.update(*id, dto.into_inner())
        .await
        .map_err(AppError::from)?;
    Ok(success_response(menu))
}

pub async fn delete_menu(
    service: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    service.sys_menu.delete(*id).await.map_err(AppError::from)?;
    Ok(success_response(()))
}

pub async fn menu_tree(
    service: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let tree = service.sys_menu.tree().await.map_err(AppError::from)?;
    Ok(success_response(tree))
}

pub async fn menu_list(
    service: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let list = service.sys_menu.list_all().await.map_err(AppError::from)?;
    Ok(success_response(list))
}

pub async fn menu_tree_by_role(
    service: web::Data<ServiceContainer>,
    role_code: web::Path<String>,
) -> AppResult<HttpResponse> {
    let tree = service.sys_menu.get_menu_tree_by_role(&role_code)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(tree))
}

pub async fn menu_list_by_role(
    service: web::Data<ServiceContainer>,
    role_code: web::Path<String>,
) -> AppResult<HttpResponse> {
    let list = service.sys_menu.get_menus_by_role(&role_code)
        .await
        .map_err(AppError::from)?;
    Ok(success_response(list))
}
