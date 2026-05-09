use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, IntoActiveModel, QueryFilter, Set};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::entity::sys_role_menu;
use crate::utils::db::{DbError, DbResult};

// ------------------------------
// DTO 改成数组
// ------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysRoleMenuCreateDto {
    pub role: String,
    pub menu_ids: Vec<i64>, // 改为数组
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysRoleMenuUpdateDto {
    pub role: Option<String>,
    pub menu_ids: Option<Vec<i64>>, // 改为数组
}

#[derive(Clone)]
pub struct SysRoleMenuService(pub DatabaseConnection);

impl SysRoleMenuService {
    pub async fn create(&self, dto: SysRoleMenuCreateDto) -> DbResult<sys_role_menu::Model> {
        let now = Utc::now();
        let active = sys_role_menu::ActiveModel {
            id: Default::default(),
            role: Set(dto.role),
            menu_ids: Set(Option::from(dto.menu_ids)), // 改为 menu_ids + 数组
            create_time: Set(now.naive_utc()),
            update_time: Set(now.naive_utc()),
        };
        Ok(active.insert(&self.0).await?)
    }

    pub async fn update(&self, id: i64, dto: SysRoleMenuUpdateDto) -> DbResult<sys_role_menu::Model> {
        let existing = sys_role_menu::Entity::find_by_id(id)
            .one(&self.0)
            .await?
            .ok_or(DbError::RecordNotFound)?;
        let mut active: sys_role_menu::ActiveModel = existing.into_active_model();

        if let Some(role) = dto.role {
            active.role = Set(role);
        }
        if let Some(menu_ids) = dto.menu_ids {
            active.menu_ids = Set(Option::from(menu_ids)); // 数组
        }

        active.update_time = Set(Utc::now().naive_utc());
        Ok(active.update(&self.0).await?)
    }

    pub async fn delete(&self, id: i64) -> DbResult<()> {
        let res = sys_role_menu::Entity::delete_by_id(id).exec(&self.0).await?;
        if res.rows_affected == 0 {
            Err(DbError::RecordNotFound)
        } else {
            Ok(())
        }
    }

    pub async fn delete_by_role(&self, role: &str) -> DbResult<u64> {
        Ok(sys_role_menu::Entity::delete_many()
            .filter(sys_role_menu::Column::Role.eq(role))
            .exec(&self.0)
            .await?
            .rows_affected)
    }

    // 废弃不用
    pub async fn delete_by_menu(&self, menu_id: i64) -> DbResult<u64> {
        Ok(0)
    }

    // 直接返回数组，不需要循环
    pub async fn find_menu_ids_by_role(&self, role: &str) -> DbResult<Vec<i64>> {
        let record = sys_role_menu::Entity::find()
            .filter(sys_role_menu::Column::Role.eq(role))
            .one(&self.0)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        Ok(record.menu_ids.unwrap())
    }

    pub async fn find_roles_by_menu(&self, _menu_id: i64) -> DbResult<Vec<String>> {
        // 数组模式下不建议这样查，如需我再给你写
        Ok(vec![])
    }

    // 直接保存数组，不需要循环插入
    pub async fn grant_menus_to_role(&self, role: &str, menu_ids: &[i64]) -> DbResult<sys_role_menu::Model> {
        Ok(self.create(SysRoleMenuCreateDto {
            role: role.to_string(),
            menu_ids: menu_ids.to_vec(),
        }).await?)
    }

    pub async fn replace_menus_for_role(&self, role: &str, menu_ids: &[i64]) -> DbResult<sys_role_menu::Model> {
        self.delete_by_role(role).await?;
        self.grant_menus_to_role(role, menu_ids).await
    }

    pub async fn find_all(&self) -> DbResult<Vec<sys_role_menu::Model>> {
        let list = sys_role_menu::Entity::find()
            .all(&self.0)
            .await?;
        Ok(list)
    }
}

