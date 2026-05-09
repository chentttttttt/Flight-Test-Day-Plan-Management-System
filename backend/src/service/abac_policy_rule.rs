use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult,
    IntoActiveModel, Order, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::Utc;

use crate::entity::abac_policy_rule;
use crate::utils::db::{DbError, DbResult};

// ======================== DTO 定义 ========================

/// ABAC 规则更新 DTO（支持部分更新）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbacPolicyRuleUpdateDto {
    pub rule_code: Option<String>,
    pub rule_name: Option<String>,
    pub subject_type: Option<String>,
    pub subject_id: Option<i64>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub action: Option<String>,
    pub condition_json: Option<serde_json::Value>,
    pub effect: Option<String>,
    pub priority: Option<i32>,
    pub status: Option<i16>,
}

// ======================== 服务实现 ========================

#[derive(Clone)]
pub struct AbacPolicyRuleService(pub DatabaseConnection);

impl AbacPolicyRuleService {
    /// 创建规则
    pub async fn create(&self, data: abac_policy_rule::Model) -> DbResult<abac_policy_rule::Model> {
        let mut model = data.into_active_model();
        // 自增主键，必须 unset，让数据库自动生成
        model.id = sea_orm::ActiveValue::not_set();
        model.create_time = Set(Option::from(Utc::now().naive_utc()));
        model.update_time = Set(Option::from(Utc::now().naive_utc()));
        model.deleted = Set(0);
        let res = model.insert(&self.0).await?;
        Ok(res)
    }

    /// 查询适用规则（最匹配的一条）
    pub async fn find_rules(
        &self,
        subject_type: &str,
        subject_id: Option<i64>,
        resource_type: &str,
        resource_id: Option<i64>,
        action: &str,
    ) -> DbResult<Option<abac_policy_rule::Model>> {
        // 基础条件：类型必须完全匹配
        let base_cond = Condition::all()
            .add(abac_policy_rule::Column::SubjectType.eq(subject_type))
            .add(abac_policy_rule::Column::ResourceType.eq(resource_type))
            .add(abac_policy_rule::Column::Action.eq(action))
            .add(abac_policy_rule::Column::Deleted.eq(0));

        let mut query = abac_policy_rule::Entity::find().filter(base_cond);

        // 处理 subject_id：有ID则优先匹配ID或NULL
        query = match subject_id {
            Some(id) => query.filter(
                Condition::any()
                    .add(abac_policy_rule::Column::SubjectId.eq(id))
                    .add(abac_policy_rule::Column::SubjectId.is_null()),
            ),
            None => query.filter(abac_policy_rule::Column::SubjectId.is_null()),
        };

        // 处理 resource_id
        query = match resource_id {
            Some(id) => query.filter(
                Condition::any()
                    .add(abac_policy_rule::Column::ResourceId.eq(id))
                    .add(abac_policy_rule::Column::ResourceId.is_null()),
            ),
            None => query.filter(abac_policy_rule::Column::ResourceId.is_null()),
        };

        // 排序：有ID的排在前面（NULL值在排序时通常排在最后，使用 DESC 可将非NULL值排前面）
        query = query
            .order_by(abac_policy_rule::Column::SubjectId, Order::Desc)
            .order_by(abac_policy_rule::Column::ResourceId, Order::Desc)
            .order_by(abac_policy_rule::Column::Priority, Order::Desc); // 优先级作为第三排序

        Ok(query.one(&self.0).await?)
    }

    /// 查询所有未删除的规则（按优先级倒序）
    pub async fn find_all(&self) -> DbResult<Vec<abac_policy_rule::Model>> {
        abac_policy_rule::Entity::find()
            .filter(abac_policy_rule::Column::Deleted.eq(0))
            .order_by(abac_policy_rule::Column::Priority, Order::Desc)
            .all(&self.0)
            .await
            .map_err(Into::into)
    }

