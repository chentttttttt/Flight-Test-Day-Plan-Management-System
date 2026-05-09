use sea_orm::*;
use chrono::Utc;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use crate::entity::pilot_model_flight_time;
use crate::error::AppError;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePilotModelFlightTimeDto {
    pub pilot_id: i64,
    pub model_id: i64,
    pub total_flight_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePilotModelFlightTimeDto {
    pub total_flight_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListQuery {
    pub pilot_id: Option<i64>,
    pub model_id: Option<i64>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
#[derive(Clone)]
pub struct PilotModelFlightTimeService {
    pub db: DatabaseConnection,
}

impl PilotModelFlightTimeService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // ========== 新增 ==========
    pub async fn create(
        &self,
        dto: CreatePilotModelFlightTimeDto,
    ) -> Result<pilot_model_flight_time::Model, AppError> {
        let now = Utc::now();
        let record = pilot_model_flight_time::ActiveModel {
            pilot_id: Set(dto.pilot_id),
            model_id: Set(dto.model_id),
            total_flight_hours: Set(dto.total_flight_hours),
            create_time: Set(DateTimeWithTimeZone::from(now)),
            update_time: Set(DateTimeWithTimeZone::from(now)),
            ..Default::default()
        };
        let res = record.insert(&self.db).await?;
        Ok(res)
    }

    // ========== 根据 ID 查询单个 ==========
    pub async fn get_by_id(
        &self,
        id: i64,
    ) -> Result<Option<pilot_model_flight_time::Model>, AppError> {
        let record = pilot_model_flight_time::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        Ok(record)
    }

    // ========== 根据 pilot_id + model_id 查询 ==========
    pub async fn get_by_pilot_and_model(
        &self,
        pilot_id: i64,
        model_id: i64,
    ) -> Result<Option<pilot_model_flight_time::Model>, AppError> {
        let record = pilot_model_flight_time::Entity::find()
            .filter(pilot_model_flight_time::Column::PilotId.eq(pilot_id))
            .filter(pilot_model_flight_time::Column::ModelId.eq(model_id))
            .one(&self.db)
            .await?;
        Ok(record)
    }

    // ========== 列表查询（支持过滤与分页） ==========
    pub async fn list(&self, query: ListQuery) -> Result<Vec<pilot_model_flight_time::Model>, AppError> {
        let mut select = pilot_model_flight_time::Entity::find();
        // if let Some(pilot_id) = query.pilot_id {
        //     select = select.filter(pilot_model_flight_time::Column::PilotId.eq(pilot_id));
        // }
        // if let Some(model_id) = query.model_id {
        //     select = select.filter(pilot_model_flight_time::Column::ModelId.eq(model_id));
        // }
        let items = select.all(&self.db).await?;
        Ok(items)
    }

    // ========== 更新（根据ID） ==========
    pub async fn update(
        &self,
        id: i64,
        dto: UpdatePilotModelFlightTimeDto,
    ) -> Result<pilot_model_flight_time::Model, AppError> {
        let record = pilot_model_flight_time::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::BusinessError("记录不存在".into()))?;

        let mut active: pilot_model_flight_time::ActiveModel = record.into_active_model();
        active.total_flight_hours = Set(dto.total_flight_hours);
        active.update_time = Set(DateTimeWithTimeZone::from(Utc::now()));
        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    // ========== Upsert：存在则累加并更新，否则插入 ==========
    pub async fn upsert(
        &self,
        dto: CreatePilotModelFlightTimeDto,
    ) -> Result<pilot_model_flight_time::Model, AppError> {
        if let Some(existing) = self.get_by_pilot_and_model(dto.pilot_id, dto.model_id).await? {
            // 累加飞行时间并更新时间
            let new_total = existing.total_flight_hours + dto.total_flight_hours;
            let mut active: pilot_model_flight_time::ActiveModel = existing.into_active_model();
            active.total_flight_hours = Set(new_total);
            active.update_time = Set(DateTimeWithTimeZone::from(Utc::now()));
            let updated = active.update(&self.db).await?;
            Ok(updated)
        } else {
            self.create(dto).await
        }
    }

    // ========== 根据 ID 删除 ==========
    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let res = pilot_model_flight_time::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        if res.rows_affected == 0 {
            return Err(AppError::BusinessError("记录不存在".into()));
        }
        Ok(())
    }

    // ========== 根据 pilot_id 批量删除 ==========
    pub async fn delete_by_pilot(&self, pilot_id: i64) -> Result<u64, AppError> {
        let res = pilot_model_flight_time::Entity::delete_many()
            .filter(pilot_model_flight_time::Column::PilotId.eq(pilot_id))
            .exec(&self.db)
            .await?;
        Ok(res.rows_affected)
    }
}