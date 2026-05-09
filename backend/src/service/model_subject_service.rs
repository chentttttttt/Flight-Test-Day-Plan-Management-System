use sea_orm::*;
use crate::entity::model_subject;
use crate::error::AppError;

#[derive(Clone)]
pub struct ModelSubjectService {
    pub db: DatabaseConnection,
}

impl ModelSubjectService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 创建关联（若已存在则返回错误）
    pub async fn create(
        &self,
        model_id: i64,
        subject_id: i64,
    ) -> Result<model_subject::Model, AppError> {
        // 先检查是否已存在
        let existing = model_subject::Entity::find()
            .filter(model_subject::Column::ModelId.eq(model_id))
            .filter(model_subject::Column::SubjectId.eq(subject_id))
            .one(&self.db)
            .await?;
        if existing.is_some() {
            return Err(AppError::BusinessError("该关联已存在".into()));
        }

        let record = model_subject::ActiveModel {
            model_id: Set(model_id),
            subject_id: Set(subject_id),
        };
        let res = record.insert(&self.db).await?;
        Ok(res)
    }

    /// 删除单个关联
    pub async fn delete(
        &self,
        model_id: i64,
        subject_id: i64,
    ) -> Result<(), AppError> {
        let res = model_subject::Entity::delete_by_id((model_id, subject_id))
            .exec(&self.db)
            .await?;
        if res.rows_affected == 0 {
            return Err(AppError::BusinessError("关联不存在".into()));
        }
        Ok(())
    }

    /// 查询某机型下的所有科目
    pub async fn list_by_model(
        &self,
        model_id: i64,
    ) -> Result<Vec<model_subject::Model>, AppError> {
        let items = model_subject::Entity::find()
            .filter(model_subject::Column::ModelId.eq(model_id))
            .all(&self.db)
            .await?;
        Ok(items)
    }

    /// 查询某科目关联的所有机型
    pub async fn list_by_subject(
        &self,
        subject_id: i64,
    ) -> Result<Vec<model_subject::Model>, AppError> {
        let items = model_subject::Entity::find()
            .filter(model_subject::Column::SubjectId.eq(subject_id))
            .all(&self.db)
            .await?;
        Ok(items)
    }

    /// 列出所有关联（可选择性过滤）
    pub async fn list_all(
        &self,
        model_id: Option<i64>,
        subject_id: Option<i64>,
    ) -> Result<Vec<model_subject::Model>, AppError> {
        let mut select = model_subject::Entity::find();
        if let Some(mid) = model_id {
            select = select.filter(model_subject::Column::ModelId.eq(mid));
        }
        if let Some(sid) = subject_id {
            select = select.filter(model_subject::Column::SubjectId.eq(sid));
        }
        let items = select.all(&self.db).await?;
        Ok(items)
    }

    /// 批量删除某机型下的所有科目
    pub async fn delete_by_model(&self, model_id: i64) -> Result<u64, AppError> {
        let res = model_subject::Entity::delete_many()
            .filter(model_subject::Column::ModelId.eq(model_id))
            .exec(&self.db)
            .await?;
        Ok(res.rows_affected)
    }

    /// 批量删除某科目关联的所有机型
    pub async fn delete_by_subject(&self, subject_id: i64) -> Result<u64, AppError> {
        let res = model_subject::Entity::delete_many()
            .filter(model_subject::Column::SubjectId.eq(subject_id))
            .exec(&self.db)
            .await?;
        Ok(res.rows_affected)
    }
}