    /// 按条件查询规则（支持 subject_type, resource_type, action, status 筛选）
    pub async fn find_by_condition(
        &self,
        subject_type: Option<String>,
        resource_type: Option<String>,
        action: Option<String>,
        status: Option<i16>,
    ) -> DbResult<Vec<abac_policy_rule::Model>> {
        let mut query = abac_policy_rule::Entity::find()
            .filter(abac_policy_rule::Column::Deleted.eq(0));

        if let Some(st) = subject_type {
            query = query.filter(abac_policy_rule::Column::SubjectType.eq(st));
        }
        if let Some(rt) = resource_type {
            query = query.filter(abac_policy_rule::Column::ResourceType.eq(rt));
        }
        if let Some(act) = action {
            query = query.filter(abac_policy_rule::Column::Action.eq(act));
        }
        if let Some(st) = status {
            query = query.filter(abac_policy_rule::Column::Status.eq(st));
        }

        query
            .order_by(abac_policy_rule::Column::Priority, Order::Desc)
            .all(&self.0)
            .await
            .map_err(Into::into)
    }

    /// 更新规则（支持部分更新）
    pub async fn update(
        &self,
        id: i64,
        dto: AbacPolicyRuleUpdateDto,
    ) -> DbResult<abac_policy_rule::Model> {
        let existing = abac_policy_rule::Entity::find_by_id(id)
            .filter(abac_policy_rule::Column::Deleted.eq(0))
            .one(&self.0)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active_model = existing.into_active_model();

        if let Some(val) = dto.rule_code {
            active_model.rule_code = Set(val);
        }
        if let Some(val) = dto.rule_name {
            active_model.rule_name = Set(val);
        }
        if let Some(val) = dto.subject_type {
            active_model.subject_type = Set(val);
        }
        if let Some(val) = dto.subject_id {
            active_model.subject_id = Set(Some(val));
        }
        if let Some(val) = dto.resource_type {
            active_model.resource_type = Set(val);
        }
        if let Some(val) = dto.resource_id {
            active_model.resource_id = Set(Some(val));
        }
        if let Some(val) = dto.action {
            active_model.action = Set(val);
        }
        if let Some(val) = dto.condition_json {
            active_model.condition_json = Set(Option::from(val));
        }
        if let Some(val) = dto.effect {
            active_model.effect = Set(val);
        }
        if let Some(val) = dto.priority {
            active_model.priority = Set(val);
        }
        if let Some(val) = dto.status {
            active_model.status = Set(val);
        }

        active_model.update_time = Set(Option::from(Utc::now().naive_utc()));
        let updated = active_model.update(&self.0).await?;
        Ok(updated)
    }

    /// 软删除规则
    pub async fn delete(&self, id: i64) -> DbResult<()> {
        let existing = abac_policy_rule::Entity::find_by_id(id)
            .filter(abac_policy_rule::Column::Deleted.eq(0))
            .one(&self.0)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active_model = existing.into_active_model();
        active_model.deleted = Set(1);
        active_model.update_time = Set(Option::from(Utc::now().naive_utc()));
        active_model.update(&self.0).await?;
        Ok(())
    }
}

