use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait, IntoActiveModel, Condition};
use chrono::Utc;
use jwt::Error::Json;
use serde_json::{json, Value};
use crate::entity::abac_subject_attribute;
use crate::entity::sys_menu::Model;
use crate::utils::db::{DbResult, DbError};

/// 通用主体属性服务
/// 用于为任意主体（人员/飞机/机型/设备等）动态拓展属性
/// 无需修改表结构，支持无限字段扩展
#[derive(Clone)]
pub struct AbacSubjectAttributeService(pub DatabaseConnection);

impl AbacSubjectAttributeService {
    // =========================================================================
    // 设置/更新单个属性
    // 自动合并，不覆盖原有属性
    // =========================================================================
    pub async fn set_attr(
        &self,
        subject_id: i64,
        subject_type: &str,
        key: &str,
        value: Value
    ) -> DbResult<()> {
        // 查询是否已存在
        let exist = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectId.eq(subject_id))
            .filter(abac_subject_attribute::Column::SubjectType.eq(subject_type))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(old) = exist {
            // 合并 JSON：保留原有字段，只更新/新增当前 key
            let mut attrs = old.attr_key_value.clone();
            attrs[key] = value;

            let mut active_model = old.into_active_model();
            active_model.attr_key_value = Set(attrs);
            active_model.update(&self.0).await?;
        } else {
            // 新建记录
            let new_attr = json!({ key: value });
            abac_subject_attribute::ActiveModel {
                id: Default::default(),
                subject_id: Set(subject_id),
                subject_type: Set(subject_type.to_string()),
                attr_key_value: Set(new_attr),
                deleted: Set(0),
                create_time: Set(Utc::now().naive_utc()),
            }.insert(&self.0).await?;
        }

        Ok(())
    }

    // =========================================================================
    // 获取主体类型
    // =========================================================================
    pub async fn get_type(
        &self,
        subject_id: i64,
    ) -> DbResult<String> {
        let record = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectId.eq(subject_id))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        Ok(record.unwrap().subject_type)
    }

    // =========================================================================
    // 获取单个属性
    // =========================================================================
    pub async fn get_attr(
        &self,
        subject_id: i64,
        subject_type: &str,
        key: &str
    ) -> DbResult<Option<Value>> {
        let record = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectId.eq(subject_id))
            .filter(abac_subject_attribute::Column::SubjectType.eq(subject_type))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        Ok(record.and_then(|v| v.attr_key_value.get(key).cloned()))
    }

    pub async fn delete_attr(&self, id: i64) -> DbResult<bool> {
        let record = abac_subject_attribute::Entity::find()
            .filter(
                Condition::all()
                    .add(abac_subject_attribute::Column::Id.eq(id))
                    .add(abac_subject_attribute::Column::Deleted.eq(0))
            )
            .one(&self.0)
            .await?;

        let Some(record) = record else {
            return Ok(false);
        };

        let mut model: abac_subject_attribute::ActiveModel = record.into();
        model.deleted = Set(1);
        model.update(&self.0).await?;

        Ok(true)
    }

    // =========================================================================
    // 获取全部属性（完整JSON）
    // =========================================================================
    pub async fn get_all_attr(
        &self,
        subject_id: i64,
        subject_type: &str
    ) -> DbResult<Value> {
        let record = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectId.eq(subject_id))
            .filter(abac_subject_attribute::Column::SubjectType.eq(subject_type))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        Ok(record.map(|v| v.attr_key_value).unwrap_or_default())
    }

    // =========================================================================
    // 删除单个属性
    // =========================================================================
    pub async fn remove_attr(
        &self,
        subject_id: i64,
        subject_type: &str,
        key: &str
    ) -> DbResult<()> {
        let record = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectId.eq(subject_id))
            .filter(abac_subject_attribute::Column::SubjectType.eq(subject_type))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut attrs = record.attr_key_value.clone();
        attrs.as_object_mut()
            .ok_or(DbError::InvalidParameter("属性格式错误".into()))?
            .remove(key);

        let mut active_model = record.into_active_model();
        active_model.attr_key_value = Set(attrs);
        active_model.update(&self.0).await?;

        Ok(())
    }

    // =========================================================================
    // 批量设置多个属性
    // =========================================================================
    pub async fn set_attr_batch(
        &self,
        subject_id: i64,
        subject_type: &str,
        attrs: Value
    ) -> DbResult<()> {
        if !attrs.is_object() {
            return Err(DbError::InvalidParameter("属性必须是对象格式".into()));
        }

        let exist = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectId.eq(subject_id))
            .filter(abac_subject_attribute::Column::SubjectType.eq(subject_type))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(old) = exist {
            let mut target = old.attr_key_value.clone();
            for (k, v) in attrs.as_object().unwrap() {
                target[k] = v.clone();
            }

            let mut active_model = old.into_active_model();
            active_model.attr_key_value = Set(target);
            active_model.update(&self.0).await?;
        } else {
            abac_subject_attribute::ActiveModel {
                id: Default::default(),
                subject_id: Set(subject_id),
                subject_type: Set(subject_type.to_string()),
                attr_key_value: Set(attrs),
                deleted: Set(0),
                create_time: Set(Utc::now().naive_utc()),
            }.insert(&self.0).await?;
        }

        Ok(())
    }

    /// 获取指定主体类型的所有主体属性
    pub async fn get_all_by_subject_type(
        &self,
        subject_type: &str,
    ) -> DbResult<Vec<abac_subject_attribute::Model>> {
        let records = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::SubjectType.eq(subject_type))
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .all(&self.0)
            .await?;
        Ok(records)
    }

    /// 获取指定主体类型的所有主体属性
    pub async fn get_all(
        &self,
    ) -> DbResult<Vec<abac_subject_attribute::Model>> {
        let records = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::Deleted.eq(0))
            .all(&self.0)
            .await?;
        Ok(records)
    }

    // =========================================================================
    // 物理删除该主体的所有属性
    // =========================================================================
    pub async fn delete_all_attr(
        &self,
        id: i64,
    ) -> DbResult<()> {
        // 1. 查询要清空属性的记录
        let mut subject = abac_subject_attribute::Entity::find()
            .filter(abac_subject_attribute::Column::Id.eq(id))
            .one(&self.0)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Subject attribute not found".into()))?
            .into_active_model();

        // 2. 清空属性为 空 JSON 对象
        subject.attr_key_value = sea_orm::Set(serde_json::json!({}));

        // 3. 更新到数据库
        subject.update(&self.0).await?;

        Ok(())
    }
}

