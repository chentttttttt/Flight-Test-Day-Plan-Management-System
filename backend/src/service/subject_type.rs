use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, PaginatorTrait, Order, QueryOrder, IntoActiveModel, QuerySelect};
use serde::{Deserialize, Serialize};

use crate::entity::subject_type;
use crate::utils::db::{DbResult, DbError};

// DTO定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectTypeCreateDto {
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectTypeUpdateDto {
    pub id: i64,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectTypeQueryDto {
    pub type_name: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

// 合并版Service
#[derive(Clone)]
pub struct SubjectTypeService {
    db: DatabaseConnection,
}

impl SubjectTypeService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 创建科目类型
    pub async fn create_type(&self, dto: SubjectTypeCreateDto) -> DbResult<subject_type::Model> {
        // 业务校验
        if dto.type_name.is_empty() {
            return Err(DbError::InvalidParameter("科目类型名称不能为空".to_string()));
        }

        // 数据库操作
        let model = subject_type::ActiveModel {
            id: Default::default(),
            type_name: Set(dto.type_name),
            deleted: Set(0),
        }.insert(&self.db).await?;

        Ok(model)
    }

    /// 更新科目类型
    pub async fn update_type(&self, dto: SubjectTypeUpdateDto) -> DbResult<subject_type::Model> {
        // 查询类型（未删除）
        let mut r#type = subject_type::Entity::find_by_id(dto.id)
            .filter(subject_type::Column::Deleted.eq(0))
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        // 校验名称
        if dto.type_name.is_empty() {
            return Err(DbError::InvalidParameter("科目类型名称不能为空".to_string()));
        }

        // 构建更新模型
        let mut active_model = r#type.into_active_model();
        active_model.type_name = Set(dto.type_name);

        // 执行更新
        let updated_type = active_model.update(&self.db).await?;
        Ok(updated_type)
    }

    /// 分页查询科目类型
    pub async fn query_types(&self, dto: SubjectTypeQueryDto) -> DbResult<(Vec<subject_type::Model>, u64)> {
        let mut query = subject_type::Entity::find().filter(subject_type::Column::Deleted.eq(0));

        // 构建查询条件
        if let Some(type_name) = dto.type_name {
            query = query.filter(subject_type::Column::TypeName.like(format!("%{}%", type_name)));
        }

        // 排序+分页
        query = query.order_by(subject_type::Column::Id, Order::Asc);
        let total = query.clone().count(&self.db).await?;
        let types = query
            .offset(((dto.page - 1) * dto.page_size) as u64)
            .limit(dto.page_size as u64)
            .all(&self.db)
            .await?;

        Ok((types, total))
    }

    /// 软删除科目类型
    pub async fn delete_type(&self, id: i64) -> DbResult<bool> {
        let mut r#type = subject_type::Entity::find_by_id(id)
            .filter(subject_type::Column::Deleted.eq(0))
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active_model = r#type.into_active_model();
        active_model.deleted = Set(1);

        let result = active_model.update(&self.db).await?;
        Ok(result.deleted == 1)
    }

    /// 根据ID查询科目类型
    pub async fn find_by_id(&self, id: i64) -> DbResult<Option<subject_type::Model>> {
        let r#type = subject_type::Entity::find_by_id(id)
            .filter(subject_type::Column::Deleted.eq(0))
            .one(&self.db)
            .await?;

        Ok(r#type)
    }
}