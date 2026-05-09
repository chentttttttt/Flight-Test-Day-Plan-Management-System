// src/controller/milp_schedule.rs

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::utils::response::success_response;
use crate::milp_client::solve_plan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub id: String,
    pub duration: f64,
    pub weight: f64,
    pub risk: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfig {
    pub enable_work_time: bool,
    pub work_start: i32,
    pub work_end: i32,
    pub check_health: bool,
    pub check_mental: bool,
    pub match_aircraft_type: bool,
    pub max_tasks_per_pilot: i32,
    pub high_risk_need_level1: bool,
    pub max_duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRequest {
    // tasks 字段已废弃，实际任务从飞机属性中自动生成，保留仅为兼容旧请求
    #[serde(default)]
    pub tasks: Vec<TaskInput>,
    pub constraints: ConstraintConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub task_id: String,
    pub pilot_id: Vec<i64>,
    pub aircraft_id: i64,
    pub start_hour: i32,
    pub duration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub status: String,
    pub objective: f64,
    pub assignments: Vec<Assignment>,
}

/// 构建 MILP 求解器所需的 JSON 输入
async fn build_solver_input(
    services: &ServiceContainer,
    _tasks: Vec<TaskInput>,  // 忽略请求中的任务，改为从飞机属性自动生成
    constraints: ConstraintConfig,
) -> Result<serde_json::Value, AppError> {
    // 获取飞行员和飞机数据
    let pilots = services.abac_subject_attribute.get_all_by_subject_type("PILOT").await?;
    let aircrafts = services.resource_attribute.list_all_attrs_by_type("AIRCRAFT").await?;

    // ----- 构建飞行员相关映射 -----
    let mut pilot_list = Vec::new();
    let mut health_map = serde_json::Map::new();
    let mut mental_map = serde_json::Map::new();
    let mut level_map = serde_json::Map::new();
    let mut max_daily_map = serde_json::Map::new();

    for p in &pilots {
        let attrs = &p.attr_key_value;
        let id_str = p.subject_id.to_string();

        let health = if constraints.check_health {
            if attrs.get("health_status").and_then(|v| v.as_str()) == Some("良好") { 1 } else { 0 }
        } else { 1 };
        let mental = if constraints.check_mental {
            if let Some(s) = attrs.get("mental_status").and_then(|v| v.as_str()) {
                if s == "稳定" || s == "优秀" { 1 } else { 0 }
            } else { 0 }
        } else { 1 };
        let level = if constraints.high_risk_need_level1 {
            if attrs.get("level").and_then(|v| v.as_str()) == Some("一级飞行员") { 1 } else { 0 }
        } else { 1 };
        let max_daily = attrs.get("max_daily_flight_hours").and_then(|v| v.as_i64()).unwrap_or(8);

        pilot_list.push(id_str.clone());
        health_map.insert(id_str.clone(), json!(health));
        mental_map.insert(id_str.clone(), json!(mental));
        level_map.insert(id_str.clone(), json!(level));
        max_daily_map.insert(id_str, json!(max_daily));
    }

    // ----- 构建飞机列表，并从飞机属性中生成任务 -----
    let mut aircraft_list = Vec::new();
    let mut tasks_json = Vec::new();

    for a in &aircrafts {
        let attrs = &a.attr_key_value;
        let aircraft_id = a.resource_id.to_string();
        aircraft_list.push(aircraft_id.clone());
        println!("{:#?}",attrs);
        // 读取飞机属性（提供默认值）
        let subjects_str = attrs.get("subject").and_then(|v| v.as_str()).unwrap_or("");
        let default_duration = attrs.get("default_duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(2.0);
        let required_pilots = attrs.get("required_pilots")
            .and_then(|v| v.as_str())          // 先读字符串
            .and_then(|s| s.parse::<i64>().ok()) // 转成数字
            .unwrap_or(1) as i32;
        let danger_level = attrs.get("danger_level").and_then(|v| v.as_str()).unwrap_or("");
        let risk = if constraints.high_risk_need_level1 && danger_level == "高危" { 1 } else { 0 };

        // 检查时长是否超过最大限制
        if default_duration > constraints.max_duration {
            return Err(AppError::BusinessError(format!(
                "飞机 {} 的默认时长 {} 超过最大允许时长 {}",
                aircraft_id, default_duration, constraints.max_duration
            )));
        }

        // 解析科目（以英文逗号分割）
        let subjects: Vec<&str> = if subjects_str.is_empty() {
            // 若无科目，可生成一个默认任务，或跳过该飞机
            vec!["基础飞行"]
        } else {
            subjects_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
        };

        for subject in subjects {
            let task_id = format!("{}_{}", aircraft_id, subject);
            tasks_json.push(json!({
                "id": task_id,
                "duration": default_duration,
                "weight": 1.0,                     // 默认权重，可根据需要调整
                "risk": risk,
                "required_pilots": required_pilots, // 关键字段：所需飞行员数量
                "aircraft": aircraft_id,
            }));
        }
    }

    // 如果没有任何任务生成，返回错误
    if tasks_json.is_empty() {
        return Err(AppError::BusinessError("没有找到任何可执行的任务（飞机无科目或属性缺失）".to_string()));
    }

    // ----- 构建资质矩阵 allow -----
    let mut allow = Vec::new();
    for p in &pilots {
        let allowed_aircraft = p.attr_key_value.get("allowed_aircraft").and_then(|v| v.as_str()).unwrap_or("");
        let allowed_models: Vec<&str> = allowed_aircraft.split(',').map(|s| s.trim()).collect();
        for a in &aircrafts {
            let model = a.attr_key_value.get("model").and_then(|v| v.as_str()).unwrap_or("");
            let allowed = if constraints.match_aircraft_type {
                allowed_models.contains(&model) || allowed_aircraft.is_empty()
            } else {
                true
            };
            allow.push(vec![
                json!(p.subject_id.to_string()),
                json!(a.resource_id.to_string()),
                json!(if allowed { 1 } else { 0 }),
            ]);
        }
    }

    // ----- 工作时间窗口 -----
    let work_start = if constraints.enable_work_time { constraints.work_start } else { 8 };
    let work_end = if constraints.enable_work_time { constraints.work_end } else { 18 };
    let time_slots: Vec<i32> = (work_start..work_end).collect();

    // ----- 组装最终输入 -----
    let input = json!({
        "pilots": pilot_list,
        "aircrafts": aircraft_list,
        "tasks": tasks_json,
        "time_slots": time_slots,
        "work_start": work_start,
        "work_end": work_end,
        "allow": allow,
        "health": health_map,
        "mental": mental_map,
        "level": level_map,
        "max_daily": max_daily_map,
        "max_duration_per_task": constraints.max_duration,
    });
    println!("{:#?}", input);
    Ok(input)
}

/// 调度接口入口
pub async fn schedule(
    services: web::Data<ServiceContainer>,
    req: web::Json<ScheduleRequest>,
) -> AppResult<HttpResponse> {
    let input = build_solver_input(&services, req.tasks.clone(), req.constraints.clone()).await?;
    let result = solve_plan(&input).map_err(|e| AppError::InternalError(e.to_string()))?;

    let status = result["status"].as_str().unwrap_or("UNKNOWN").to_string();
    let objective = result["objective"].as_f64().unwrap_or(0.0);
    let default_empty = vec![];
    let assignments_raw = result["assignments"].as_array().unwrap_or(&default_empty);

    let mut assignments = Vec::new();
    for assign in assignments_raw {
        let task_id = assign["task_id"].as_str().unwrap_or("").to_string();
        // 注意：求解器返回的 pilot/aircraft 字段是字符串，需解析为 i64
        // let pilot_id = assign["pilots"]
        //     .as_array()
        //     .and_then(|arr| arr.first())
        //     .and_then(|v| v.as_str())
        //     .and_then(|s| s.parse::<i64>().ok())
        //     .unwrap_or(0);
        // 改进：返回所有飞行员 ID 列表
        let pilot_id: Vec<i64> = assign["pilots"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()))
            .collect();
        let aircraft_id = assign["aircraft"].as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        let start_hour = assign["start_hour"].as_i64().unwrap_or(0) as i32;
        let duration = assign["duration"].as_f64().unwrap_or(0.0);
        assignments.push(Assignment {
            task_id,
            pilot_id,
            aircraft_id,
            start_hour,
            duration,
        });
    }

    let response = ScheduleResponse { status, objective, assignments };
    Ok(success_response(response))
}