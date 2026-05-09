// src/handlers/abac_policy_rule.rs

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::entity::abac_policy_rule;
use crate::error::AppResult;
use crate::middleware::perm::DynamicRoleBasedAbac;
use crate::service::ServiceContainer;
use crate::service::abac_policy_rule::AbacPolicyRuleUpdateDto;
use crate::utils::conf::Normalize;
use crate::utils::response::{success_response};

// ---------- DTO ----------
#[derive(Debug, Deserialize)]
pub struct CreateRuleReq {
    pub rule_code: String,
    pub rule_name: String,
    pub subject_type: String,
    pub subject_id: Option<i64>,
    pub resource_type: String,
    pub resource_id: Option<i64>,
    pub action: String,
    pub condition_json: Value,
    pub effect: String,
    pub priority: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleReq {
    pub rule_code: Option<String>,
    pub rule_name: Option<String>,
    pub subject_type: Option<String>,
    pub subject_id: Option<i64>,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub action: Option<String>,
    pub condition_json: Option<Value>,
    pub effect: Option<String>,
    pub priority: Option<i32>,
    pub status: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct QueryRulesReq {
    pub subject_type: Option<String>,
    pub resource_type: Option<String>,
    pub action: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct FindRuleReq {
    pub subject_type: String,
    pub subject_id: Option<i64>,
    pub resource_type: String,
    pub resource_id: Option<i64>,
    pub action: String,
}

// ---------- Handlers ----------

/// 创建规则
pub async fn create_rule(
    services: web::Data<ServiceContainer>,
    engine: web::Data<DynamicRoleBasedAbac>,
    body: web::Json<CreateRuleReq>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_policy_rule;
    let model = abac_policy_rule::Model {
        id: 0,
        rule_code: body.rule_code.clone(),
        rule_name: body.rule_name.clone(),
        subject_type: body.subject_type.clone(),
        subject_id: body.subject_id,
        resource_type: body.resource_type.clone(),
        resource_id: body.resource_id,
        action: body.action.clone(),
        condition_json: Option::from(body.condition_json.clone()),
        effect: body.effect.clone(),
        priority: body.priority,
        status: 1,
        deleted: 0,
        create_time: Default::default(),
        update_time: Default::default(),
    };
    let rule = s.create(model).await?;
    // 刷新引擎
    if let Err(e) = engine.refresh().await {
        tracing::error!("刷新 ABAC 引擎失败: {}", e);
    }
    Ok(success_response(rule))
}

/// 查询所有规则
pub async fn list_all_rules(
    services: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_policy_rule;
    let rules = s.find_all().await?;
    Ok(success_response(rules))
}

/// 按条件查询规则
pub async fn query_rules(
    services: web::Data<ServiceContainer>,
    query: web::Query<QueryRulesReq>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_policy_rule;
    let rules = s.find_by_condition(
        query.subject_type.clone().normalize(),
        query.resource_type.clone().normalize(),
        query.action.clone().normalize(),
        query.status.clone(),
    ).await?;
    Ok(success_response(rules))
}

/// 更新规则
pub async fn update_rule(
    services: web::Data<ServiceContainer>,
    engine: web::Data<DynamicRoleBasedAbac>,
    id: web::Path<i64>,
    body: web::Json<UpdateRuleReq>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_policy_rule;
    let dto = AbacPolicyRuleUpdateDto {
        rule_code: body.rule_code.clone(),
        rule_name: body.rule_name.clone(),
        subject_type: body.subject_type.clone(),
        subject_id: body.subject_id,
        resource_type: body.resource_type.clone(),
        resource_id: body.resource_id,
        action: body.action.clone(),
        condition_json: body.condition_json.clone(),
        effect: body.effect.clone(),
        priority: body.priority,
        status: body.status,
    };
    let rule = s.update(*id, dto).await?;
    // 刷新引擎
    if let Err(e) = engine.refresh().await {
        tracing::error!("刷新 ABAC 引擎失败: {}", e);
    }
    Ok(success_response(rule))
}

/// 删除规则
pub async fn delete_rule(
    services: web::Data<ServiceContainer>,
    engine: web::Data<DynamicRoleBasedAbac>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_policy_rule;
    s.delete(*id).await?;
    // 刷新引擎
    if let Err(e) = engine.refresh().await {
        tracing::error!("刷新 ABAC 引擎失败: {}", e);
    }
    Ok(success_response(()))
}

/// 查找最匹配的规则（用于权限校验）
pub async fn find_rule(
    services: web::Data<ServiceContainer>,
    body: web::Json<FindRuleReq>,
) -> AppResult<HttpResponse> {
    let s = &services.abac_policy_rule;
    let rule = s.find_rules(
        &body.subject_type,
        body.subject_id,
        &body.resource_type,
        body.resource_id,
        &body.action,
    ).await?;
    Ok(success_response(rule))
}