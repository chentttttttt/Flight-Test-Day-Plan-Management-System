use std::ptr::null;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set, IntoActiveModel, RelationTrait, JoinType, TransactionTrait};
use chrono::Utc;
use rust_decimal::Decimal;
use anyhow::Result as AnyResult;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use crate::entity::{aircraft, aircraft_model, aircraft_subject_completion, model_subject, pilot, pilot_aircraft_model, subject, sys_user};
use crate::error::AppError;
use crate::utils::db::{DbError, DbResult};

// ========== DTOs ==========
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAircraftModelDto {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAircraftModelDto {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubjectDto {
    pub name: String,
    pub code: Option<String>,
    pub danger_level: String,
    pub default_duration: Decimal,
    pub subject_type: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
    pub required_pilots: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubjectDto {
    pub name: Option<String>,
    pub code: Option<String>,
    pub danger_level: Option<String>,
    pub default_duration: Option<Decimal>,
    pub subject_type: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
    pub required_pilots: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAircraftDto {
    pub model_id: i64,
    pub plane_no: String,
    pub status: Option<String>,
    pub remark: Option<String>,
    pub extra_attrs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAircraftDto {
    pub model_id: Option<i64>,
    pub plane_no: Option<String>,
    pub status: Option<String>,
    pub remark: Option<String>,
    pub extra_attrs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartFlightDto {
    pub aircraft_id: i64,
    pub subject_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndFlightDto {
    pub aircraft_id: i64,
    pub subject_id: i64,
    pub flight_hours: Decimal,      // 空中飞行小时
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePilotDto {
    pub user_id: i64,
    pub code: Option<String>,
    pub gender: Option<String>,
    pub age: Option<i32>,
    pub health_status: Option<String>,
    pub mental_status: Option<String>,
    pub level: Option<String>,
    pub remark: Option<String>,
    pub status: Option<i16>,
    pub model_ids: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePilotDto {
    pub user_id: Option<i64>,
    pub code: Option<String>,
    pub gender: Option<String>,
    pub age: Option<i32>,
    pub health_status: Option<String>,
    pub mental_status: Option<String>,
    pub level: Option<String>,
    pub remark: Option<String>,
    pub status: Option<i16>,
    pub model_ids: Option<Vec<i64>>,
    pub total_flight_hours: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishFlightDto {
    pub aircraft_id: i64,
    pub subject_id: i64,
    pub flight_hours: Option<f64>,      // 本次飞行时长（增量）
    pub operation_hours: Option<f64>,   // 本次运行时长（增量）
    pub completed: bool,                // 是否完成该科目
}
#[derive(Debug, Deserialize)]
pub struct UpdatePilotDtoM {
    pub model_ids: Vec<i64>,  // 接收 { model_ids: [1,2,3] }
}

// ========== 服务结构体 ==========
#[derive(Clone)]
pub struct AircraftService {
    pub db: DatabaseConnection,
}

impl AircraftService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // 创建飞行员
    pub async fn create_pilot(&self, dto: CreatePilotDto) -> DbResult<pilot::Model> {
        let user = sys_user::Entity::find_by_id(dto.user_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbError::InvalidParameter("用户不存在".into()))?;

        let now = Utc::now();
        let active = pilot::ActiveModel {
            id: Default::default(),
            user_id: Set(dto.user_id),
            code: Set(dto.code),
            gender: Set(dto.gender),
            age: Set(dto.age),
            health_status: Set(dto.health_status),
            mental_status: Set(dto.mental_status),
            level: Set(dto.level),
            total_flight_hours: Set(Some(Decimal::ZERO)),
            flight_hours_by_type: Set(None),
            break_hours_by_type: Set(None),
            remark: Set(dto.remark),
            status: Set(dto.status.unwrap_or(1)),
            create_time: Set(now.naive_utc()),
            update_time: Set(now.naive_utc()),
        };
        let pilot = active.insert(&self.db).await?;

        if let Some(model_ids) = dto.model_ids {
            for model_id in model_ids {
                let rel = pilot_aircraft_model::ActiveModel {
                    pilot_id: Set(pilot.id),
                    model_id: Set(model_id),
                };
                rel.insert(&self.db).await?;
            }
        }
        Ok(pilot)
    }

    // 更新飞行员
    pub async fn update_pilot(&self, id: i64, dto: UpdatePilotDto) -> DbResult<pilot::Model> {
        let pilot = pilot::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let mut active: pilot::ActiveModel = pilot.into();
        if let Some(user_id) = dto.user_id {
            sys_user::Entity::find_by_id(user_id)
                .one(&self.db)
                .await?
                .ok_or_else(|| DbError::InvalidParameter("用户不存在".into()))?;
            active.user_id = Set(user_id);
        }
        if let Some(code) = dto.code { active.code = Set(Option::from(code)); }
        if let Some(gender) = dto.gender { active.gender = Set(Option::from(gender)); }
        if let Some(age) = dto.age { active.age = Set(Option::from(age)); }
        if let Some(health) = dto.health_status { active.health_status = Set(Option::from(health)); }
        if let Some(mental) = dto.mental_status { active.mental_status = Set(Option::from(mental)); }
        if let Some(level) = dto.level { active.level = Set(Option::from(level)); }
        if let Some(remark) = dto.remark { active.remark = Set(Option::from(remark)); }
        if let Some(status) = dto.status { active.status = Set(status); }
        if let Some(total_flight_hours) = dto.total_flight_hours {active.total_flight_hours = Set(Some(total_flight_hours));}
        active.update_time = Set(Utc::now().naive_utc());
        let updated = active.update(&self.db).await?;

        if let Some(model_ids) = dto.model_ids {
            // 先删除旧关联
            pilot_aircraft_model::Entity::delete_many()
                .filter(pilot_aircraft_model::Column::PilotId.eq(id))
                .exec(&self.db)
                .await?;
            for model_id in model_ids {
                let rel = pilot_aircraft_model::ActiveModel {
                    pilot_id: Set(id),
                    model_id: Set(model_id),
                };
                rel.insert(&self.db).await?;
            }
        }
        Ok(updated)
    }

    // 删除飞行员
    pub async fn delete_pilot(&self, id: i64) -> DbResult<()> {
        pilot::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    // 获取飞行员列表（含用户名）
    pub async fn list_pilots(&self) -> DbResult<Vec<serde_json::Value>> {
        let results = pilot::Entity::find()
            .find_also_related(sys_user::Entity)
            .all(&self.db)
            .await?;
        let mut list = Vec::new();
        for (p, u) in results {
            let username = u.map(|u| u.real_name.clone().unwrap_or_else(|| u.username.clone())).unwrap_or_default();
            list.push(serde_json::json!({
                "id": p.id,
                "user_id": p.user_id,
                "username": username,
                "code": p.code,
                "gender": p.gender,
                "age": p.age,
                "health_status": p.health_status,
                "mental_status": p.mental_status,
                "level": p.level,
                "total_flight_hours": p.total_flight_hours,
                "remark": p.remark,
                "status": p.status,
            }));
        }
        Ok(list)
    }

    // 获取单个飞行员详情
    pub async fn get_pilot(&self, id: i64) -> DbResult<Option<serde_json::Value>> {
        let result = pilot::Entity::find_by_id(id)
            .find_also_related(sys_user::Entity)
            .one(&self.db)
            .await?;
        if let Some((p, u)) = result {
            let username = u.map(|u| u.real_name.clone().unwrap_or_else(|| u.username.clone())).unwrap_or_default();
            Ok(Some(serde_json::json!({
                "id": p.id,
                "user_id": p.user_id,
                "username": username,
                "code": p.code,
                "gender": p.gender,
                "age": p.age,
                "health_status": p.health_status,
                "mental_status": p.mental_status,
                "level": p.level,
                "total_flight_hours": p.total_flight_hours,
                "flight_hours_by_type": p.flight_hours_by_type,
                "break_hours_by_type": p.break_hours_by_type,
                "remark": p.remark,
                "status": p.status,
            })))
        } else {
            Ok(None)
        }
    }

    // 获取飞行员的可飞机型列表
    pub async fn get_pilot_models(&self, pilot_id: i64) -> DbResult<Vec<aircraft_model::Model>> {
        Ok(aircraft_model::Entity::find()
            .inner_join(pilot_aircraft_model::Entity)
            .filter(pilot_aircraft_model::Column::PilotId.eq(pilot_id))
            .all(&self.db)
            .await?)
    }

    /// 设置飞行员的可飞机型（先删除旧关联，再插入新关联）
    pub async fn set_pilot_models(&self, pilot_id: i64, model_ids: UpdatePilotDtoM) -> DbResult<Vec<aircraft_model::Model>> {
        // 开启事务
        let txn = self.db.begin().await?;

        // 1. 删除该飞行员的所有旧机型关联
        pilot_aircraft_model::Entity::delete_many()
            .filter(pilot_aircraft_model::Column::PilotId.eq(pilot_id))
            .exec(&txn)
            .await?;

        // 2. 插入新关联
        for &model_id in &model_ids.model_ids {
            let active = pilot_aircraft_model::ActiveModel {
                pilot_id: Set(pilot_id),
                model_id: Set(model_id),
            };
            active.insert(&txn).await?;
        }

        // 3. 查询新关联的机型详情
        let models = aircraft_model::Entity::find()
            .filter(aircraft_model::Column::Id.is_in(model_ids.model_ids))
            .all(&txn)
            .await?;

        txn.commit().await?;
        Ok(models)
    }

    // ---------- 机型管理 ----------
    pub async fn create_model(&self, dto: CreateAircraftModelDto) -> DbResult<aircraft_model::Model> {
        let now = Utc::now();
        let active = aircraft_model::ActiveModel {
            id: Default::default(),
            name: Set(dto.name),
            code: Set(Option::from(dto.code)),
            description: Set(dto.description),
            status: Set(dto.status.unwrap_or(1)),
            create_time: Set(now.naive_utc()),
            update_time: Set(now.naive_utc()),
        };
        Ok(active.insert(&self.db).await?)
    }

    pub async fn update_model(&self, id: i64, dto: UpdateAircraftModelDto) -> DbResult<aircraft_model::Model> {
        let model = aircraft_model::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;
        let mut active: aircraft_model::ActiveModel = model.into();
        if let Some(name) = dto.name { active.name = Set(name); }
        if let Some(code) = dto.code { active.code = Set(Option::from(code)); }
        if let Some(desc) = dto.description { active.description = Set(Option::from(desc)); }
        if let Some(status) = dto.status { active.status = Set(status); }
        active.update_time = Set(Utc::now().naive_utc());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete_model(&self, id: i64) -> DbResult<()> {
        aircraft_model::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn list_models(&self) -> DbResult<Vec<aircraft_model::Model>> {
        Ok(aircraft_model::Entity::find()
            .order_by_asc(aircraft_model::Column::Name)
            .all(&self.db)
            .await?)
    }

    // ---------- 科目管理 ----------
    pub async fn create_subject(&self, dto: CreateSubjectDto) -> DbResult<subject::Model> {
        let now = Utc::now();
        let active = subject::ActiveModel {
            id: Default::default(),
            name: Set(dto.name),
            code: Set(dto.code),
            danger_level: Set(Option::from(dto.danger_level)),
            default_duration: Set(dto.default_duration),
            subject_type: Set(dto.subject_type),
            description: Set(dto.description),
            status: Set(dto.status.unwrap_or(1)),
            required_pilots: Set(dto.required_pilots.unwrap_or(1)),
            create_time: Set(now.naive_utc()),
            update_time: Set(now.naive_utc()),
        };
        Ok(active.insert(&self.db).await?)
    }

    pub async fn update_subject(&self, id: i64, dto: UpdateSubjectDto) -> DbResult<subject::Model> {
        let subj = subject::Entity::find_by_id(id).one(&self.db).await?.ok_or(DbError::RecordNotFound)?;
        let mut active: subject::ActiveModel = subj.into();
        if let Some(name) = dto.name { active.name = Set(name); }
        if let Some(code) = dto.code { active.code = Set(Option::from(code)); }
        if let Some(level) = dto.danger_level { active.danger_level = Set(Option::from(level)); }
        if let Some(dur) = dto.default_duration { active.default_duration = Set(dur); }
        if let Some(typ) = dto.subject_type { active.subject_type = Set(Option::from(typ)); }
        if let Some(desc) = dto.description { active.description = Set(Option::from(desc)); }
        if let Some(status) = dto.status { active.status = Set(status); }
        if let Some(pil) = dto.required_pilots {active.required_pilots = Set(pil); }
        active.update_time = Set(Utc::now().naive_utc());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete_subject(&self, id: i64) -> DbResult<()> {
        subject::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn list_subjects(&self) -> DbResult<Vec<subject::Model>> {
        Ok(subject::Entity::find().order_by_asc(subject::Column::Name).all(&self.db).await?)
    }

    // 获取机型的关联科目（用于飞机可执行的科目）
    pub async fn get_subjects_by_model(&self, model_id: i64) -> DbResult<Vec<subject::Model>> {
        let subjects = subject::Entity::find()
            .inner_join(model_subject::Entity)
            .filter(model_subject::Column::ModelId.eq(model_id))
            .all(&self.db)
            .await?;
        Ok(subjects)
    }

    // ---------- 飞机管理 ----------
    pub async fn create_aircraft(&self, dto: CreateAircraftDto) -> DbResult<aircraft::Model> {
        let now = Utc::now();
        let active = aircraft::ActiveModel {
            id: Default::default(),
            model_id: Set(dto.model_id),
            plane_no: Set(dto.plane_no),
            status: Set(Option::from(dto.status.unwrap_or_else(|| "测试中".to_string()))),
            remark: Set(dto.remark),
            extra_attrs: Set(dto.extra_attrs),
            total_running_hours: Default::default(),
            create_time: Set(now.naive_utc()),
            update_time: Set(now.naive_utc()),
        };
        let ac = active.insert(&self.db).await?;
        // 创建该飞机与机型关联科目的初始完成记录（未开始）
        let subjects = self.get_subjects_by_model(dto.model_id).await?;
        for subj in subjects {
            let comp_active = aircraft_subject_completion::ActiveModel {
                id: Default::default(),
                aircraft_id: Set(ac.id),
                subject_id: Set(subj.id),
                start_time: Set(None),
                end_time: Set(None),
                flight_hours: Set(None),
                operation_hours: Set(None),
                aircraft_running_hours: Set(None),
                completed: Set(false),
                completion_time: Set(None),
                remark: Set(None),
                create_time: Set(now.naive_utc()),
                update_time: Set(now.naive_utc()),
            };
            comp_active.insert(&self.db).await?;
        }
        Ok(ac)
    }

    pub async fn update_aircraft(&self, id: i64, dto: UpdateAircraftDto) -> DbResult<aircraft::Model> {
        let ac = aircraft::Entity::find_by_id(id).one(&self.db).await?.ok_or(DbError::RecordNotFound)?;
        let mut active: aircraft::ActiveModel = ac.into();
        if let Some(model_id) = dto.model_id { active.model_id = Set(model_id); }
        if let Some(plane_no) = dto.plane_no { active.plane_no = Set(plane_no); }
        if let Some(status) = dto.status { active.status = Set(Option::from(status)); }
        if let Some(remark) = dto.remark { active.remark = Set(Option::from(remark)); }
        if let Some(extra) = dto.extra_attrs { active.extra_attrs = Set(Option::from(extra)); }
        active.update_time = Set(Utc::now().naive_utc());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete_aircraft(&self, id: i64) -> DbResult<()> {
        aircraft::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn list_aircraft(&self) -> DbResult<Vec<aircraft::Model>> {
        Ok(aircraft::Entity::find().order_by_asc(aircraft::Column::PlaneNo).all(&self.db).await?)
    }

    // 获取飞机详细信息（含机型名称、可用科目及完成状态）
    pub async fn get_aircraft_detail(&self, id: i64) -> DbResult<serde_json::Value> {
        let ac = aircraft::Entity::find_by_id(id).one(&self.db).await?.ok_or(DbError::RecordNotFound)?;
        let model = aircraft_model::Entity::find_by_id(ac.model_id).one(&self.db).await?;
        let subjects = subject::Entity::find()
            .inner_join(model_subject::Entity)
            .filter(model_subject::Column::ModelId.eq(ac.model_id))
            .all(&self.db)
            .await?;
        let mut subjects_with_status = Vec::new();
        for subj in subjects {
            let comp = aircraft_subject_completion::Entity::find()
                .filter(aircraft_subject_completion::Column::AircraftId.eq(ac.id))
                .filter(aircraft_subject_completion::Column::SubjectId.eq(subj.id))
                .one(&self.db)
                .await?;
            subjects_with_status.push(serde_json::json!({
                "subject": subj,
                "completed": comp.clone().map(|c| c.completed).unwrap_or(false),
                "start_time": comp.clone().and_then(|c| c.start_time),
                "end_time": comp.clone().and_then(|c| c.end_time),
                "operation_hours": comp.clone().and_then(|c| c.operation_hours),
            }));
        }
        Ok(serde_json::json!({
            "aircraft": ac,
            "model": model,
            "subjects": subjects_with_status,
        }))
    }

    // ---------- 试飞记录 ----------
    pub async fn finish_flight(&self, dto: FinishFlightDto) -> DbResult<()> {
        let record = aircraft_subject_completion::Entity::find()
            .filter(aircraft_subject_completion::Column::AircraftId.eq(dto.aircraft_id))
            .filter(aircraft_subject_completion::Column::SubjectId.eq(dto.subject_id))
            .one(&self.db)
            .await?
            .ok_or(DbError::RecordNotFound)?;

        let now = Utc::now().naive_utc();
        let mut active: aircraft_subject_completion::ActiveModel = record.clone().into_active_model();

        // start_time 为空时写入当前时间
        if record.start_time.is_none() {
            active.start_time = Set(Some(now));
        }

        // 每次操作都记录结束时间
        active.end_time = Set(Some(now));

        // 累加 flight_hours（增量）
        if let Some(fh) = dto.flight_hours {
            let delta = Decimal::from_f64(fh).unwrap_or(Decimal::ZERO);
            let current = record.flight_hours.unwrap_or(Decimal::ZERO);
            active.flight_hours = Set(Some(current + delta));
        }

        // 累加 operation_hours（增量）
        if let Some(oh) = dto.operation_hours {
            let delta = Decimal::from_f64(oh).unwrap_or(Decimal::ZERO);
            let current = record.operation_hours.unwrap_or(Decimal::ZERO);
            active.operation_hours = Set(Some(current + delta));

            // 同时累加 aircraft_running_hours
            let current_running = record.aircraft_running_hours.unwrap_or(Decimal::ZERO);
            active.aircraft_running_hours = Set(Some(current_running + delta));
        }

        if !dto.completed {
            active.completed = Set(false);
        }

        // 完成科目时设置完成标记和时间
        if dto.completed {
            active.completed = Set(true);
            active.completion_time = Set(Some(now));
        }

        active.update_time = Set(now);
        active.update(&self.db).await?;
        Ok(())
    }

    // 查询飞机的所有未完成科目（可用于待办列表）
    pub async fn get_incomplete_subjects(&self, aircraft_id: i64) -> DbResult<Vec<subject::Model>> {
        let ac = aircraft::Entity::find_by_id(aircraft_id).one(&self.db).await?.ok_or(DbError::RecordNotFound)?;
        let all_subjects = self.get_subjects_by_model(ac.model_id).await?;
        let completed_ids: Vec<i64> = aircraft_subject_completion::Entity::find()
            .filter(aircraft_subject_completion::Column::AircraftId.eq(aircraft_id))
            .filter(aircraft_subject_completion::Column::Completed.eq(true))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|c| c.subject_id)
            .collect();
        let incomplete = all_subjects.into_iter().filter(|s| !completed_ids.contains(&s.id)).collect();
        Ok(incomplete)
    }
}