// ==============================
// 测试
// ==============================
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
    use serial_test::serial;
    use serde_json::json;
    use std::env;

    async fn get_test_db() -> DatabaseConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_TEST_URL").unwrap();
        let db = Database::connect(&database_url).await.unwrap();
        db.execute_unprepared("TRUNCATE TABLE abac_subject_attribute RESTART IDENTITY CASCADE;").await.unwrap();
        db
    }

    async fn create_service() -> AbacSubjectAttributeService {
        AbacSubjectAttributeService(get_test_db().await)
    }

    #[tokio::test] #[serial] async fn test_set_attr() {
        let s = create_service().await;
        assert!(s.set_attr(1, "TEST", "name", json!("test")).await.is_ok());
    }

    #[tokio::test] #[serial] async fn test_get_attr() {
        let s = create_service().await;
        s.set_attr(1, "TEST", "name", json!("a")).await.unwrap();
        assert_eq!(s.get_attr(1, "TEST", "name").await.unwrap(), Some(json!("a")));
    }

    #[tokio::test] #[serial] async fn test_get_all_attr() {
        let s = create_service().await;
        s.set_attr(1, "TEST", "name", json!("a")).await.unwrap();
        s.set_attr(1, "TEST", "age", json!(18)).await.unwrap();
        let all = s.get_all_attr(1, "TEST").await.unwrap();
        assert_eq!(all["name"], "a");
    }

    #[tokio::test] #[serial] async fn test_set_attr_batch() {
        let s = create_service().await;
        let attrs = json!({ "role": "admin", "status": 1 });
        assert!(s.set_attr_batch(1, "TEST", attrs).await.is_ok());
    }

    #[tokio::test] #[serial] async fn test_remove_attr() {
        let s = create_service().await;
        s.set_attr(1, "TEST", "name", json!("a")).await.unwrap();
        s.remove_attr(1, "TEST", "name").await.unwrap();
        assert!(s.get_attr(1, "TEST", "name").await.unwrap().is_none());
    }

    #[tokio::test] #[serial] async fn test_delete_all_attr() {
        let s = create_service().await;
        s.set_attr(1, "TEST", "name", json!("a")).await.unwrap();
        s.delete_all_attr(1, "TEST").await.unwrap();

        let all = s.get_all_attr(1, "TEST").await.unwrap();
        assert!(all.is_null() || all.as_object().map(|o| o.is_empty()).unwrap_or(true));
    }
}