use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, JsonValue};
use chrono::Utc;
use crate::entity::abac_access_log;
use crate::utils::db::{DbResult};

#[derive(Clone)]
pub struct AbacAccessLogService(pub DatabaseConnection);

impl AbacAccessLogService {
    pub async fn log(&self,
                     subject_id: i64,
                     res_id: i64,
                     res_type: &str,
                     action: &str,
                     policy_id: Option<i64>,
                     decision: &str,
                     ip: Option<String>,
                     detail: JsonValue,
    ) -> DbResult<()> {
        abac_access_log::ActiveModel {
            id: Default::default(),
            subject_id: Set(subject_id),
            resource_id: Set(res_id),
            resource_type: Set(res_type.into()),
            action: Set(action.into()),
            policy_id: Set(policy_id),
            decision: Set(decision.into()),
            request_time: Set(Utc::now().naive_utc()),
            request_ip: Set(ip),
            detail: Set(Option::from(detail)),
        }.insert(&self.0).await?;
        Ok(())
    }
}