// ======================== 单元测试 ========================

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sea_orm::{Database, DatabaseConnection};
    use serial_test::serial;
    use serde_json::json;
    use std::env;

    async fn get_db() -> DatabaseConnection {
        dotenv().ok();
        let url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL not set");
        Database::connect(&url).await.unwrap()
    }

    async fn service() -> AbacPolicyRuleService {
        AbacPolicyRuleService(get_db().await)
    }

    async fn clean_db(db: &DatabaseConnection) {
        let _ = abac_policy_rule::Entity::delete_many()
            .exec(db)
            .await;
    }

    /// 辅助函数：创建测试规则
    async fn create_test_rule(
        svc: &AbacPolicyRuleService,
        code: &str,
        subject_type: &str,
        resource_type: &str,
        action: &str,
        priority: i32,
    ) -> abac_policy_rule::Model {
        let model = abac_policy_rule::Model {
            id: 0,
            rule_code: code.to_string(),
            rule_name: format!("测试规则-{}", code),
            subject_type: subject_type.to_string(),
            subject_id: None,
            resource_type: resource_type.to_string(),
            resource_id: None,
            action: action.to_string(),
            condition_json: json!({}),
            effect: "ALLOW".to_string(),
            priority,
            status: 1,
            deleted: 0,
            create_time: Default::default(),
            update_time: Default::default(),
        };
        svc.create(model).await.unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_create_rule() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let model = abac_policy_rule::Model {
            id: 0,
            rule_code: "TEST_RULE".into(),
            rule_name: "测试规则".into(),
            subject_type: "PILOT".into(),
            subject_id: None,
            resource_type: "FLIGHT_TEST_PLAN".into(),
            resource_id: None,
            action: "FLY".into(),
            condition_json: json!({
                "requiredLicense": "A",
                "minFlightHours": 500
            }),
            effect: "ALLOW".into(),
            priority: 10,
            status: 1,
            deleted: 0,
            create_time: Default::default(),
            update_time: Default::default(),
        };

        let res = svc.create(model).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_find_rules() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let model = abac_policy_rule::Model {
            id: 0,
            rule_code: "TEST_FIND".into(),
            rule_name: "测试查询".into(),
            subject_type: "PILOT".into(),
            subject_id: None,
            resource_type: "PLAN".into(),
            resource_id: None,
            action: "EDIT".into(),
            condition_json: json!({}),
            effect: "ALLOW".into(),
            priority: 10,
            status: 1,
            deleted: 0,
            create_time: Default::default(),
            update_time: Default::default(),
        };
        svc.create(model).await.unwrap();

        let rule = svc.find_rules("PILOT", None, "PLAN", None, "EDIT").await.unwrap();
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().rule_code, "TEST_FIND");
    }

    #[tokio::test]
    #[serial]
    async fn test_find_all() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let rule1 = create_test_rule(&svc, "RULE1", "PILOT", "PLAN", "FLY", 10).await;
        let rule2 = create_test_rule(&svc, "RULE2", "PILOT", "PLAN", "VIEW", 5).await;

        let all = svc.find_all().await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, rule1.id);
        assert_eq!(all[1].id, rule2.id);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_by_condition() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let rule1 = create_test_rule(&svc, "RULE1", "PILOT", "PLAN", "FLY", 10).await;
        let rule2 = create_test_rule(&svc, "RULE2", "PILOT", "PLAN", "VIEW", 5).await;
        let rule3 = create_test_rule(&svc, "RULE3", "MANAGER", "REPORT", "VIEW", 8).await;

        let pilot_rules = svc.find_by_condition(Some("PILOT"), None, None, None).await.unwrap();
        assert_eq!(pilot_rules.len(), 2);
        let plan_rules = svc.find_by_condition(None, Some("PLAN"), None, None).await.unwrap();
        assert_eq!(plan_rules.len(), 2);
        let view_rules = svc.find_by_condition(None, None, Some("VIEW"), None).await.unwrap();
        assert_eq!(view_rules.len(), 2);
        let pilot_view = svc.find_by_condition(Some("PILOT"), None, Some("VIEW"), None).await.unwrap();
        assert_eq!(pilot_view.len(), 1);
        assert_eq!(pilot_view[0].id, rule2.id);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_rule() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let rule = create_test_rule(&svc, "UPDATE_TEST", "PILOT", "PLAN", "FLY", 10).await;

        let update_dto = AbacPolicyRuleUpdateDto {
            rule_name: Some("新规则名".to_string()),
            priority: Some(20),
            status: Some(0),
            ..Default::default()
        };

        let updated = svc.update(rule.id, update_dto).await.unwrap();
        assert_eq!(updated.rule_name, "新规则名");
        assert_eq!(updated.priority, 20);
        assert_eq!(updated.status, 0);
        // 未修改字段保持不变
        assert_eq!(updated.subject_type, "PILOT");
        assert_eq!(updated.action, "FLY");

        // 验证数据库
        let from_db = svc.find_all().await.unwrap().pop().unwrap();
        assert_eq!(from_db.rule_name, "新规则名");
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_rule() {
        let svc = service().await;
        clean_db(&svc.0).await;

        let rule = create_test_rule(&svc, "DEL_TEST", "USER", "TEST", "VIEW", 1).await;

        assert!(svc.delete(rule.id).await.is_ok());

        // 验证软删除：通过 find_all 查不到，但直接查（不过滤 deleted）能查到
        let all = svc.find_all().await.unwrap();
        assert_eq!(all.len(), 0);

        let raw = abac_policy_rule::Entity::find_by_id(rule.id)
            .one(&svc.0)
            .await
            .unwrap();
        assert!(raw.is_some());
        assert_eq!(raw.unwrap().deleted, 1);
    }
}