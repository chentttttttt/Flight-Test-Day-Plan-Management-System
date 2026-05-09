use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, Set,
};
use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use crate::entity::abac_resource_attribute;
use crate::utils::db::{DbError, DbResult};

/// 资源属性服务
///
/// 管理各类资源（如飞机、试飞条件等）的扩展属性，以 JSON 形式存储。
#[derive(Clone)]
pub struct AbacResourceAttributeService(pub DatabaseConnection);

impl AbacResourceAttributeService {
    /// 设置或更新资源的单个属性（合并到 JSON 对象）
    ///
    /// # 参数
    /// - `resource_id`: 资源ID
    /// - `resource_type`: 资源类型，例如 "AIRCRAFT", "TEST_CONDITION"
    /// - `key`: 属性名
    /// - `value`: 属性值
    pub async fn set_attr(
        &self,
        resource_id: i64,
        resource_type: &str,
        key: &str,
        value: &str,
    ) -> DbResult<()> {
        let exist = abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(old) = exist {
            let mut current_json = old.attr_key_value.clone();
            if let JsonValue::Object(ref mut map) = current_json {
                map.insert(key.to_string(), JsonValue::String(value.to_string()));
            } else {
                current_json = json!({ key: value });
            }
            let mut a = old.into_active_model();
            a.attr_key_value = Set(current_json);
            a.update(&self.0).await?;
        } else {
            let _model = abac_resource_attribute::ActiveModel {
                id: Default::default(),
                resource_id: Set(resource_id),
                resource_type: Set(resource_type.to_string()),
                attr_key_value: Set(json!({ key: value })),
                deleted: Set(0),
                create_time: Set(Utc::now().naive_utc()),
            }
                .insert(&self.0)
                .await?;
        }
        Ok(())
    }

    /// 获取资源的所有属性（JSON 对象）
    pub async fn get_attrs(&self, resource_id: i64, resource_type: &str) -> DbResult<Option<JsonValue>> {
        let record = abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;
        Ok(record.map(|r| r.attr_key_value))
    }

    /// 获取资源的单个属性值
    pub async fn get_attr(
        &self,
        resource_id: i64,
        resource_type: &str,
        key: &str,
    ) -> DbResult<Option<String>> {
        let record = self.get_attrs(resource_id, resource_type).await?;
        if let Some(JsonValue::Object(map)) = record {
            if let Some(JsonValue::String(val)) = map.get(key) {
                return Ok(Some(val.clone()));
            }
        }
        Ok(None)
    }

