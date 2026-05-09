use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::error::{AppError, AppResult};
use crate::utils::response::success_response;
use crate::milp_client::solve_plan;

// ==================== 请求/响应类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PilotInfo {
    pub id: i64,
    pub health_status: String,
    pub mental_status: String,
    pub level: String,
    #[serde(default)]
    pub max_daily_flight_hours: Option<f64>,
    pub allowed_aircraft_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftInfo {
    pub id: i64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub aircraft_id: i64,
    pub duration: f64,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default)]
    pub risk: i32,
    #[serde(default = "default_required_pilots")]
    pub required_pilots: i32,
}

fn default_weight() -> f64 { 1.0 }
fn default_required_pilots() -> i32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilpConstraintConfig {
    pub enable_work_time: bool,
    pub work_start: i32,
    pub work_end: i32,
    pub check_health: bool,
    pub check_mental: bool,
    pub match_aircraft_type: bool,
    pub high_risk_need_level1: bool,
    pub max_duration: f64,
    pub max_daily_flight_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilpScheduleRequest {
    pub pilots: Vec<PilotInfo>,
    pub aircrafts: Vec<AircraftInfo>,
    pub tasks: Vec<TaskInfo>,
    pub constraints: MilpConstraintConfig,
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

// ==================== 构建求解器输入 ====================

fn build_milp_input(req: &MilpScheduleRequest) -> Result<serde_json::Value, AppError> {
    let pilots = &req.pilots;
    let aircrafts = &req.aircrafts;
    let tasks = &req.tasks;
    let cfg = &req.constraints;

    // --- 飞行员属性映射 ---
    let pilot_list: Vec<String> = pilots.iter().map(|p| p.id.to_string()).collect();
    let mut health_map = serde_json::Map::new();
    let mut mental_map = serde_json::Map::new();
    let mut level_map = serde_json::Map::new();
    let mut max_daily_map = serde_json::Map::new();

    for p in pilots {
        let id_str = p.id.to_string();

        // 健康检查：仅当启用且健康状态为“良好”时合格，否则启用忽略检查时一律合格
        let health_val = if cfg.check_health {
            if p.health_status == "良好" { 1 } else { 0 }
        } else { 1 };
        // 心理健康检查：启用时要求“优秀”或“良好”
        let mental_val = if cfg.check_mental {
            if p.mental_status == "优秀" || p.mental_status == "良好" { 1 } else { 0 }
        } else { 1 };
        // 高危任务一级飞行员要求（所有任务通用此检查，求解器内针对risk=1的任务会进一步过滤）
        let level_val = if cfg.high_risk_need_level1 {
            if p.level == "一级飞行员" { 1 } else { 0 }
        } else { 1 };
        let max_daily = p.max_daily_flight_hours.unwrap_or(cfg.max_daily_flight_hours);

        health_map.insert(id_str.clone(), json!(health_val));
        mental_map.insert(id_str.clone(), json!(mental_val));
        level_map.insert(id_str.clone(), json!(level_val));
        max_daily_map.insert(id_str, json!(max_daily));
    }

    // --- 飞机列表 ---
    let aircraft_list: Vec<String> = aircrafts.iter().map(|a| a.id.to_string()).collect();

    // --- 任务列表 ---
    let tasks_json: Vec<serde_json::Value> = tasks
        .iter()
        .map(|t| {
            if t.duration > cfg.max_duration {
                // 若严格校验可在上层拦截，此处仅为稳妥保留检查
                return json!({
                    "error": format!("任务 {} 持续时间 {} 超过最大允许 {} ", t.id, t.duration, cfg.max_duration)
                });
            }
            json!({
                "id": t.id,
                "duration": t.duration,
                "weight": t.weight,
                "risk": t.risk,
                "required_pilots": t.required_pilots,
                "aircraft": t.aircraft_id.to_string(),
            })
        })
        .collect();

    // --- 资质矩阵 allow ---
    let mut allow = Vec::new();
    for p in pilots {
        for a in aircrafts {
            let allowed = if cfg.match_aircraft_type {
                p.allowed_aircraft_models.contains(&a.model) || p.allowed_aircraft_models.is_empty()
            } else {
                true
            };
            allow.push(vec![
                json!(p.id.to_string()),
                json!(a.id.to_string()),
                json!(if allowed { 1 } else { 0 }),
            ]);
        }
    }

    // --- 工作时间窗口 ---
    let work_start = if cfg.enable_work_time { cfg.work_start } else { 8 };
    let work_end = if cfg.enable_work_time { cfg.work_end } else { 18 };
    let time_slots: Vec<i32> = (work_start..work_end).collect();

    // --- 组装最终输入 ---
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
        "max_duration_per_task": cfg.max_duration,
    });

    Ok(input)
}

// ==================== 控制器 ====================

pub async fn schedule(
    req: web::Json<MilpScheduleRequest>,
) -> AppResult<HttpResponse> {
    let input = build_milp_input(&req)?;
    // 调用 Python 求解器
    let result = solve_plan(&input).map_err(|e| AppError::InternalError(e.to_string()))?;

    let status = result["status"].as_str().unwrap_or("UNKNOWN").to_string();
    let objective = result["objective"].as_f64().unwrap_or(0.0);
    let default_empty = vec![];
    let assignments_raw = result["assignments"].as_array().unwrap_or(&default_empty);

    let mut assignments = Vec::new();
    for assign in assignments_raw {
        let task_id = assign["task_id"].as_str().unwrap_or("").to_string();
        let pilot_ids: Vec<i64> = assign["pilots"]
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
            pilot_id: pilot_ids,
            aircraft_id,
            start_hour,
            duration,
        });
    }

    let response = ScheduleResponse {
        status,
        objective,
        assignments,
    };
    Ok(success_response(response))
}