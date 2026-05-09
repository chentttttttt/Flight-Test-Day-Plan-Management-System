use actix_web::{web, HttpResponse};
use crate::error::{AppError, AppResult};
use crate::service::ServiceContainer;
use crate::service::aircraft_service::*;
use crate::utils::response::success_response;

// ---------- 机型 ----------
pub async fn create_model(
    svc: web::Data<ServiceContainer>,
    dto: web::Json<CreateAircraftModelDto>,
) -> AppResult<HttpResponse> {
    let model = svc.aircraft.create_model(dto.into_inner()).await?;
    Ok(success_response(model))
}

pub async fn update_model(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<UpdateAircraftModelDto>,
) -> AppResult<HttpResponse> {
    let model = svc.aircraft.update_model(*id, dto.into_inner()).await?;
    Ok(success_response(model))
}

pub async fn delete_model(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    svc.aircraft.delete_model(*id).await?;
    Ok(success_response(()))
}

pub async fn list_models(
    svc: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let list = svc.aircraft.list_models().await?;
    Ok(success_response(list))
}

// ---------- 科目 ----------
pub async fn create_subject(
    svc: web::Data<ServiceContainer>,
    dto: web::Json<CreateSubjectDto>,
) -> AppResult<HttpResponse> {
    let subj = svc.aircraft.create_subject(dto.into_inner()).await?;
    Ok(success_response(subj))
}

pub async fn update_subject(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<UpdateSubjectDto>,
) -> AppResult<HttpResponse> {
    let subj = svc.aircraft.update_subject(*id, dto.into_inner()).await?;
    Ok(success_response(subj))
}

pub async fn delete_subject(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    svc.aircraft.delete_subject(*id).await?;
    Ok(success_response(()))
}

pub async fn list_subjects(
    svc: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let list = svc.aircraft.list_subjects().await?;
    Ok(success_response(list))
}

// ---------- 飞机 ----------
pub async fn create_aircraft(
    svc: web::Data<ServiceContainer>,
    dto: web::Json<CreateAircraftDto>,
) -> AppResult<HttpResponse> {
    let ac = svc.aircraft.create_aircraft(dto.into_inner()).await?;
    Ok(success_response(ac))
}

pub async fn update_aircraft(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<UpdateAircraftDto>,
) -> AppResult<HttpResponse> {
    let ac = svc.aircraft.update_aircraft(*id, dto.into_inner()).await?;
    Ok(success_response(ac))
}

pub async fn delete_aircraft(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    svc.aircraft.delete_aircraft(*id).await?;
    Ok(success_response(()))
}

pub async fn list_aircraft(
    svc: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let list = svc.aircraft.list_aircraft().await?;
    Ok(success_response(list))
}

pub async fn aircraft_detail(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let detail = svc.aircraft.get_aircraft_detail(*id).await?;
    Ok(success_response(detail))
}

// ---------- 试飞记录 ----------
pub async fn finish_flight(
    svc: web::Data<ServiceContainer>,
    dto: web::Json<FinishFlightDto>,
) -> AppResult<HttpResponse> {
    svc.aircraft.finish_flight(dto.into_inner()).await?;
    Ok(success_response(()))
}

pub async fn incomplete_subjects(
    svc: web::Data<ServiceContainer>,
    aircraft_id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let subjects = svc.aircraft.get_incomplete_subjects(*aircraft_id).await?;
    Ok(success_response(subjects))
}

pub async fn create_pilot(
    svc: web::Data<ServiceContainer>,
    dto: web::Json<CreatePilotDto>,
) -> AppResult<HttpResponse> {
    let pilot = svc.aircraft.create_pilot(dto.into_inner()).await?;
    Ok(success_response(pilot))
}

pub async fn update_pilot(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<UpdatePilotDto>,
) -> AppResult<HttpResponse> {
    let pilot = svc.aircraft.update_pilot(*id, dto.into_inner()).await?;
    Ok(success_response(pilot))
}

pub async fn delete_pilot(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    svc.aircraft.delete_pilot(*id).await?;
    Ok(success_response(()))
}

pub async fn list_pilots(
    svc: web::Data<ServiceContainer>,
) -> AppResult<HttpResponse> {
    let pilots = svc.aircraft.list_pilots().await?;
    Ok(success_response(pilots))
}

pub async fn get_pilot(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let pilot = svc.aircraft.get_pilot(*id).await?;
    Ok(success_response(pilot))
}

pub async fn get_pilot_models(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
) -> AppResult<HttpResponse> {
    let models = svc.aircraft.get_pilot_models(*id).await?;
    Ok(success_response(models))
}

pub async fn set_pilot_models(
    svc: web::Data<ServiceContainer>,
    id: web::Path<i64>,
    dto: web::Json<UpdatePilotDtoM>,
) -> AppResult<HttpResponse> {
    let models = svc.aircraft.set_pilot_models(*id, dto.into_inner()).await?;
    Ok(success_response(models))
}