    /// 删除资源的单个属性（从 JSON 对象中移除）
    pub async fn delete_attr(
        &self,
        resource_id: i64,
        resource_type: &str,
        key: &str,
    ) -> DbResult<bool> {
        let exist = abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(mut record) = exist {
            let mut current_json = record.attr_key_value.clone();
            if let JsonValue::Object(ref mut map) = current_json {
                if map.remove(key).is_some() {
                    let mut a = record.into_active_model();
                    a.attr_key_value = Set(current_json);
                    a.update(&self.0).await?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// 批量设置资源的所有属性（完全替换 JSON 对象）
    pub async fn set_all_attrs(
        &self,
        resource_id: i64,
        resource_type: &str,
        attrs: JsonValue,
    ) -> DbResult<()> {
        if !attrs.is_object() {
            return Err(DbError::InvalidParameter("attrs must be a JSON object".to_string()));
        }

        let exist = abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(old) = exist {
            let mut a = old.into_active_model();
            a.attr_key_value = Set(attrs);
            a.update(&self.0).await?;
        } else {
            let _model = abac_resource_attribute::ActiveModel {
                id: Default::default(),
                resource_id: Set(resource_id),
                resource_type: Set(resource_type.to_string()),
                attr_key_value: Set(attrs),
                deleted: Set(0),
                create_time: Set(Utc::now().naive_utc()),
            }
                .insert(&self.0)
                .await?;
        }
        Ok(())
    }

    /// 删除资源的所有属性
    pub async fn delete_all_attrs(&self, resource_id: i64, resource_type: &str) -> DbResult<bool> {
        let record = abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(mut old) = record {
            let mut a = old.into_active_model();
            a.attr_key_value = sea_orm::Set(serde_json::json!({}));
            a.update(&self.0).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 删除资源（软删除记录）
    pub async fn delete_resource(&self, resource_id: i64, resource_type: &str) -> DbResult<bool> {
        let record = abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await?;

        if let Some(mut old) = record {
            let mut a = old.into_active_model();
            a.deleted = Set(1);
            a.update(&self.0).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 获取指定类型的所有未删除资源属性记录（用于管理）
    pub async fn list_all_attrs_by_type(&self, resource_type: &str) -> DbResult<Vec<abac_resource_attribute::Model>> {
        abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .all(&self.0)
            .await
            .map_err(Into::into)
    }

    /// 获取所有未删除的资源属性记录（不限类型）
    pub async fn list_all_attrs(&self) -> DbResult<Vec<abac_resource_attribute::Model>> {
        abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .all(&self.0)
            .await
            .map_err(Into::into)
    }

    /// 根据资源ID和类型获取记录
    pub async fn find_by_resource(&self, resource_id: i64, resource_type: &str) -> DbResult<Option<abac_resource_attribute::Model>> {
        abac_resource_attribute::Entity::find()
            .filter(abac_resource_attribute::Column::ResourceId.eq(resource_id))
            .filter(abac_resource_attribute::Column::ResourceType.eq(resource_type))
            .filter(abac_resource_attribute::Column::Deleted.eq(0))
            .one(&self.0)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sea_orm::{Database, DatabaseConnection};
    use serial_test::serial;
    use std::env;

    async fn get_db() -> DatabaseConnection {
        dotenv().ok();
        let url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL not set");
        Database::connect(&url).await.unwrap()
    }

    async fn service() -> AbacResourceAttributeService {
        AbacResourceAttributeService(get_db().await)
    }

    async fn clean_db(db: &DatabaseConnection) {
        let _ = abac_resource_attribute::Entity::delete_many().exec(db).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_set_and_get_attr() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let resource_id = 100;
        let resource_type = "AIRCRAFT";
        svc.set_attr(resource_id, resource_type, "model", "Boeing 737").await.unwrap();
        let val = svc.get_attr(resource_id, resource_type, "model").await.unwrap();
        assert_eq!(val, Some("Boeing 737".to_string()));

        // 更新已有属性
        svc.set_attr(resource_id, resource_type, "model", "Airbus A320").await.unwrap();
        let val = svc.get_attr(resource_id, resource_type, "model").await.unwrap();
        assert_eq!(val, Some("Airbus A320".to_string()));

        // 添加新属性
        svc.set_attr(resource_id, resource_type, "range", "4000km").await.unwrap();
        let attrs = svc.get_attrs(resource_id, resource_type).await.unwrap().unwrap();
        assert_eq!(attrs["model"], "Airbus A320");
        assert_eq!(attrs["range"], "4000km");
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_attr() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let resource_id = 101;
        let resource_type = "TEST_CONDITION";
        svc.set_attr(resource_id, resource_type, "temperature", "25C").await.unwrap();
        svc.set_attr(resource_id, resource_type, "humidity", "60%").await.unwrap();

        let deleted = svc.delete_attr(resource_id, resource_type, "temperature").await.unwrap();
        assert!(deleted);

        let temp = svc.get_attr(resource_id, resource_type, "temperature").await.unwrap();
        assert!(temp.is_none());
        let hum = svc.get_attr(resource_id, resource_type, "humidity").await.unwrap();
        assert_eq!(hum, Some("60%".to_string()));

        let deleted = svc.delete_attr(resource_id, resource_type, "nonexist").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_all_attrs() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let resource_id = 102;
        let resource_type = "AIRCRAFT";
        let new_attrs = json!({ "manufacturer": "Boeing", "year": "2023" });
        svc.set_all_attrs(resource_id, resource_type, new_attrs.clone()).await.unwrap();

        let attrs = svc.get_attrs(resource_id, resource_type).await.unwrap().unwrap();
        assert_eq!(attrs, new_attrs);

        let new_attrs2 = json!({ "status": "active" });
        svc.set_all_attrs(resource_id, resource_type, new_attrs2.clone()).await.unwrap();
        let attrs = svc.get_attrs(resource_id, resource_type).await.unwrap().unwrap();
        assert_eq!(attrs, new_attrs2);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_all_attrs() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let resource_id = 103;
        let resource_type = "TEST_CONDITION";
        svc.set_attr(resource_id, resource_type, "a", "1").await.unwrap();
        assert!(svc.get_attrs(resource_id, resource_type).await.unwrap().is_some());

        let deleted = svc.delete_all_attrs(resource_id, resource_type).await.unwrap();
        assert!(deleted);
        assert!(svc.get_attrs(resource_id, resource_type).await.unwrap().is_none());

        let deleted = svc.delete_all_attrs(resource_id, resource_type).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    #[serial]
    async fn test_list_all_attrs_by_type() {
        let svc = service().await;
        clean_db(&svc.0).await;

        svc.set_attr(1, "AIRCRAFT", "a", "1").await.unwrap();
        svc.set_attr(2, "AIRCRAFT", "b", "2").await.unwrap();
        svc.set_attr(3, "TEST_CONDITION", "c", "3").await.unwrap();

        let aircraft_attrs = svc.list_all_attrs_by_type("AIRCRAFT").await.unwrap();
        assert_eq!(aircraft_attrs.len(), 2);

        let all = svc.list_all_attrs().await.unwrap();
        assert_eq!(all.len(), 3);
    }
}