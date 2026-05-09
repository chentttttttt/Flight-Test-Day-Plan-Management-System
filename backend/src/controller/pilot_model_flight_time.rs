use actix_web::{web, HttpResponse};
use crate::error::AppResult;
use crate::service::ServiceContainer;
use crate::service::pilot_model_flight_time::{CreatePilotModelFlightTimeDto, UpdatePilotModelFlightTimeDto, ListQuery};
use crate::utils::response::success_response;

/// 新增
pub async fn create(
    container: web::Data<ServiceContainer>,
    dto: web::Json<CreatePilotModelFlightTimeDto>,
) -> AppResult<HttpResponse> {
    let record = container.pilot_model_flight_time_service.create(dto.into_inner()).await?;
    Ok(success_response(record))
}

/// 根据 ID 查询
pub async fn get_by_id(
    container: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let record = container.pilot_model_flight_time_service.get_by_id(id).await?
        .ok_or_else(|| crate::error::AppError::RecordNotFound)?;
    Ok(success_response(record))
}

/// 根据 pilot_id + model_id 查询
pub async fn get_by_pilot_model(
    container: web::Data<ServiceContainer>,
    path: web::Path<(i64, i64)>,
) -> AppResult<HttpResponse> {
    let (pilot_id, model_id) = path.into_inner();
    let record = container.pilot_model_flight_time_service
        .get_by_pilot_and_model(pilot_id, model_id)
        .await?
        .ok_or_else(|| crate::error::AppError::RecordNotFound)?;
    Ok(success_response(record))
}

/// 列表查询
pub async fn list(
    container: web::Data<ServiceContainer>,
    query: web::Query<ListQuery>,
) -> AppResult<HttpResponse> {
    let (items) = container.pilot_model_flight_time_service.list(query.into_inner()).await?;
    Ok(success_response(serde_json::json!({
        "list": items,
    })))
}

/// 更新（根据 ID）
pub async fn update(
    container: web::Data<ServiceContainer>,
    path: web::Path<i64>,
    dto: web::Json<UpdatePilotModelFlightTimeDto>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    let updated = container.pilot_model_flight_time_service.update(id, dto.into_inner()).await?;
    Ok(success_response(updated))
}

/// Upsert
pub async fn upsert(
    container: web::Data<ServiceContainer>,
    dto: web::Json<CreatePilotModelFlightTimeDto>,
) -> AppResult<HttpResponse> {
    let record = container.pilot_model_flight_time_service.upsert(dto.into_inner()).await?;
    Ok(success_response(record))
}

/// 根据 ID 删除
pub async fn delete(
    container: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    container.pilot_model_flight_time_service.delete(id).await?;
    Ok(success_response(()))
}

/// 根据 pilot_id 批量删除
pub async fn delete_by_pilot(
    container: web::Data<ServiceContainer>,
    path: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let pilot_id = path.into_inner();
    let affected = container.pilot_model_flight_time_service.delete_by_pilot(pilot_id).await?;
    Ok(success_response(serde_json::json!({ "deleted": affected })))
}